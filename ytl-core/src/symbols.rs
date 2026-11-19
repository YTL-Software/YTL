use crate::syntax::{SyntaxNode, YtlKind};
use crate::fs_utils::resolve_path;
use tower_lsp_server::ls_types::Uri;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub fn get_parents(parents_node: &SyntaxNode) -> Vec<String> {
    parents_node
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .filter(|t| t.kind() == YtlKind::BlockName)
        .map(|t| t.text().trim().to_string())
        .collect()
}

/// Собирает блоки из текущего файла и всех Include (рекурсивно)
pub fn collect_all_blocks(
    cst: &SyntaxNode,
    _text: &str,
    uri: &Uri,
    root_path: Option<&Path>,
) -> HashMap<String, SyntaxNode> {
    let mut blocks = HashMap::new();
    let file_dir = if let Some(path) = uri.to_file_path() {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        PathBuf::from(".")
    };

    // 1. Блоки из текущего файла
    for node in cst.descendants() {
        if node.kind() == YtlKind::Block {
            if let Some(name) = node.children_with_tokens()
                .filter_map(|el| el.into_token())
                .find(|t| t.kind() == YtlKind::BlockName)
                .map(|t| t.text().trim().to_string()) 
            {
                blocks.insert(name, node.clone());
            }
        }
    }

    // 2. Блоки из Include файлов
    for node in cst.descendants() {
        if node.kind() == YtlKind::Field {
            let name = node.children_with_tokens()
                .filter_map(|el| el.into_token())
                .find(|t| t.kind() == YtlKind::FieldName)
                .map(|t| t.text().trim().to_string())
                .unwrap_or_default();

            if name == "Include" {
                let values: Vec<String> = node.children_with_tokens()
                    .filter_map(|el| el.into_token())
                    .filter(|t| t.kind() == YtlKind::FieldValue)
                    .map(|t| t.text().trim().to_string())
                    .collect();

                if let Some(val) = values.first() {
                    if val.starts_with("ytl://") {
                        let rel_path = val.trim_start_matches("ytl://");
                        let target_path = resolve_path(rel_path, &file_dir, root_path);

                        if let Some((included_text, included_cst)) = crate::fs_utils::read_ytl_file(&target_path) {
                            // Рекурсивно собираем блоки из включенного файла
                            let included_blocks = collect_all_blocks(&included_cst, &included_text, uri, root_path);
                            for (name, node) in included_blocks {
                                blocks.entry(name).or_insert(node);
                            }
                        }
                    }
                }
            }
        }
    }

    blocks
}

/// Собирает поля из блока (игнорируя Realm)
pub fn get_fields_from_block(block: &SyntaxNode) -> Vec<(String, String)> {
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
                if field_start > realm_pos { continue; }
            }
            let name = child.children_with_tokens()
                .filter_map(|el| el.into_token())
                .find(|t| t.kind() == YtlKind::FieldName)
                .map(|t| t.text().trim().to_string())
                .unwrap_or_default();
            let val = child.children_with_tokens()
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