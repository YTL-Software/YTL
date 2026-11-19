use crate::syntax::{SyntaxNode, YtlKind};
use crate::validators::{self, ValidationError};
use crate::fs_utils::resolve_path;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Position, Range, Uri};
use std::collections::{HashSet};
use std::path::{Path, PathBuf};
use crate::locales;

// Добавляем Display для ValidationError, чтобы можно было использовать {} в format!
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn validate(
    cst: &SyntaxNode,
    text: &str,
    uri: &Uri,
    root_path: Option<&Path>,
    locale: &locales::Locale,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut const_fields: Vec<String> = Vec::new();
    let mut seen_block = false;

    for node in cst.descendants() {
        match node.kind() {
            YtlKind::Block => {
                seen_block = true;
            }
            YtlKind::ConstKeyword => {
                if let Some(field_name) = node
                    .children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .filter(|t| t.kind() == YtlKind::FieldName)
                    .last()
                {
                    const_fields.push(field_name.text().trim().to_string());
                }
            }
            YtlKind::Realm => {
                let mut parent = node.parent();
                let mut is_inside_block = false;
                while let Some(p) = parent {
                    if p.kind() == YtlKind::Block {
                        is_inside_block = true;
                        break;
                    }
                    parent = p.parent();
                }
                if !is_inside_block {
                    let range = node.text_range();
                    diagnostics.push(create_err(
                        text,
                        range.start(),
                        range.end(),
                        "Realm must be inside a block",
                    ));
                }
            }
            YtlKind::Field => {
                if let Some(field_name) = node
                    .children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .find(|t| t.kind() == YtlKind::FieldName)
                {
                    let name = field_name.text().trim().to_string();
                    let range = node.text_range();

                    let values: Vec<String> = node
                        .children_with_tokens()
                        .filter_map(|el| el.into_token())
                        .filter(|t| t.kind() == YtlKind::FieldValue)
                        .map(|t| t.text().trim().to_string())
                        .collect();

                    if let Some(parent) = node.parent() {
                        let is_in_root = parent.kind() == YtlKind::Root;
                        let start_pos = offset_to_position(text, range.start());
                        let has_indent = start_pos.character > 0;

                        if is_in_root && has_indent {
                            diagnostics.push(create_err(
                                text,
                                range.start(),
                                range.end(),
                                "Unexpected indentation: no block is open above this line",
                            ));
                        } else {
                            if name == "Include" {
                                if !is_in_root {
                                    diagnostics.push(create_err(
                                        text,
                                        range.start(),
                                        range.end(),
                                        "Include must be at the root level",
                                    ));
                                } else if seen_block {
                                    diagnostics.push(create_err(
                                        text,
                                        range.start(),
                                        range.end(),
                                        "Include must be at the top of the file (before any blocks)",
                                    ));
                                }
                            }

                            if name == "Attach" && is_in_root {
                                diagnostics.push(create_err(
                                    text,
                                    range.start(),
                                    range.end(),
                                    "Attach must be inside a block",
                                ));
                            }
                        }
                    }

                    // === ПРОВЕРКА ATTACH НА ПЕРЕОПРЕДЕЛЕНИЕ ===
                    if name == "Attach" {
                        if let Some(val) = values.first() {
                            if val.starts_with("ytl://") {
                                let rel_path = val.trim_start_matches("ytl://");
                                let file_dir = if let Some(path) = uri.to_file_path() {
                                    path.parent().unwrap_or(Path::new(".")).to_path_buf()
                                } else {
                                    PathBuf::from(".")
                                };
                                let target_path = resolve_path(rel_path, &file_dir, root_path);

                                if let Some((_content, attached_cst)) = crate::fs_utils::read_ytl_file(&target_path) {
                                    let mut attached_fields = HashSet::new();
                                    for child in attached_cst.descendants() {
                                        if child.kind() == YtlKind::Field {
                                            if let Some(f_name) = child.children_with_tokens()
                                                .filter_map(|el| el.into_token())
                                                .find(|t| t.kind() == YtlKind::FieldName)
                                                .map(|t| t.text().trim().to_string())
                                            {
                                                attached_fields.insert(f_name);
                                            }
                                        }
                                    }

                                    if let Some(parent_block) = node.parent() {
                                        for sibling in parent_block.children() {
                                            if sibling.kind() == YtlKind::Field {
                                                if let Some(sib_name) = sibling.children_with_tokens()
                                                    .filter_map(|el| el.into_token())
                                                    .find(|t| t.kind() == YtlKind::FieldName)
                                                    .map(|t| t.text().trim().to_string())
                                                {
                                                    if attached_fields.contains(&sib_name) {
                                                        let sib_range = sibling.text_range();
                                                        diagnostics.push(create_err(
                                                            text,
                                                            sib_range.start(),
                                                            sib_range.end(),
                                                            &format!("Cannot override field '{}' from Attach. Create a new component instead.", sib_name),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // === УМНАЯ ВАЛИДАЦИЯ (С ТИПОМ И БЕЗ) ===
                    let name_lower = name.to_lowercase();
                    
                    for val in values {
                        if val.starts_with("ytl://") || val.starts_with("http://") || val.starts_with("https://") || val.is_empty() {
                            continue;
                        }
                        if val.contains('>') {
                            let parts: Vec<&str> = val.splitn(2, '>').map(|s| s.trim()).collect();
                            if parts.len() != 2 {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(text, val_range.start(), val_range.end(), "Invalid type syntax. Expected '> type value'"));
                                continue;
                            }

                            let right_side = parts[1];

                            let mut split_iter = right_side.splitn(2, char::is_whitespace);
                            let type_name = split_iter.next().unwrap_or("").trim();
                            
                            let value_to_check = split_iter.next().unwrap_or("").trim();

                            if type_name.is_empty() || value_to_check.is_empty() {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(text, val_range.start(), val_range.end(), "Missing value after type (e.g. ip > ipv4 192.168.1.1)"));
                                continue;
                            }

                            let mut clean_value = value_to_check;
                            if clean_value.len() >= 2 && (
                                (clean_value.starts_with('"') && clean_value.ends_with('"')) || 
                                (clean_value.starts_with('\'') && clean_value.ends_with('\''))
                            ) {
                                clean_value = &clean_value[1..clean_value.len()-1];
                            }

                            let result = match type_name {
                                "ipv4" => validators::validate_ipv4(clean_value, locale),
                                "ipv6" => validators::validate_ipv6(clean_value, locale),
                                "cidr" => validators::validate_cidr(clean_value, locale),
                                "mac"  => validators::validate_mac(clean_value, locale),
                                "uuid" => validators::validate_uuid(clean_value, locale),
                                _ => Err(ValidationError {
                                    title: format!("Unknown type: '{}'", type_name),
                                    message: "Allowed types: ipv4, ipv6, cidr, mac, uuid.".to_string(),
                                }),
                            };

                            if let Err(reason) = result {
                                let node_start_offset = usize::from(node.text_range().start());
                                let node_end_offset = usize::from(node.text_range().end());
                                
                                // Безопасный срез: проверяем, что не выходим за пределы текста
                                let node_text = if node_end_offset <= text.len() {
                                    &text[node_start_offset..node_end_offset]
                                } else {
                                    ""
                                };

                                let mut final_start = node.text_range().start();
                                let mut final_end = node.text_range().end();

                                if let Some(arrow_idx) = node_text.find('>') {
                                    let after_arrow = &node_text[arrow_idx + 1..];
                                    
                                    if !clean_value.is_empty() {
                                        if let Some(value_idx) = after_arrow.rfind(clean_value) {
                                            let absolute_start = node_start_offset + arrow_idx + 1 + value_idx;
                                            let absolute_end = absolute_start + clean_value.len();
                                            final_start = rowan::TextSize::from(absolute_start as u32);
                                            final_end = rowan::TextSize::from(absolute_end as u32);
                                        }
                                    } else {
                                        if let Some(quote_idx) = after_arrow.find('"') {
                                            let absolute_start = node_start_offset + arrow_idx + 1 + quote_idx;
                                            let absolute_end = absolute_start + 2;
                                            final_start = rowan::TextSize::from(absolute_start as u32);
                                            final_end = rowan::TextSize::from(absolute_end as u32);
                                        }
                                    }
                                }
                                let full_message = format!("{}\n{}", reason.title, reason.message);
                                diagnostics.push(create_err(text, final_start, final_end, &full_message));
                            }
                            continue;
                        }

                        // 2. MAC без стрелок
                        if name_lower.starts_with("mac") || name_lower == "hw" || name_lower == "hardware" {
                            let mut mac_value = val.clone();
                            
                            if val.contains('>') {
                                let parts: Vec<&str> = val.split('>').map(|s| s.trim()).collect();
                                if parts.len() == 2 {
                                    let right_side: Vec<&str> = parts[1].split_whitespace().collect();
                                    if right_side.len() >= 2 {
                                        mac_value = right_side[1..].join(" ");
                                    }
                                }
                            }

                            let clean_mac = mac_value.trim_matches('"').trim_matches('\'').trim();

                            if let Err(reason) = validators::validate_mac(clean_mac, locale) {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(text, val_range.start(), val_range.end(), &format!("Invalid MAC: {}", reason)));
                            }
                            continue;
                        }

                        // 3. IP и CIDR без стрелок
                        if name_lower.starts_with("ip") || name_lower.starts_with("cidr") || name_lower == "gateway" || name_lower == "subnet" {
                            if val.contains('/') {
                                if let Err(reason) = validators::validate_cidr(&val, locale) {
                                    let val_range = find_value_range(&node, &val);
                                    diagnostics.push(create_err(text, val_range.start(), val_range.end(), &format!("Invalid CIDR: {}", reason)));
                                }
                            } 
                            else if val.contains(':') {
                                if let Err(reason) = validators::validate_ipv6(&val, locale) {
                                    let val_range = find_value_range(&node, &val);
                                    diagnostics.push(create_err(text, val_range.start(), val_range.end(), &format!("Invalid IPv6: {}", reason)));
                                }
                            } 
                            else {
                                if let Err(reason) = validators::validate_ipv4(&val, locale) {
                                    let val_range = find_value_range(&node, &val);
                                    diagnostics.push(create_err(text, val_range.start(), val_range.end(), &format!("Invalid IPv4: {}", reason)));
                                }
                            }
                            continue;
                        }

                        // 4. UUID без стрелок
                        if name_lower.starts_with("uuid") || name_lower.contains("uuid") || name_lower == "id" {
                            if let Err(reason) = validators::validate_uuid(&val, locale) {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(text, val_range.start(), val_range.end(), &format!("Invalid UUID: {}", reason)));
                            }
                            continue;
                        }
                    }

                    if const_fields.contains(&name) {
                        let name_range = field_name.text_range();
                        diagnostics.push(create_err(
                            text,
                            name_range.start(),
                            name_range.end(),
                            &format!("Cannot modify constant field '{}'", name),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}

fn find_value_range(node: &SyntaxNode, target: &str) -> rowan::TextRange {
    for element in node.children_with_tokens() {
        if let Some(token) = element.as_token() {
            if token.kind() == YtlKind::FieldValue && token.text().trim() == target {
                let text = token.text();
                
                if let Some(arrow_idx) = text.find('>') {
                    let after_arrow = &text[arrow_idx + 1..];
                    
                    if let Some(first_ws) = after_arrow.find(char::is_whitespace) {
                        let value_str = &after_arrow[first_ws..].trim();
                        
                        if !value_str.is_empty() {
                            if let Some(offset) = text.find(value_str) {
                                let start = token.text_range().start() + rowan::TextSize::from(offset as u32);
                                let end = start + rowan::TextSize::from(value_str.len() as u32);
                                return rowan::TextRange::new(start, end);
                            }
                        }
                    }
                }
                return token.text_range();
            }
        }
    }
    node.text_range()
}

fn create_err(text: &str, start: rowan::TextSize, end: rowan::TextSize, message: &str) -> Diagnostic {
    let start_pos = offset_to_position(text, start);
    let end_pos = offset_to_position(text, end);

    Diagnostic {
        range: Range {
            start: start_pos,
            end: end_pos,
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: message.to_string(),
        ..Default::default()
    }
}

pub fn offset_to_position(text: &str, offset: rowan::TextSize) -> Position {
    let byte_offset = u32::from(offset) as usize;
    let mut line = 0;
    let mut col = 0;

    for (i, c) in text.char_indices() {
        if i >= byte_offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else if c == '\r' {
        } else {
            col += 1;
        }
    }

    Position { line: line as u32, character: col as u32 }
}