use crate::syntax::{SyntaxNode, YtlKind};
use crate::validators;
use crate::fs_utils::resolve_path;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Position, Range, Uri};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn validate(
    cst: &SyntaxNode,
    text: &str,
    uri: &Uri,
    root_path: Option<&Path>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut const_fields: Vec<String> = Vec::new();
    let mut seen_block = false;
    let mut block_names: HashMap<String, rowan::TextRange> = HashMap::new();

    for node in cst.descendants() {
        match node.kind() {
            YtlKind::Block => {
                seen_block = true;
                if let Some(name_token) = node.children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .find(|t| t.kind() == YtlKind::BlockName)
                {
                    let name = name_token.text().trim().to_string();
                    if let Some(_prev_range) = block_names.get(&name) {
                        diagnostics.push(create_err(
                            text,
                            name_token.text_range().start(),
                            name_token.text_range().end(),
                            &format!("Duplicate block name '{}'.", name),
                        ));
                    } else {
                        block_names.insert(name, name_token.text_range());
                    }
                }
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
                                    // Собираем поля из Attach
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

                                    // Проверяем сиблинги в текущем блоке
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

                    // === УМНАЯ ВАЛИДАЦИЯ ДАННЫХ (ТОЛЬКО ПО ЗНАЧЕНИЮ) ===
                    for val in values {
                        if val.starts_with("ytl://") || val.starts_with("http://") || val.starts_with("https://") {
                            continue;
                        }

                        let val_lower = val.to_lowercase();
                        
                        let parts_colon: Vec<&str> = val_lower.split(':').collect();
                        let parts_dash: Vec<&str> = val_lower.split('-').collect();
                        let parts_dot: Vec<&str> = val_lower.split('.').collect();

                        let is_cidr_like = val_lower.contains('/') && (val_lower.contains('.') || val_lower.contains(':'));
                        
                        let is_mac_like = (parts_colon.len() == 6 && parts_colon.iter().all(|p| p.len() == 2))
                            || (parts_dash.len() == 6 && parts_dash.iter().all(|p| p.len() == 2))
                            || (parts_dot.len() == 3 && parts_dot.iter().all(|p| p.len() == 4 && p.chars().all(|c| c.is_ascii_hexdigit())));

                        let is_cisco_mac_like = val_lower.contains('.') 
                            && val_lower.chars().all(|c| c.is_ascii_hexdigit() || c == '.')
                            && !val_lower.chars().all(|c| c.is_ascii_digit() || c == '.');

                        let is_ipv6_like = !is_mac_like 
                            && (val_lower.contains("::") || val_lower.matches(':').count() == 7)
                            && !val_lower.contains('.')
                            && !val_lower.contains("://");

                        let is_ipv4_like = val_lower.matches('.').count() == 3 
                            && !val_lower.contains('/') 
                            && !val_lower.contains(':')
                            && !val_lower.chars().any(|c| c.is_alphabetic());

                        let is_uuid_like = val_lower.len() >= 30 && val_lower.len() <= 40 && val_lower.matches('-').count() == 4;

                        if is_uuid_like {
                            if let Err(reason) = validators::validate_uuid(&val) {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(
                                    text,
                                    val_range.start(),
                                    val_range.end(),
                                    &format!("Invalid UUID: {}", reason),
                                ));
                            }
                        }
                        else if is_cidr_like {
                            if let Err(reason) = validators::validate_cidr(&val) {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(
                                    text,
                                    val_range.start(),
                                    val_range.end(),
                                    &format!("Invalid CIDR: {}", reason),
                                ));
                            }
                        }
                        else if is_mac_like || is_cisco_mac_like || (val_lower.contains(':') && !is_ipv6_like && !is_uuid_like) {
                            if let Err(reason) = validators::validate_mac(&val) {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(
                                    text,
                                    val_range.start(),
                                    val_range.end(),
                                    &format!("Invalid MAC: {}", reason),
                                ));
                            }
                        }
                        else if is_ipv6_like {
                            if let Err(reason) = validators::validate_ipv6(&val) {
                                let val_range = find_value_range(&node, &val);
                                diagnostics.push(create_err(
                                    text,
                                    val_range.start(),
                                    val_range.end(),
                                    &format!("Invalid IPv6: {}", reason),
                                ));
                            }
                        }
                        else if is_ipv4_like {
                            if !validators::is_version(&val) {
                                if let Err(reason) = validators::validate_ipv4(&val) {
                                    let val_range = find_value_range(&node, &val);
                                    diagnostics.push(create_err(
                                        text,
                                        val_range.start(),
                                        val_range.end(),
                                        &format!("Invalid IPv4: {}", reason),
                                    ));
                                }
                            }
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