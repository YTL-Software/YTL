use tower_lsp_server::ls_types::Diagnostic;
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<&Token> = Vec::new();

    let mut indent_stack: Vec<usize> = vec![0]; 
    let mut last_line = 0;
    let mut has_seen_colon = false;

    let mut const_fields: Vec<String> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        // Обновление стека отступов
        if token.line > last_line {
            last_line = token.line;
            let current_indent = token.char;
            if current_indent > *indent_stack.last().unwrap() {
                indent_stack.push(current_indent);
            } else if current_indent < *indent_stack.last().unwrap() {
                while current_indent < *indent_stack.last().unwrap() {
                    indent_stack.pop();
                }
            }
        }

        let is_inside_block = indent_stack.len() > 1;

        match token.kind {
            TokenKind::OpenBrace | TokenKind::OpenBracket | TokenKind::OpenParen => {
                stack.push(token);
            }
            TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::CloseParen => {
                if let Some(open_token) = stack.pop() {
                    let is_matched = matches!(
                        (open_token.kind.clone(), token.kind.clone()),
                        (TokenKind::OpenBrace, TokenKind::CloseBrace)
                        | (TokenKind::OpenBracket, TokenKind::CloseBracket)
                        | (TokenKind::OpenParen, TokenKind::CloseParen)
                    );

                    if !is_matched {
                        let (open_sym, _) = get_symbols(&open_token.kind);
                        let (_, close_sym) = get_symbols(&token.kind);
                        diagnostics.push(create_err(
                            token.line,
                            token.char,
                            1,
                            &format!("Mismatched bracket: expected '{}' but found '{}'.", open_sym, close_sym),
                        ));
                    }
                } else {
                    let (_, close_sym) = get_symbols(&token.kind);
                    diagnostics.push(create_err(
                        token.line,
                        token.char,
                        1,
                        &format!("Unmatched closing bracket '{}'.", close_sym),
                    ));
                }
            }
            TokenKind::Colon => {
                has_seen_colon = true;
            }
            TokenKind::Operator => {
            }
            TokenKind::Word => {
                if token.text == "Include" && (is_inside_block || has_seen_colon) {
                    diagnostics.push(create_err(
                        token.line,
                        token.char,
                        token.text.len(),
                        "Include must be at the root level"
                    ));
                }

                if token.text == "Attach" && !is_inside_block {
                    diagnostics.push(create_err(
                        token.line,
                        token.char,
                        token.text.len(),
                        "Attach must be inside a block"
                    ));
                }

                if token.text == "Realm" && !is_inside_block {
                    diagnostics.push(create_err(
                        token.line,
                        token.char,
                        token.text.len(),
                        "Realm must be inside a block (indented)"
                    ));
                }

                if token.text == "Const" {
                    if let Some(next_token) = tokens.get(i + 1) {
                        if next_token.kind == TokenKind::Word {
                            const_fields.push(next_token.text.clone());
                        }
                    }
                }

                if const_fields.contains(&token.text) && token.text != "Const" {
                    if i > 0 {
                        if let Some(prev_token) = tokens.get(i - 1) {
                            let is_patch = prev_token.text == "+" || prev_token.text == "-";
                            let is_declaration = prev_token.text == "Const";
                            
                            if is_patch || !is_declaration {
                                diagnostics.push(create_err(
                                    token.line,
                                    token.char,
                                    token.text.len(),
                                    &format!("Cannot modify constant field '{}'", token.text),
                                ));
                            }
                        }
                    }
                }
            }
            TokenKind::Network => {
                if crate::validators::is_version(&token.text) {} 
                else if token.text.contains('/') {
                    if let Err(reason) = crate::validators::validate_cidr(&token.text) {
                        diagnostics.push(create_err(
                            token.line,
                            token.char,
                            token.text.len(),
                            &format!("Invalid CIDR: {}", reason),
                        ));
                    }
                }
                else if token.text.contains('.') && !token.text.contains(':') && !token.text.ends_with('.') 
                    && !(token.text.len() == 14 && token.text.matches('.').count() == 2 && token.text.chars().all(|c| c.is_ascii_hexdigit() || c == '.')) {
                    if let Err(reason) = crate::validators::validate_ipv4(&token.text) {
                        diagnostics.push(create_err(
                            token.line,
                            token.char,
                            token.text.len(),
                            &format!("Invalid IPv4: {}", reason),
                        ));
                    }
                }
                else if (token.text.contains(':') || token.text.contains('-') || token.text.contains('.')) 
                     && token.text.len() >= 12 && token.text.len() <= 20 
                     && !token.text.contains("::") {
                    if let Err(reason) = crate::validators::validate_mac(&token.text) {
                        diagnostics.push(create_err(
                            token.line,
                            token.char,
                            token.text.len(),
                            &format!("Invalid MAC: {}", reason),
                        ));
                    }
                }
                else if token.text.contains(':') && !token.text.contains('.') {
                    if let Err(reason) = crate::validators::validate_ipv6(&token.text) {
                        diagnostics.push(create_err(
                            token.line,
                            token.char,
                            token.text.len(),
                            &format!("Invalid IPv6: {}", reason),
                        ));
                    }
                }
            }
        }
    }

    for open_token in stack {
        let (open_sym, _) = get_symbols(&open_token.kind);
        diagnostics.push(create_err(
            open_token.line,
            open_token.char,
            1,
            &format!("Unclosed bracket '{}'.", open_sym),
        ));
    }

    diagnostics
}

fn create_err(line: usize, char: usize, length: usize, message: &str) -> Diagnostic {
    Diagnostic {
        range: tower_lsp_server::ls_types::Range {
            start: tower_lsp_server::ls_types::Position { line: line as u32, character: char as u32 },
            end: tower_lsp_server::ls_types::Position { line: line as u32, character: (char + length) as u32 },
        },
        severity: Some(tower_lsp_server::ls_types::DiagnosticSeverity::ERROR),
        message: message.to_string(),
        ..Default::default()
    }
}

fn get_symbols(kind: &TokenKind) -> (String, String) {
    match kind {
        TokenKind::OpenBrace => ("{".to_string(), "}".to_string()),
        TokenKind::OpenBracket => ("[".to_string(), "]".to_string()),
        TokenKind::OpenParen => ("(".to_string(), ")".to_string()),
        TokenKind::CloseBrace => ("}".to_string(), "}".to_string()),
        TokenKind::CloseBracket => ("]".to_string(), "]".to_string()),
        TokenKind::CloseParen => (")".to_string(), ")".to_string()),
        _ => ("?".to_string(), "?".to_string()),        
    }
}