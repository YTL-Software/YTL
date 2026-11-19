use rowan::{GreenNode, GreenNodeBuilder, Language};
use crate::syntax::{YtlKind, YtlLanguage};
use crate::lexer::{Token, TokenKind};

fn is_trivia(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::Whitespace | TokenKind::Newline | TokenKind::Comment)
}

fn next_meaningful_on_line(tokens: &[Token], mut idx: usize, line: usize) -> Option<usize> {
    while idx < tokens.len() && tokens[idx].line == line {
        if !is_trivia(&tokens[idx].kind) {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

pub fn parse_to_cst(tokens: &[Token]) -> GreenNode {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(YtlLanguage::kind_to_raw(YtlKind::Root));

    let mut block_stack: Vec<usize> = Vec::new();
    let mut i = 0;
    let mut last_line = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if is_trivia(&token.kind) {
            builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &token.text);
            i += 1;
            continue;
        }

        let is_stray_include =
            token.kind == TokenKind::Word && token.text == "Include" && !block_stack.is_empty();

        if !is_stray_include
            && token.line > last_line
            && !block_stack.is_empty()
            && token.char <= *block_stack.last().unwrap()
        {
            while !block_stack.is_empty() && token.char <= *block_stack.last().unwrap() {
                builder.finish_node();
                block_stack.pop();
            }
        }
        last_line = token.line;

        match token.kind {
            TokenKind::Word => {
                if let Some(next_idx) = next_meaningful_on_line(tokens, i + 1, token.line) {
                    let next = &tokens[next_idx];

                    if next.kind == TokenKind::OpenBrace {
                        let mut j = next_idx + 1;
                        while j < tokens.len() && tokens[j].kind != TokenKind::CloseBrace {
                            j += 1;
                        }
                        let after_brace = if j < tokens.len() {
                            next_meaningful_on_line(tokens, j + 1, token.line)
                        } else {
                            None
                        };

                        if let Some(colon_idx) = after_brace {
                            if tokens[colon_idx].kind == TokenKind::Colon {
                                builder.start_node(YtlLanguage::kind_to_raw(YtlKind::Block));
                                builder.token(YtlLanguage::kind_to_raw(YtlKind::BlockName), &token.text);

                                for t in &tokens[i + 1..next_idx] {
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &t.text);
                                }

                                builder.start_node(YtlLanguage::kind_to_raw(YtlKind::BlockParents));
                                let mut k = next_idx;
                                while k <= j && k < tokens.len() {
                                    if tokens[k].kind == TokenKind::Word {
                                        builder.token(YtlLanguage::kind_to_raw(YtlKind::BlockName), &tokens[k].text);
                                    } else {
                                        builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &tokens[k].text);
                                    }
                                    k += 1;
                                }
                                builder.finish_node();

                                for t in &tokens[j + 1..=colon_idx] {
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &t.text);
                                }

                                block_stack.push(token.char);
                                i = colon_idx + 1;
                                continue;
                            }
                        }
                    }
                    else if next.kind == TokenKind::Colon {
                        builder.start_node(YtlLanguage::kind_to_raw(YtlKind::Block));
                        builder.token(YtlLanguage::kind_to_raw(YtlKind::BlockName), &token.text);
                        for t in &tokens[i + 1..=next_idx] {
                            builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &t.text);
                        }
                        block_stack.push(token.char);
                        i = next_idx + 1;
                        continue;
                    }
                    else if matches!(
                        next.kind,
                        TokenKind::Word
                            | TokenKind::Network
                            | TokenKind::OpenParen
                            | TokenKind::OpenBracket
                            | TokenKind::StringLiteral
                            | TokenKind::Range
                            | TokenKind::EnvVar
                            | TokenKind::Url
                            | TokenKind::IpV6
                            | TokenKind::Path
                            | TokenKind::Operator // Добавили оператор для `>`
                    ) {
                        if token.text == "Const" {
                            builder.start_node(YtlLanguage::kind_to_raw(YtlKind::ConstKeyword));
                            builder.token(YtlLanguage::kind_to_raw(YtlKind::FieldName), &token.text);

                            for t in &tokens[i + 1..next_idx] {
                                builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &t.text);
                            }
                            builder.token(YtlLanguage::kind_to_raw(YtlKind::FieldName), &tokens[next_idx].text);

                            if let Some(val_idx) = next_meaningful_on_line(tokens, next_idx + 1, token.line) {
                                for t in &tokens[next_idx + 1..val_idx] {
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &t.text);
                                }
                                builder.token(YtlLanguage::kind_to_raw(YtlKind::FieldValue), &tokens[val_idx].text);
                                i = val_idx + 1;
                            } else {
                                i = next_idx + 1;
                            }
                            builder.finish_node();
                            continue;
                        }
                        else if token.text == "Realm" {
                            builder.start_node(YtlLanguage::kind_to_raw(YtlKind::Realm));
                            builder.token(YtlLanguage::kind_to_raw(YtlKind::BlockName), &token.text);

                            let mut j = i + 1;
                            while j < tokens.len() && tokens[j].line == token.line {
                                if is_trivia(&tokens[j].kind) || tokens[j].kind == TokenKind::Colon {
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &tokens[j].text);
                                } else {
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::BlockName), &tokens[j].text);
                                }
                                j += 1;
                            }
                            
                            block_stack.push(token.char);
                            i = j;
                            continue;
                        }

                        // ИСПРАВЛЕННЫЙ БЛОК ОБЫЧНОГО ПОЛЯ
                        builder.start_node(YtlLanguage::kind_to_raw(YtlKind::Field));
                        builder.token(YtlLanguage::kind_to_raw(YtlKind::FieldName), &token.text);

                        let mut j = i + 1;
                        let mut found_type_arrow = false;
                        let mut combined_value = String::new();

                        while j < tokens.len() && tokens[j].line == token.line {
                            if is_trivia(&tokens[j].kind) {
                                // Если мы уже начали собирать значение (встретили >), пробелы идут внутрь значения
                                if found_type_arrow || !combined_value.is_empty() {
                                    combined_value.push_str(&tokens[j].text);
                                } else {
                                    // Пробелы до стрелки остаются отдельными токенами
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &tokens[j].text);
                                }
                            } else if tokens[j].text == ">" {
                                found_type_arrow = true;
                                if !combined_value.is_empty() {
                                    combined_value.push_str(&tokens[j].text); // Добавляем ">" в конец (если что-то было)
                                } else {
                                    combined_value.push_str(&tokens[j].text); // Или начинаем с ">"
                                }
                            } else {
                                if found_type_arrow {
                                    // Всё, что после стрелки, склеиваем в одну строку
                                    combined_value.push_str(&tokens[j].text);
                                } else {
                                    // Всё, что до стрелки — это обычные значения (например, если их несколько)
                                    builder.token(YtlLanguage::kind_to_raw(YtlKind::FieldValue), &tokens[j].text);
                                }
                            }
                            j += 1;
                        }

                        // Если мы что-то накопили (стрелку и тип), отдаем это как ОДИН FieldValue
                        if !combined_value.is_empty() {
                            builder.token(YtlLanguage::kind_to_raw(YtlKind::FieldValue), &combined_value);
                        }

                        builder.finish_node();
                        i = j;
                        continue;
                    }
                }
                builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &token.text);
                i += 1;
            }
            _ => {
                builder.token(YtlLanguage::kind_to_raw(YtlKind::Whitespace), &token.text);
                i += 1;
            }
        }
    }

    while !block_stack.is_empty() {
        builder.finish_node();
        block_stack.pop();
    }
    builder.finish_node();

    builder.finish()
}