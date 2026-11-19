use ytl_core::syntax::{SyntaxNode, YtlKind};
use tower_lsp_server::ls_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Position, Range, TextEdit,
    WorkspaceEdit, Uri,
};
use std::collections::HashMap;

pub fn get_code_actions(
    cst: &SyntaxNode,
    text: &str,
    uri: &Uri,
    position: Position,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    for node in cst.descendants() {
        if node.kind() == YtlKind::Block {
            let range = node.text_range();
            let start = crate::cst_validator::offset_to_position(text, range.start());

            let is_on_line = position.line == start.line;

            if is_on_line {
                if let Some(parents_node) = node.children().find(|n| n.kind() == YtlKind::BlockParents) {
                    let parent_names = get_parents(&parents_node);
                    if parent_names.is_empty() { continue; }

                    let mut existing_fields: Vec<String> = Vec::new();
                    for child in node.children() {
                        if child.kind() == YtlKind::Field {
                            if let Some(name) = child.children_with_tokens()
                                .filter_map(|el| el.into_token())
                                .find(|t| t.kind() == YtlKind::FieldName)
                                .map(|t| t.text().trim().to_string()) {
                                existing_fields.push(name);
                            }
                        }
                    }

                    let mut parents_with_fields: Vec<(String, Vec<(String, String)>)> = Vec::new();
                    for parent_name in &parent_names {
                        if let Some(parent_block) = find_block_by_name(cst, parent_name) {
                            let fields = get_fields_from_block(&parent_block);
                            let missing: Vec<(String, String)> = fields.into_iter()
                                .filter(|(name, _)| !existing_fields.contains(name))
                                .collect();
                            
                            if !missing.is_empty() {
                                parents_with_fields.push((parent_name.clone(), missing));
                            }
                        }
                    }

                    if parents_with_fields.is_empty() { continue; }

                    actions.extend(create_actions(
                        &parents_with_fields,
                        text,
                        uri,
                        start.line,
                        &format!("Insert all {} inherited fields", count_total(&parents_with_fields)),
                    ));
                }
            }
        }

        if node.kind() == YtlKind::Realm {
            let range = node.text_range();
            let start = crate::cst_validator::offset_to_position(text, range.start());

            if position.line == start.line {
                let mut parent_block = None;
                let mut current = node.parent();
                while let Some(p) = current {
                    if p.kind() == YtlKind::Block {
                        parent_block = Some(p);
                        break;
                    }
                    current = p.parent();
                }

                if let Some(parent) = parent_block {
                    let parent_fields = get_fields_from_block(&parent); 
                    
                    let realm_range = node.text_range();
                    let mut existing_fields: Vec<String> = Vec::new();
                    
                    for child in parent.descendants() {
                        if child.kind() == YtlKind::Field {
                            let field_range = child.text_range();
                            if field_range.start() > realm_range.end() && field_range.end() <= parent.text_range().end() {
                                let mut inside_other_realm = false;
                                for other in parent.descendants() {
                                    if other.kind() == YtlKind::Realm && other != node {
                                        let other_range = other.text_range();
                                        if field_range.start() > other_range.start() && field_range.end() < other_range.end() {
                                            inside_other_realm = true;
                                            break;
                                        }
                                    }
                                }
                                if !inside_other_realm {
                                    if let Some(name) = child.children_with_tokens()
                                        .filter_map(|el| el.into_token())
                                        .find(|t| t.kind() == YtlKind::FieldName)
                                        .map(|t| t.text().trim().to_string()) {
                                        existing_fields.push(name);
                                    }
                                }
                            }
                        }
                    }

                    let missing: Vec<(String, String)> = parent_fields.into_iter()
                        .filter(|(name, _)| !existing_fields.contains(name))
                        .collect();

                    if !missing.is_empty() {
                        let parents_with_fields = vec![("parent".to_string(), missing)];
                        actions.extend(create_actions(
                            &parents_with_fields,
                            text,
                            uri,
                            start.line,
                            &format!("Insert all {} fields from parent", count_total(&parents_with_fields)),
                        ));
                    }
                }
            }
        }
    }

    actions
}

fn create_actions(
    parents_with_fields: &[(String, Vec<(String, String)>)],
    text: &str,
    uri: &Uri,
    line: u32,
    title_all: &str,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    let current_indent = text.lines().nth(line as usize)
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).collect::<String>())
        .unwrap_or_default();
    let indent = format!("{}    ", current_indent);

    let mut all_fields_text = String::new();
    for (_, fields) in parents_with_fields {
        for (name, val) in fields {
            all_fields_text.push_str(&format!("{}{} {}\n", indent, name, val));
        }
    }

    let insert_pos = Position { line: line + 1, character: 0 };

    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
        title: title_all.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: Range { start: insert_pos, end: insert_pos },
                    new_text: format!("\n{}", all_fields_text),
                }],
            )])),
            ..Default::default()
        }),
        ..Default::default()
    }));

    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
        title: "Select fields to insert...".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        command: Some(Command {
            title: "Select fields".to_string(),
            command: "ytl.selectInheritedFields".to_string(),
            arguments: Some(vec![
                serde_json::json!(uri.to_string()),
                serde_json::json!(line),
                serde_json::json!(parents_with_fields),
                serde_json::json!(indent),
            ]),
        }),
        ..Default::default()
    }));

    actions
}

fn count_total(parents: &[(String, Vec<(String, String)>)]) -> usize {
    parents.iter().map(|(_, f)| f.len()).sum()
}

fn get_parents(parents_node: &SyntaxNode) -> Vec<String> {
    parents_node
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .filter(|t| t.kind() == YtlKind::BlockName)
        .map(|t| t.text().trim().to_string())
        .collect()
}

fn find_block_by_name<'a>(cst: &'a SyntaxNode, name: &str) -> Option<SyntaxNode> {
    let target = name.trim();
    let mut last_found = None;
    
    for node in cst.descendants() {
        if node.kind() == YtlKind::Block {
            let block_name = node.children_with_tokens()
                .filter_map(|el| el.into_token())
                .find(|t| t.kind() == YtlKind::BlockName)
                .map(|t| t.text().trim().to_string())
                .unwrap_or_default();
            
            if block_name == target {
                last_found = Some(node);
            }
        }
    }
    last_found
}

fn get_fields_from_block(block: &SyntaxNode) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut first_realm_start = None;
    for child in block.children() {
        if child.kind() == YtlKind::Realm {
            first_realm_start = Some(child.text_range().start());
            break;
        }
    }
    for child in block.children() {
        if child.kind() == YtlKind::Field {
            let field_start = child.text_range().start();
            if let Some(realm_pos) = first_realm_start {
                if field_start > realm_pos {
                    continue;
                }
            }
            let name = child
                .children_with_tokens()
                .filter_map(|el| el.into_token())
                .find(|t| t.kind() == YtlKind::FieldName)
                .map(|t| t.text().trim().to_string())
                .unwrap_or_default();
            
            let val = child
                .children_with_tokens()
                .filter_map(|el| el.into_token())
                .filter(|t| t.kind() == YtlKind::FieldValue)
                .map(|t| t.text().trim().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            
            if !name.is_empty() && name != "Const" {
                fields.push((name, val));
            }
        }
    }
    fields
}