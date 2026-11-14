use tower_lsp_server::ls_types::{CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, WorkspaceEdit, Uri, Command};
use std::collections::HashMap;
use regex::Regex;

pub fn get_code_actions(text: &str, uri: &Uri, position: Position) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let current_line = lines.get(position.line as usize).unwrap_or(&"");
    
    // Ищем наследование
    let re = Regex::new(r"^(\s*)([a-zA-Z_]\w*)\{([a-zA-Z_]\w*)\}\s*:").unwrap();
    if let Some(caps) = re.captures(current_line) {
        let base_class = &caps[3];
        
        // Берем отступ ТЕКУЩЕЙ строки (блока) и добавляем 4 пробела
        let current_indent_str: String = current_line.chars().take_while(|c| c.is_whitespace()).collect();
        let indent = format!("{}    ", current_indent_str);
        
        if let Some(base_fields) = find_fields(&lines, base_class) {
            let mut current_fields = Vec::new();
            let current_indent = indent.len();
            
            // Ищем, что уже есть в блоке
            for line in lines.iter().skip(position.line as usize + 1) {
                let trimmed = line.trim();
                let line_indent = line.len() - line.trim_start().len();
                if !trimmed.is_empty() && line_indent < current_indent { break; }
                if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#") {
                    if let Some(first_word) = trimmed.split_whitespace().next() {
                        if !first_word.contains(':') { current_fields.push(first_word.to_string()); }
                    }
                }
            }
            
            // Оставляем только недостающие поля
            let missing_fields: Vec<(String, String)> = base_fields.into_iter()
                .filter(|(f, _)| !current_fields.contains(f))
                .collect();
            
            if !missing_fields.is_empty() {
                // 1. Опция: Вставить ВСЕ (чистый код, без //)
                let new_text = format!("\n{}", missing_fields.iter()
                    .map(|(name, value)| format!("{}{} {}\n", indent, name, value))
                    .collect::<String>());
                
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Insert all {} inherited fields", missing_fields.len()),
                    kind: Some(CodeActionKind::REFACTOR),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position { line: position.line + 1, character: 0 },
                                    end: Position { line: position.line + 1, character: 0 },
                                },
                                new_text,
                            }],
                        )])),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));

                // 2. Опция: Выбрать конкретные (вызывает команду в JS)
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Select fields to insert...".to_string(),
                    kind: Some(CodeActionKind::REFACTOR),
                    command: Some(Command {
                        title: "Select fields".to_string(),
                        command: "ytl.selectInheritedFields".to_string(),
                        arguments: Some(vec![
                            serde_json::Value::String(uri.to_string()),
                            serde_json::to_value(position.line).unwrap(),
                            serde_json::to_value(&missing_fields).unwrap(),
                            serde_json::Value::String(indent),
                        ]),
                    }),
                    ..Default::default()
                }));
            }
        }
    }
    actions
}

fn find_fields(lines: &[&str], block_name: &str) -> Option<Vec<(String, String)>> {
    let mut all_fields = Vec::new();
    let mut current_block = block_name.to_string();
    let mut visited = Vec::new(); // Защита от бесконечных циклов

    // Максимум 5 уровней вложенности
    for _ in 0..5 {
        if visited.contains(&current_block) { break; }
        visited.push(current_block.clone());

        let target_with_colon = format!("{}:", current_block);
        let target_with_brace = format!("{}{{", current_block);
        
        let mut start_indent = 0;
        let mut in_block = false;
        let mut found_next_parent = None;
        let mut current_fields = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if !in_block {
                if trimmed.starts_with(&target_with_colon) || trimmed.starts_with(&target_with_brace) {
                    in_block = true;
                    start_indent = line.len() - line.trim_start().len();
                    
                    let re_parent = Regex::new(r"\{([a-zA-Z_]\w*)\}").unwrap();
                    if let Some(caps) = re_parent.captures(trimmed) {
                        found_next_parent = Some(caps[1].to_string());
                    }
                }
            } else {
                let indent = line.len() - line.trim_start().len();
                if !trimmed.is_empty() && indent <= start_indent { break; }
                if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 && !parts[0].contains(':') {
                        let name = parts[0].to_string();
                        let value = parts[1].to_string();
                        
                        // ФИЛЬТР: Добавляем, только если такого поля еще нет
                        if !all_fields.iter().any(|(n, _)| n == &name) {
                            current_fields.push((name, value));
                        }
                    }
                }
            }
        }

        all_fields.extend(current_fields);
        
        if let Some(parent) = found_next_parent {
            current_block = parent;
        } else {
            break; 
        }
    }
    
    Some(all_fields)
}