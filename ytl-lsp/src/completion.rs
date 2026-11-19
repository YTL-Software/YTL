use ytl_core::syntax::{SyntaxNode, YtlKind};
use tower_lsp_server::ls_types::{
    Command, CompletionItem, CompletionItemKind, CompletionTextEdit, Position, Range, TextEdit, Uri,
};
use crate::cst_validator::offset_to_position;
use crate::symbols::{collect_all_blocks, get_fields_from_block};
use crate::fs_utils::resolve_path;
use std::path::{Path, PathBuf};
use std::fs;

pub fn get_completions(
    cst: &SyntaxNode,
    position: Position,
    text: &str,
    uri: &Uri,
    root_path: Option<&Path>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    
    let current_line = text.lines().nth(position.line as usize).unwrap_or("");
    let char_pos = position.character as usize;
    let trimmed_line = current_line.trim();
    
    // === ПРОВЕРКА НА INCLUDE/ATTACH (АВТОДОПОЛНЕНИЕ ПУТЕЙ) ===
    if trimmed_line.starts_with("Include") || trimmed_line.starts_with("Attach") {
        if let Some(path_start) = current_line.find("ytl://") {
            let end_idx = std::cmp::min(char_pos, current_line.len());
            if path_start + 6 <= end_idx {
                let path_prefix = &current_line[path_start + 6..end_idx];
                
                let clean_prefix = path_prefix.replace("//", "/").replace("\\", "/");

                let file_dir = if let Some(path) = uri.to_file_path() {
                    path.parent().unwrap_or(Path::new(".")).to_path_buf()
                } else {
                    PathBuf::from(".")
                };

                let (dir_part, file_prefix) = if let Some(last_slash) = clean_prefix.rfind('/') {
                    (&clean_prefix[..last_slash], &clean_prefix[last_slash + 1..])
                } else {
                    ("", clean_prefix.as_str())
                };

                // 1. Если пользователь ничего не ввел после ytl://, предлагаем диски (на Windows)
                if dir_part.is_empty() && file_prefix.is_empty() {
                    if cfg!(windows) {
                        for drive in &["C:/", "D:/", "E:/", "F:/"] {
                            let start_pos = Position { line: position.line, character: (path_start + 6) as u32 };
                            let end_pos = Position { line: position.line, character: end_idx as u32 };
                            let edit = TextEdit {
                                range: Range { start: start_pos, end: end_pos },
                                new_text: drive.to_string(),
                            };
                            items.push(CompletionItem {
                                label: drive.to_string(),
                                kind: Some(CompletionItemKind::FOLDER),
                                text_edit: Some(CompletionTextEdit::Edit(edit)),
                                command: Some(Command {
                                    title: "Suggest".to_string(),
                                    command: "editor.action.triggerSuggest".to_string(),
                                    arguments: None,
                                }),
                                ..Default::default()
                            });
                        }
                    }
                }

                // 2. Определяем целевую директорию
                let target_dir = resolve_path(dir_part, &file_dir, root_path);

                if let Ok(entries) = fs::read_dir(&target_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();

                        if !name.to_lowercase().starts_with(&file_prefix.to_lowercase()) {
                            continue;
                        }

                        let full_path = if dir_part.is_empty() {
                            name.clone()
                        } else {
                            format!("{}/{}", dir_part.trim_end_matches('/'), name)
                        };

                        let start_pos = Position { line: position.line, character: (path_start + 6) as u32 };
                        let end_pos = Position { line: position.line, character: end_idx as u32 };

                        let new_text = if path.is_dir() {
                            format!("{}/", full_path)
                        } else {
                            full_path.clone()
                        };

                        let edit = TextEdit {
                            range: Range { start: start_pos, end: end_pos },
                            new_text,
                        };

                        let trigger_command = Some(Command {
                            title: "Suggest".to_string(),
                            command: "editor.action.triggerSuggest".to_string(),
                            arguments: None,
                        });

                        if path.is_dir() {
                            items.push(CompletionItem {
                                label: format!("{}/", full_path),
                                kind: Some(CompletionItemKind::FOLDER),
                                text_edit: Some(CompletionTextEdit::Edit(edit)),
                                command: trigger_command.clone(),
                                ..Default::default()
                            });
                        } else if path.extension().map_or(false, |ext| ext == "ytl") {
                            items.push(CompletionItem {
                                label: full_path,
                                kind: Some(CompletionItemKind::FILE),
                                text_edit: Some(CompletionTextEdit::Edit(edit)),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
        return items; 
    }

    // 1. Проверка на { ... }
    let open_brace = current_line.find('{');
    let close_brace = current_line.rfind('}');
    
    let mut in_parents = false;
    if let Some(open_idx) = open_brace {
        if char_pos > open_idx && close_brace.map_or(true, |close_idx| char_pos <= close_idx) {
            in_parents = true;
        }
    }

    if in_parents {
        let mut current_block_name = String::new();
        for node in cst.descendants() {
            if node.kind() == YtlKind::Block {
                let start = offset_to_position(text, node.text_range().start());
                if start.line == position.line {
                    if let Some(name_token) = node.children_with_tokens()
                        .filter_map(|el| el.into_token())
                        .find(|t| t.kind() == YtlKind::BlockName)
                    {
                        current_block_name = name_token.text().trim().to_string();
                        break;
                    }
                }
            }
        }
        
        let mut existing_parents = Vec::new();
        if let (Some(open), Some(close)) = (open_brace, close_brace) {
            if open + 1 < close {
                let inside = &current_line[open + 1..close];
                for part in inside.split(',') {
                    let name = part.trim();
                    if !name.is_empty() {
                        existing_parents.push(name.to_string());
                    }
                }
            }
        }

        let all_blocks = collect_all_blocks(cst, text, uri, root_path);

        for (name, _) in &all_blocks {
            if !name.is_empty() && name != &current_block_name && !existing_parents.contains(name) {
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    detail: Some("Block".to_string()),
                    ..Default::default()
                });
            }
        }
        return items; 
    }

    // 2. Логика для полей внутри блока
    let mut current_block = None;
    for node in cst.descendants() {
        if node.kind() == YtlKind::Block {
            let range = node.text_range();
            let start = offset_to_position(text, range.start());
            let end = offset_to_position(text, range.end());
            
            let block_indent = text.lines().nth(start.line as usize)
                .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                .unwrap_or(0);
            let current_indent = current_line.chars().take_while(|c| c.is_whitespace()).count();

            if position.line >= start.line && position.line <= end.line {
                if position.line == start.line || current_indent > block_indent {
                    current_block = Some(node);
                    break;
                }
            }
        }
    }

    if let Some(block) = current_block {
        let mut existing_fields: Vec<String> = Vec::new();
        for child in block.children() {
            if child.kind() == YtlKind::Field {
                if let Some(name) = child.children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .find(|t| t.kind() == YtlKind::FieldName)
                    .map(|t| t.text().trim().to_string()) {
                    existing_fields.push(name);
                }
            }
        }

        let mut field_names = Vec::new();
        
        // 1. ПОЛЯ ОТ РОДИТЕЛЕЙ
        if let Some(parents_node) = block.children().find(|n| n.kind() == YtlKind::BlockParents) {
            let parent_names = crate::symbols::get_parents(&parents_node);
            let all_blocks = collect_all_blocks(cst, text, uri, root_path);

            for parent_name in parent_names {
                if let Some(parent_block) = all_blocks.get(&parent_name) {
                    let fields = get_fields_from_block(parent_block);
                    for (name, _) in fields {
                        if !existing_fields.contains(&name) && !field_names.contains(&name) {
                            field_names.push(name);
                        }
                    }
                }
            }
        }

        // 2. ПОЛЯ ОТ ATTACH
        let file_dir = if let Some(path) = uri.to_file_path() {
            path.parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            PathBuf::from(".")
        };

        for child in block.children() {
            if child.kind() == YtlKind::Field {
                let field_name = child.children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .find(|t| t.kind() == YtlKind::FieldName)
                    .map(|t| t.text().trim().to_string())
                    .unwrap_or_default();

                if field_name == "Attach" {
                    let values: Vec<String> = child.children_with_tokens()
                        .filter_map(|el| el.into_token())
                        .filter(|t| t.kind() == YtlKind::FieldValue)
                        .map(|t| t.text().trim().to_string())
                        .collect();

                    if let Some(val) = values.first() {
                        if val.starts_with("ytl://") {
                            let rel_path = val.trim_start_matches("ytl://");
                            let target_path = resolve_path(rel_path, &file_dir, root_path);

                            if let Some((_content, attached_cst)) = crate::fs_utils::read_ytl_file(&target_path) {
                                for att_node in attached_cst.descendants() {
                                    if att_node.kind() == YtlKind::Field {
                                        if let Some(att_name) = att_node.children_with_tokens()
                                            .filter_map(|el| el.into_token())
                                            .find(|t| t.kind() == YtlKind::FieldName)
                                            .map(|t| t.text().trim().to_string()) 
                                        {
                                            if !existing_fields.contains(&att_name) && !field_names.contains(&att_name) {
                                                field_names.push(att_name);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for name in field_names {
            items.push(CompletionItem {
                label: name,
                kind: Some(CompletionItemKind::FIELD),
                ..Default::default()
            });
        }
    }

    // 3. Ключевые слова
    let words: Vec<&str> = trimmed_line.split_whitespace().collect();
    let is_typing_value_or_name = !words.is_empty() && !trimmed_line.ends_with(':') && !trimmed_line.is_empty();

    if !is_typing_value_or_name || trimmed_line.is_empty() {
        items.push(CompletionItem { label: "Include".to_string(), kind: Some(CompletionItemKind::KEYWORD), ..Default::default() });
        items.push(CompletionItem { label: "Attach".to_string(), kind: Some(CompletionItemKind::KEYWORD), ..Default::default() });
        items.push(CompletionItem { label: "Const".to_string(), kind: Some(CompletionItemKind::KEYWORD), ..Default::default() });
        items.push(CompletionItem { label: "Realm".to_string(), kind: Some(CompletionItemKind::KEYWORD), ..Default::default() });
    }

    items
}