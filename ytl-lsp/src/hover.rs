use tower_lsp_server::ls_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

pub fn get_hover(text: &str, position: Position) -> Option<Hover> {
    let lines: Vec<&str> = text.lines().collect();
    let line = lines.get(position.line as usize)?;

    // 1. Подсказка для Include
    if line.contains("Include") {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**Include**\n\nGlobally imports root blocks from another file.\n\nMust be at the root level (no indentation).".to_string(),
            }),
            range: None,
        });
    }

    // 2. Подсказка для Attach
    if line.contains("Attach") {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**Attach**\n\nAttaches components (stats, traits) directly into this entity.\n\nMust be inside a block (indented).".to_string(),
            }),
            range: None,
        });
    }

    // 3. Подсказка для Realm
    if line.contains("Realm") {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**Realm**\n\nOverlays settings for a specific environment (e.g. `Realm \"prod\":`).".to_string(),
            }),
            range: None,
        });
    }

    None
}