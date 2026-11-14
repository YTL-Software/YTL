use tower_lsp_server::ls_types::{CompletionItem, CompletionItemKind, Position};

pub fn get_completions(text: &str, position: Position) -> Vec<CompletionItem> {
    let lines: Vec<&str> = text.lines().collect();
    let line = match lines.get(position.line as usize) {
        Some(l) => l,
        None => return vec![],
    };

    let prefix: String = line.chars().take(position.character as usize).collect();
    
    // --- 1. ПРОВЕРКА: Мы внутри скобок наследования? ---
    // Если строка содержит '{' и '}' и курсор между ними
    if prefix.contains('{') && !prefix.contains('}') {
        let mut items = Vec::new();
        
        // Ищем все блоки в файле (строки, которые заканчиваются на ':')
        for (i, l) in lines.iter().enumerate() {
            let trimmed = l.trim();
            if i == position.line as usize { continue; }
            
            // Если строка заканчивается на ':' и это не комментарий
            if trimmed.ends_with(':') && !trimmed.starts_with("//") && !trimmed.starts_with("#") {
                // Извлекаем имя блока
                let name_part = trimmed.split(':').next().unwrap_or("").trim();
                let name = name_part.split('{').next().unwrap_or("").trim();
                
                // ФИЛЬТР: Пропускаем Realm, Include, Attach, Const
                if name.starts_with("Realm") || name == "Include" || name == "Attach" || name == "Const" {
                    continue;
                }

                if !name.is_empty() {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some("YTL Block".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
        return items;
    }

    // --- 2. ОБЫЧНЫЕ КЛЮЧЕВЫЕ СЛОВА ---
    let word = prefix.split_whitespace().last().unwrap_or("");
    let mut items = Vec::new();
    let keywords = vec!["Include", "Attach", "Const", "Realm"];
    
    for kw in keywords {
        if kw.to_lowercase().starts_with(&word.to_lowercase()) {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("YTL Keyword".to_string()),
                ..Default::default()
            });
        }
    }

    items
}