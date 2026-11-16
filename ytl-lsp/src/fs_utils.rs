use std::path::{Path, PathBuf};
use std::fs;
use crate::syntax::SyntaxNode;

pub fn resolve_path(
    path_str: &str,
    file_dir: &Path,
    root_path: Option<&Path>,
) -> PathBuf {
    let clean_path = path_str.replace("//", "/").replace("\\", "/");
    let is_absolute = clean_path.starts_with('/') 
        || (cfg!(windows) && clean_path.len() >= 2 && clean_path.chars().nth(1) == Some(':'));

    let mut target_path = if is_absolute {
        PathBuf::from(&clean_path)
    } else if let Some(root) = root_path {
        if clean_path.starts_with("./") || clean_path.starts_with("../") {
            file_dir.join(&clean_path)
        } else {
            root.join(&clean_path)
        }
    } else {
        file_dir.join(&clean_path)
    };

    // Фикс для Windows (если путь это просто "C:")
    if cfg!(windows) && clean_path.len() == 2 && clean_path.ends_with(':') {
        target_path.push("\\");
    }

    target_path
}

pub fn read_ytl_file(path: &Path) -> Option<(String, SyntaxNode)> {
    if let Ok(content) = fs::read_to_string(path) {
        let tokens = crate::lexer::tokenize(&content);
        let cst = crate::cst_parser::parse_to_cst(&tokens);
        let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);
        Some((content, syntax_node))
    } else {
        None
    }
}