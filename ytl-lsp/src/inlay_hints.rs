use ytl_core::syntax::{SyntaxNode, YtlKind};
use tower_lsp_server::ls_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range, Uri};
use crate::cst_validator::offset_to_position;
use crate::symbols::{collect_all_blocks, get_fields_from_block};
use crate::fs_utils::resolve_path;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn get_inlay_hints(
    cst: &SyntaxNode,
    text: &str,
    range: Range,
    uri: &Uri,
    root_path: Option<&Path>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    let current_path = if let Some(path) = uri.to_file_path() {
        path.into_owned()
    } else {
        PathBuf::from(".")
    };
    let file_dir = current_path.parent().unwrap_or(Path::new("."));

    let all_blocks = collect_all_blocks(cst, text, uri, root_path);

    for node in cst.descendants() {
        // 1. ХИНТЫ ДЛЯ НАСЛЕДОВАНИЯ
        if node.kind() == YtlKind::Block {
            let block_start = offset_to_position(text, node.text_range().start());
            if block_start.line < range.start.line || block_start.line > range.end.line {
                continue;
            }

            if let Some(parents_node) = node.children().find(|n| n.kind() == YtlKind::BlockParents) {
                let parent_names = crate::symbols::get_parents(&parents_node);
                if parent_names.is_empty() { continue; }

                let mut inherited_fields = Vec::new();
                let mut seen_fields = HashSet::new();

                for parent_name in &parent_names {
                    if let Some(parent_block) = all_blocks.get(parent_name) {
                        for (name, _) in get_fields_from_block(parent_block) {
                            if seen_fields.insert(name.clone()) {
                                inherited_fields.push(name);
                            }
                        }
                    }
                }

                if inherited_fields.is_empty() { continue; }

                let max_fields = 4;
                let total = inherited_fields.len();
                
                let hint_text = if total > max_fields {
                    let visible = &inherited_fields[0..max_fields];
                    let remaining = total - max_fields;
                    format!("[{}, ... +{}]", visible.join(", "), remaining)
                } else {
                    format!("[{}]", inherited_fields.join(", "))
                };

                let block_line = block_start.line;
                let line_text = lines.get(block_line as usize).unwrap_or(&"");
                let end_char = line_text.len() as u32;

                hints.push(InlayHint {
                    position: Position { line: block_line, character: end_char },
                    label: InlayHintLabel::String(hint_text),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                });
            }
        }

        // 2. ХИНТЫ ДЛЯ ДИФФА И INCLUDE/ATTACH
        if node.kind() == YtlKind::Field {
            let field_start = offset_to_position(text, node.text_range().start());
            if field_start.line < range.start.line || field_start.line > range.end.line {
                continue;
            }

            let name = node.children_with_tokens()
                .filter_map(|el| el.into_token())
                .find(|t| t.kind() == YtlKind::FieldName)
                .map(|t| t.text().trim().to_string())
                .unwrap_or_default();

            if name == "Include" || name == "Attach" {
                let values: Vec<String> = node
                    .children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .filter(|t| t.kind() == YtlKind::FieldValue)
                    .map(|t| t.text().trim().to_string())
                    .collect();

                if let Some(val) = values.first() {
                    if val.starts_with("ytl://") {
                        let rel_path = val.trim_start_matches("ytl://");
                        let target_path = resolve_path(rel_path, file_dir, root_path);

                        if let Some((_content, syntax_node)) = crate::fs_utils::read_ytl_file(&target_path) {
                            let mut items = Vec::new();
                            if name == "Include" {
                                for child in syntax_node.descendants() {
                                    if child.kind() == YtlKind::Block {
                                        if let Some(b_name) = child.children_with_tokens()
                                            .filter_map(|el| el.into_token())
                                            .find(|t| t.kind() == YtlKind::BlockName)
                                            .map(|t| t.text().trim().to_string())
                                        {
                                            items.push(b_name);
                                        }
                                    }
                                }
                            } else if name == "Attach" {
                                for child in syntax_node.descendants() {
                                    if child.kind() == YtlKind::Field {
                                        if let Some(f_name) = child.children_with_tokens()
                                            .filter_map(|el| el.into_token())
                                            .find(|t| t.kind() == YtlKind::FieldName)
                                            .map(|t| t.text().trim().to_string())
                                        {
                                            items.push(f_name);
                                        }
                                    }
                                }
                            }

                            if !items.is_empty() {
                                let max_items = 5;
                                let total_items = items.len();
                                let display_items: Vec<String> = if total_items > max_items {
                                    let mut visible = items[0..max_items].to_vec();
                                    let remaining = total_items - max_items;
                                    visible.push(format!("... +{}", remaining));
                                    visible
                                } else {
                                    items
                                };

                                let hint_text = format!(": [{}]", display_items.join(", "));
                                let line_text = lines.get(field_start.line as usize).unwrap_or(&"");
                                let end_char = line_text.len() as u32;

                                hints.push(InlayHint {
                                    position: Position { line: field_start.line, character: end_char },
                                    label: InlayHintLabel::String(hint_text),
                                    kind: Some(InlayHintKind::TYPE),
                                    text_edits: None,
                                    tooltip: None,
                                    padding_left: Some(true),
                                    padding_right: Some(false),
                                    data: None,
                                });
                            }
                        }
                    }
                }
                continue;
            }

            // 3. ОСТАЛЬНАЯ ЛОГИКА ДИФФА (для обычных полей)
            if name.starts_with('+') {
                continue;
            }

            let target_prefix = format!("+{}", name);
            let mut next_val = None;

            for i in (field_start.line as usize + 1)..lines.len() {
                let next_line = lines[i].trim();
                if next_line.starts_with(&target_prefix) {
                    let val_part = next_line.trim_start_matches(&target_prefix).trim();
                    if !val_part.is_empty() {
                        next_val = Some(val_part.to_string());
                    }
                    break;
                }
                if !next_line.is_empty() && !next_line.starts_with('+') && !next_line.starts_with("//") {
                    break;
                }
            }

            if let Some(new_val) = next_val {
                let clean_val = if new_val.starts_with('[') && new_val.ends_with(']') {
                    &new_val[1..new_val.len()-1]
                } else {
                    &new_val
                };
                let elements: Vec<&str> = clean_val.split(',').map(|s| s.trim()).collect();
                
                let hint_text = if elements.len() > 3 {
                    let visible = &elements[0..3];
                    let remaining = elements.len() - 3;
                    format!(" :[+{}, ... +{}]", visible.join(", "), remaining)
                } else {
                    format!(" :[+{}]", clean_val)
                };

                let line_text = lines.get(field_start.line as usize).unwrap_or(&"");
                let end_char = line_text.len() as u32;
                
                hints.push(InlayHint {
                    position: Position { line: field_start.line, character: end_char },
                    label: InlayHintLabel::String(hint_text),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                });
            }
        }
    }

    hints
}