use ytl_core::syntax::{SyntaxNode, YtlKind};
use tower_lsp_server::ls_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

pub fn get_hover(cst: &SyntaxNode, position: Position) -> Option<Hover> {
    let offset = position_to_offset(cst, position)?;
    let node = cst.token_at_offset(offset).right_biased()?.parent()?;

    // Если это Field, проверяем имя (нужно только для Include и Attach)
    if node.kind() == YtlKind::Field {
        if let Some(name_token) = node.children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| t.kind() == YtlKind::FieldName)
        {
            let name = name_token.text().trim();
            let description = match name {
                "Include" => "Includes another YTL file at the root level.",
                "Attach" => "Attaches an external component or trait to the current block.",
                _ => return None, // Для всех остальных полей (damage, hp и т.д.) подсказок НЕТ
            };

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}**\n\n{}", name, description),
                }),
                range: None,
            });
        }
    }

    // Проверяем остальные специальные узлы
    let (title, description) = match node.kind() {
        YtlKind::ConstKeyword => {
            ("Const".to_string(), "This field is immutable. It cannot be changed in child blocks or Realms.")
        }
        YtlKind::Realm => {
            ("Realm".to_string(), "Environment overlay (e.g., prod, dev, Hardcore). Overrides fields without duplicating code.")
        }
        _ => return None, // Для всего остального (включая обычные BlockName) подсказок НЕТ
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("**{}**\n\n{}", title, description),
        }),
        range: None,
    })
}

// Вспомогательная функция (без изменений)
fn position_to_offset(root: &SyntaxNode, position: Position) -> Option<rowan::TextSize> {
    let mut current_line = 0;
    let mut current_col = 0;
    let text = root.text().to_string();

    for (i, c) in text.char_indices() {
        if current_line == position.line && current_col == position.character {
            return Some(rowan::TextSize::from(i as u32));
        }
        if c == '\n' {
            current_line += 1;
            current_col = 0;
        } else if c != '\r' { // Игнорируем \r для Windows
            current_col += 1;
        }
    }
    None
}