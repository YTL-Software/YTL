#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenBrace, CloseBrace, OpenBracket, CloseBracket,
    OpenParen, CloseParen, Word, Network, Colon, Operator,
    Comma,
    StringLiteral,
    Range,
    Path,
    EnvVar,     // $DB_HOST
    Url,        // https://... или ytl://...
    IpV6,       // [::1]:8080
    Whitespace, // пробелы и табы
    Newline,    // \n или \r\n
    Comment,    // // ...
    Unknown,    // любой другой символ, чтобы не терять байты
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
    let mut chars = text.char_indices().peekable();
    let mut line_idx = 0;
    let mut char_idx = 0;

    while let Some(&(byte_idx, ch)) = chars.peek() {
        let rest = &text[byte_idx..];

        // 1. ПЕРЕНОСЫ СТРОК (\r\n или \n)
        if ch == '\r' || ch == '\n' {
            let start_char = char_idx;
            let mut nl = String::new();
            if ch == '\r' {
                nl.push('\r');
                chars.next();
                if let Some(&(_, '\n')) = chars.peek() {
                    nl.push('\n');
                    chars.next();
                }
            } else {
                nl.push('\n');
                chars.next();
            }
            tokens.push(Token { kind: TokenKind::Newline, text: nl, line: line_idx, char: start_char });
            line_idx += 1;
            char_idx = 0;
        }
        // 2. ПРОБЕЛЫ И ТАБЫ
        else if ch == ' ' || ch == '\t' {
            let start_char = char_idx;
            let mut ws = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c == ' ' || c == '\t' {
                    ws.push(c);
                    chars.next();
                    char_idx += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: TokenKind::Whitespace, text: ws, line: line_idx, char: start_char });
        }
        // 3. КОММЕНТАРИИ (//... и /* ... */ и #...)
        else if rest.starts_with("//") || rest.starts_with("/*") || (ch == '#' && is_hash_comment(rest, char_idx, &text[..byte_idx])) {
            let start_char = char_idx;
            let mut comment = String::new();
            
            if ch == '#' {
                // Комментарий через решетку
                comment.push('#');
                chars.next();
                char_idx += 1;
                while let Some(&(_, c)) = chars.peek() {
                    if c != '\r' && c != '\n' {
                        comment.push(c);
                        chars.next();
                        char_idx += 1;
                    } else { break; }
                }
            } else if rest.starts_with("/*") {
                // Блочный комментарий
                comment.push_str("/*");
                chars.next(); // съедаем /
                chars.next(); // съедаем *
                char_idx += 2;
                
                while let Some(&(_, c)) = chars.peek() {
                    comment.push(c);
                    chars.next();
                    char_idx += 1;
                    if c == '*' && chars.peek().map_or(false, |&(_, next_c)| next_c == '/') {
                        comment.push('/');
                        chars.next();
                        char_idx += 1;
                        break;
                    }
                    if c == '\n' {
                        line_idx += 1;
                        char_idx = 0;
                    }
                }
            } else {
                // Строчный комментарий //
                while let Some(&(_, c)) = chars.peek() {
                    if c != '\r' && c != '\n' {
                        comment.push(c);
                        chars.next();
                        char_idx += 1;
                    } else { break; }
                }
            }
            
            tokens.push(Token { kind: TokenKind::Comment, text: comment, line: line_idx, char: start_char });
        }
        // 4. URL И YTL-ПУТИ (http://, https://, ytl://)
        else if rest.starts_with("http://") || rest.starts_with("https://") || rest.starts_with("ytl://") {
            let start_char = char_idx;
            let mut url = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if !c.is_whitespace() && c != ',' && c != ']' && c != '}' {
                    url.push(c);
                    chars.next();
                    char_idx += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: TokenKind::Url, text: url, line: line_idx, char: start_char });
        }
        // 5. ENV ($VAR)
        else if ch == '$' {
            let start_char = char_idx;
            let mut env = String::new();
            env.push('$');
            chars.next();
            char_idx += 1;
            while let Some(&(_, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == ':' || c == '-' {
                    env.push(c);
                    chars.next();
                    char_idx += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: TokenKind::EnvVar, text: env, line: line_idx, char: start_char });
        }
        // 6. СТРОКИ В КАВЫЧКАХ
        else if ch == '"' {
            let start_char = char_idx;
            let mut string = String::new();
            string.push('"');
            chars.next();
            char_idx += 1;
            
            let mut closed = false;
            while let Some(&(_, c)) = chars.peek() {
                string.push(c);
                chars.next();
                char_idx += 1;
                if c == '"' {
                    closed = true;
                    break;
                }
            }
            
            if closed {
                tokens.push(Token { kind: TokenKind::StringLiteral, text: string, line: line_idx, char: start_char });
            } else {
                // Если кавычка не закрыта, отдаем как Unknown, чтобы не ломать парсер
                tokens.push(Token { kind: TokenKind::Unknown, text: string, line: line_idx, char: start_char });
            }
        }
        // 7. ПУТИ (C:\... и /usr/...)
        else if (ch.is_alphabetic() && rest.split_whitespace().next().unwrap_or("").contains(":\\")) || ch == '/' {
            let start_char = char_idx;
            let mut path = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if !c.is_whitespace() && c != ',' && c != ']' && c != '}' {
                    path.push(c);
                    chars.next();
                    char_idx += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: TokenKind::Path, text: path, line: line_idx, char: start_char });
        }
        // 8. IPv6 ([::1]:8080) ИЛИ МАССИВ ([1.1.1.1, ...])
        else if ch == '[' {
            let rest_of_line = rest.split('\n').next().unwrap_or("");
            let close_bracket_idx = rest_of_line.find(']').unwrap_or(rest_of_line.len());
            let inside_brackets = if close_bracket_idx > 0 { &rest_of_line[1..close_bracket_idx] } else { "" };
            let is_ipv6 = inside_brackets.contains(':') && !inside_brackets.contains(',');

            if is_ipv6 {
                let start_char = char_idx;
                let mut ipv6 = String::new();
                ipv6.push('[');
                chars.next();
                char_idx += 1;
                while let Some(&(_, c)) = chars.peek() {
                    ipv6.push(c);
                    chars.next();
                    char_idx += 1;
                    if c == ']' { break; }
                }
                if let Some(&(_, ':')) = chars.peek() {
                    ipv6.push(':');
                    chars.next();
                    char_idx += 1;
                    while let Some(&(_, c)) = chars.peek() {
                        if c.is_ascii_digit() {
                            ipv6.push(c);
                            chars.next();
                            char_idx += 1;
                        } else { break; }
                    }
                }
                tokens.push(Token { kind: TokenKind::IpV6, text: ipv6, line: line_idx, char: start_char });
            } else {
                tokens.push(Token { kind: TokenKind::OpenBracket, text: "[".to_string(), line: line_idx, char: char_idx });
                chars.next();
                char_idx += 1;
            }
        }
        // 9. СЛОВА (ВКЛЮЧАЯ +ip)
        else if ch.is_alphabetic() || ch == '_' || ch == '@' || (ch == '+' && chars.peek().map_or(false, |&(_, next_c)| next_c.is_alphabetic())) {
            let start_char = char_idx;
            let mut word = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '@' || c == '+' {
                    word.push(c);
                    chars.next();
                    char_idx += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: TokenKind::Word, text: word, line: line_idx, char: start_char });
        }
        // 10. ЧИСЛА, СЕТИ, ДИАПАЗОНЫ
        else if ch.is_ascii_digit() || ch == '~' {
            let start_char = char_idx;
            let mut net = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '/' || c == '-' || c == '~' || c == '_' || c == '%' {
                    net.push(c);
                    chars.next();
                    char_idx += 1;
                } else { break; }
            }
            let kind = if net.contains('~') {
                TokenKind::Range
            } else if net.contains('%') {
                TokenKind::Word
            } else {
                let dot_count = net.matches('.').count();
                let is_network = net.contains(':') || net.contains('/') || dot_count >= 2;
                if is_network { TokenKind::Network } else { TokenKind::Word }
            };
            tokens.push(Token { kind, text: net, line: line_idx, char: start_char });
        }
        // 11. ОСТАЛЬНЫЕ СИМВОЛЫ
        else {
            let kind = match ch {
                '{' => TokenKind::OpenBrace,
                '}' => TokenKind::CloseBrace,
                '[' => TokenKind::OpenBracket,
                ']' => TokenKind::CloseBracket,
                '(' => TokenKind::OpenParen,
                ')' => TokenKind::CloseParen,
                ':' => TokenKind::Colon,
                '+' | '-' | '*' | '/' | '>' => TokenKind::Operator,
                ',' => TokenKind::Comma,
                _ => TokenKind::Unknown,
            };
            tokens.push(Token { kind, text: ch.to_string(), line: line_idx, char: char_idx });
            chars.next();
            char_idx += 1;
        }
    }
    tokens
}

/// Проверяет, является ли `#` началом комментария.
/// Комментарий — если `#` в начале строки или после пробела, 
/// И после него НЕ идёт hex-цвет (3 или 6 hex-символов до конца строки/разделителя).
fn is_hash_comment(rest: &str, char_idx: usize, prefix: &str) -> bool {
    // 1. # должен быть в начале строки или после пробела/таба
    let is_at_start_or_space = char_idx == 0 || prefix.ends_with(' ') || prefix.ends_with('\t');
    if !is_at_start_or_space {
        return false;
    }

    // 2. Проверяем, не hex-цвет ли это
    let after_hash = rest.trim_start_matches('#');
    let hex_part: String = after_hash.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    
    // Если после # идёт 3 или 6 hex-символов, и дальше конец строки/пробел/разделитель — это цвет
    let is_hex_color = (hex_part.len() == 3 || hex_part.len() == 6)
        && (after_hash.len() == hex_part.len()
            || after_hash.chars().nth(hex_part.len()).map_or(true, |c| {
                c.is_whitespace() || c == ',' || c == ']' || c == '}' || c == ')'
            }));

    !is_hex_color
}