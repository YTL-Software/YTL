#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenBrace, CloseBrace, OpenBracket, CloseBracket,
    OpenParen, CloseParen, Word, Network, Colon, Operator,    
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub char: usize,
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (line_idx, line) in text.lines().enumerate() {
        let mut chars = line.chars().peekable();
        let mut char_idx = 0;

        while let Some(&ch) = chars.peek() {
            if ch == '/' {
                let mut temp_chars = chars.clone();
                temp_chars.next();
                if let Some(&next_ch) = temp_chars.peek() {
                    if next_ch == '/' {
                        break;
                    }
                }
            }

            if ch.is_alphabetic() || ch == '_' {
                let start_char = char_idx;
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        word.push(c);
                        chars.next();
                        char_idx += 1;
                    } else { break; }
                }
                tokens.push(Token { kind: TokenKind::Word, text: word, line: line_idx, char: start_char });
            } else if ch.is_ascii_digit() {
                let start_char = char_idx;
                let mut net = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '/' || c == '-' {
                        net.push(c);
                        chars.next();
                        char_idx += 1;
                    } else { break; }
                }
                let dot_count = net.matches('.').count();
                let is_network = net.contains(':') || net.contains('/') || dot_count >= 2;
                tokens.push(Token { 
                    kind: if is_network { TokenKind::Network } else { TokenKind::Word },
                    text: net, line: line_idx, char: start_char 
                });
            } else {
                let kind = match ch {
                    '{' => Some(TokenKind::OpenBrace),
                    '}' => Some(TokenKind::CloseBrace),
                    '[' => Some(TokenKind::OpenBracket),
                    ']' => Some(TokenKind::CloseBracket),
                    '(' => Some(TokenKind::OpenParen),
                    ')' => Some(TokenKind::CloseParen),
                    ':' => Some(TokenKind::Colon),
                    '+' | '-' | '*' | '/' => Some(TokenKind::Operator),
                    _ => None,
                };
                if let Some(k) = kind {
                    tokens.push(Token { kind: k, text: ch.to_string(), line: line_idx, char: char_idx });
                }
                chars.next();
                char_idx += 1;
            }
        }
    }
    tokens
}