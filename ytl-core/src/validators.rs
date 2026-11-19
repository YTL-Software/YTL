use std::net::{Ipv4Addr, Ipv6Addr};
use uuid::Uuid;
use crate::locales::Locale;

// === СТРУКТУРА ДЛЯ УМНЫХ ОШИБОК ===
pub struct ValidationError {
    pub title: String,
    pub message: String,
}

impl ValidationError {
    fn new(title: &str, message: &str) -> Self {
        Self { 
            title: title.to_string(), 
            message: message.to_string() 
        }
    }
}

// === IPv4 ===
pub fn validate_ipv4(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    let text = text.trim_start_matches('\u{FEFF}'); 
    if text.contains('"') || text.contains('\'') {
        return Err(ValidationError::new(&locale.ipv4_quotes_forbidden, &locale.ipv4_quotes_forbidden_msg));
    }
    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.ipv4_spaces_forbidden, &locale.ipv4_spaces_forbidden_msg));
    }
    if text.contains(':') {
        return Err(ValidationError::new(&locale.ipv4_port_forbidden, &locale.ipv4_port_forbidden_msg));
    }
    if text.contains('/') {
        return Err(ValidationError::new(&locale.ipv4_cidr_forbidden, &locale.ipv4_cidr_forbidden_msg));
    }

    // Ищем не-ASCII точки
    if let Some(bad_dot) = text.chars().find(|&c| c != '.' && !c.is_ascii_digit() && !c.is_ascii_alphabetic()) {
        if bad_dot == '．' || bad_dot == '․' || bad_dot == '。' {
            let char_display = format!("'{}' [U+{:04X}]", bad_dot, bad_dot as u32);
            return Err(ValidationError::new(
                &locale.ipv4_bad_dot, 
                &locale.ipv4_bad_dot_msg.replacen("{}", &char_display, 1)
            ));
        }
    }

    let parts: Vec<&str> = text.split('.').collect();
    
    if parts.len() == 1 {
        // Если нет ASCII-точек, но есть подозрительные символы — скажем о них
        if let Some(bad_char) = text.chars().find(|&c| !c.is_ascii_digit() && c != '.') {
            let char_display = format!("'{}' [U+{:04X}]", bad_char, bad_char as u32);
            return Err(ValidationError::new(
                &locale.ipv4_bad_dot, 
                &locale.ipv4_bad_dot_msg.replacen("{}", &char_display, 1)
            ));
        }
        return Err(ValidationError::new(&locale.ipv4_number_instead, &locale.ipv4_number_instead_msg));
    }

    if text.starts_with('.') || text.ends_with('.') {
         return Err(ValidationError::new(&locale.ipv4_start_end_dot, &locale.ipv4_start_end_dot_msg));
    }

    if parts.len() != 4 {
        return Err(ValidationError::new(
            &locale.ipv4_invalid_format,
            &locale.ipv4_expected_octets.replacen("{}", &parts.len().to_string(), 1)
        ));
    }
    
    let mut out_of_range = Vec::new();
    let mut bad_chars_info = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        for bad_char in part.chars().filter(|c| !c.is_ascii_digit()) {
            let char_display = format!("'{}' [U+{:04X}]", bad_char, bad_char as u32);
            bad_chars_info.push(format!("  {} {}: {}", locale.ipv4_octet_name, i + 1, char_display)); 
        }

        if part.is_empty() {
            return Err(ValidationError::new(
                &locale.ipv4_empty_octet,
                &locale.ipv4_empty_octet_msg.replacen("{}", &(i + 1).to_string(), 1)
            ));
        }

        // 1. НОВОЕ: Проверка на HEX (0x0a)
        if part.starts_with("0x") || part.starts_with("0X") {
            return Err(ValidationError::new(
                &locale.ipv4_bad_char, 
                &format!("{} {}: {}", locale.ipv4_octet_name, i + 1, part)
            ));
        }

        let char_count = part.chars().count();
        if char_count > 3 {
            let msg = locale.ipv4_octet_too_long_new_msg
                .replacen("{}", &(i + 1).to_string(), 1)
                .replacen("{}", &char_count.to_string(), 1)
                .replacen("{}", part, 1);
            return Err(ValidationError::new(&locale.ipv4_octet_too_long_new, &msg));
        }

        // 3. НОВОЕ: Проверка ведущего нуля (убираем ВСЕ нули для подсказки)
        if part.chars().count() > 1 && part.starts_with('0') {
            let msg = locale.ipv4_leading_zero_msg
                .replacen("{}", &(i + 1).to_string(), 1)
                .replacen("{}", part, 1)
                .replacen("{}", part.trim_start_matches('0'), 1); // <-- Изменено тут
            return Err(ValidationError::new(&locale.ipv4_leading_zero, &msg));
        }

        if let Ok(num) = part.parse::<u16>() {
            if num > 255 {
                out_of_range.push((i + 1, num));
            }
        }
    }

    if !bad_chars_info.is_empty() {
        let info_str = bad_chars_info.join("\n  ");
        return Err(ValidationError::new(
            &locale.ipv4_multiple_bad_chars,
            &locale.ipv4_multiple_bad_chars_msg.replacen("{}", &info_str, 1)
        ));
    }

    if out_of_range.len() == 4 {
        return Err(ValidationError::new(&locale.ipv4_all_out_of_range, &locale.ipv4_all_out_of_range_msg));
    }

    if !out_of_range.is_empty() {
        let octet_info = out_of_range.iter()
            .map(|(idx, val)| {
                locale.ipv4_out_of_range_msg
                    .replacen("{}", &idx.to_string(), 1)
                    .replacen("{}", &val.to_string(), 1)
            })
            .collect::<Vec<_>>()
            .join(", ");
            
        return Err(ValidationError::new(&locale.ipv4_out_of_range, &octet_info));
    }
    
    Ok(())
}

// === IPv6 ===
pub fn validate_ipv6(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    let text = text.trim_start_matches('\u{FEFF}');
    let text = if let Some(idx) = text.find('%') {
        &text[..idx]
    } else {
        text
    };
    if text.contains('"') || text.contains('\'') {
        return Err(ValidationError::new(&locale.ipv6_quotes_forbidden, &locale.ipv6_quotes_forbidden_msg));
    }
    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.ipv6_whitespace, &locale.ipv6_whitespace_msg));
    }
    if text.contains('[') || text.contains(']') {
        return Err(ValidationError::new(&locale.ipv6_brackets, &locale.ipv6_brackets_msg));
    }
    if text.contains('/') {
        return Err(ValidationError::new(&locale.ipv6_cidr_forbidden, &locale.ipv6_cidr_forbidden_msg));
    }

    // 1. Ищем поддельные двоеточия
    let fake_colons = text.chars().filter(|&c| c != ':' && (c == '∶' || c == '꞉' || c == '：' || c == '﹕')).collect::<Vec<_>>();
    if !fake_colons.is_empty() {
        let mut info = String::new();
        for c in fake_colons {
            info.push_str(&format!("  '{}' [U+{:04X}]\n", c, c as u32));
        }
        return Err(ValidationError::new(
            &locale.ipv6_bad_dot, 
            &locale.ipv6_bad_dot_msg.replacen("{}", &info, 1)
        ));
    }
    if let Some(bad_dot) = text.chars().find(|&c| c != '.' && (c == '．' || c == '․' || c == '。')) {
        let char_display = format!("'{}' [U+{:04X}]", bad_dot, bad_dot as u32);
        return Err(ValidationError::new(
            &locale.ipv6_bad_dot, 
            &locale.ipv6_bad_dot_msg.replacen("{}", &char_display, 1)
        ));
    }
    if text.contains(":::") {
        return Err(ValidationError::new(&locale.ipv6_triple_colons, &locale.ipv6_triple_colons_msg));
    }
    
    let double_colon_count = text.matches("::").count();
    if double_colon_count > 1 {
        return Err(ValidationError::new(
            &locale.ipv6_multiple_colons,
            &locale.ipv6_multiple_colons_msg.replacen("{}", &double_colon_count.to_string(), 1)
        ));
    }

    let has_double_colon = text.contains("::");
    
    if text.starts_with(':') && !text.starts_with("::") {
        return Err(ValidationError::new(&locale.ipv6_empty_first_group, &locale.ipv6_empty_first_group_msg));
    }
    if text.ends_with(':') && !text.ends_with("::") {
        return Err(ValidationError::new(&locale.ipv6_empty_last_group, &locale.ipv6_empty_last_group_msg));
    }

    let mut parts: Vec<&str> = text.split(':').collect();
    if has_double_colon {
        parts.retain(|s| !s.is_empty());
    }

    let is_embedded_ipv4 = text.contains('.');
    let expected_groups = if is_embedded_ipv4 { 7 } else { 8 };

    if !has_double_colon {
        if parts.len() != expected_groups {
            return Err(ValidationError::new(
                &locale.ipv6_missing_group,
                &locale.ipv6_missing_group_msg
                    .replacen("{}", &expected_groups.to_string(), 1)
                    .replacen("{}", &parts.len().to_string(), 1)
            ));
        }
    } else {
        if parts.len() >= expected_groups {
            return Err(ValidationError::new(
                &locale.ipv6_too_many_groups,
                &locale.ipv6_too_many_groups_msg.replacen("{}", &parts.len().to_string(), 1)
            ));
        }
    }

    if is_embedded_ipv4 {
        if let Some(ipv4_part) = parts.iter().find(|g| g.contains('.')) {
            if let Err(e) = validate_ipv4(ipv4_part, locale) {
                return Err(ValidationError::new(&locale.ipv6_invalid_embedded_ipv4, &e.message));
            }
        }
    }

    let mut bad_chars_info = Vec::new();

    for (i, group) in parts.iter().enumerate() {
        if group.is_empty() || group.contains('.') { 
            continue; 
        }

        for bad_char in group.chars().filter(|c| !c.is_ascii_hexdigit()) {
            let char_display = format!("'{}' [U+{:04X}]", bad_char, bad_char as u32);
            // Здесь используем локаль для названия группы
            bad_chars_info.push(format!("  {} {}: {}", locale.ipv6_group_name, i + 1, char_display));
        }

        let char_count = group.chars().count();
        if char_count > 4 {
            return Err(ValidationError::new(
                &locale.ipv6_group_too_long,
                &locale.ipv6_group_too_long_msg
                    .replacen("{}", &(i + 1).to_string(), 1)
                    .replacen("{}", &char_count.to_string(), 1)
            ));
        }
    }

    if !bad_chars_info.is_empty() {
        return Err(ValidationError::new(
            &locale.ipv6_multiple_bad_chars,
            &locale.ipv6_multiple_bad_chars_msg.replacen("{}", &format!("\n{}", bad_chars_info.join("\n")), 1)
        ));
    }
    
    Ok(())
}

// === UUID ===
// === UUID ===
pub fn validate_uuid(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    // Убираем BOM
    let text = text.trim_start_matches('\u{FEFF}');

    if text.contains('"') || text.contains('\'') {
        return Err(ValidationError::new(&locale.uuid_quotes_forbidden, &locale.uuid_quotes_forbidden_msg));
    }

    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.uuid_whitespace, &locale.uuid_whitespace_msg));
    }
    if text.starts_with("urn:") {
        return Err(ValidationError::new(&locale.uuid_urn, &locale.uuid_urn_msg));
    }
    if text.starts_with('{') || text.ends_with('}') {
        return Err(ValidationError::new(&locale.uuid_braces, &locale.uuid_braces_msg));
    }
    
    let char_count = text.chars().count();
    if char_count != 36 {
        return Err(ValidationError::new(
            &locale.uuid_invalid_length,
            &locale.uuid_invalid_length_msg.replacen("{}", &char_count.to_string(), 1)
        ));
    }
    
    let hyphens = text.matches('-').count();
    if hyphens != 4 {
        return Err(ValidationError::new(
            &locale.uuid_missing_hyphens,
            &locale.uuid_missing_hyphens_msg.replacen("{}", &hyphens.to_string(), 1)
        ));
    }

    let chars: Vec<char> = text.chars().collect();
    if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
        return Err(ValidationError::new(&locale.uuid_invalid_format, &locale.uuid_invalid_format_msg));
    }
    
    // Проверяем каждый символ (кроме дефисов)
    for c in chars {
        if c != '-' && !c.is_ascii_hexdigit() {
            let char_display = format!("'{}' [U+{:04X}]", c, c as u32);
            return Err(ValidationError::new(
                &locale.uuid_bad_char,
                &locale.uuid_bad_char_msg.replacen("{0}", &char_display, 1)
            ));
        }
    }
    
    // Финальная проверка через крейт uuid (на всякий случай)
    if Uuid::parse_str(text).is_ok() {
        return Ok(());
    }
    
    Err(ValidationError::new(&locale.uuid_invalid_format, &locale.uuid_invalid_format_msg))
}

// === CIDR ===
pub fn validate_cidr(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    // 1. Проверка на BOM
    let text = text.trim_start_matches('\u{FEFF}');

    // 2. Проверка на обратный слэш и поддельные слэши
    if text.contains('\\') {
        return Err(ValidationError::new(&locale.cidr_backslash, &locale.cidr_backslash_msg));
    }
    if let Some(bad_slash) = text.chars().find(|&c| c != '/' && (c == '∕' || c == '／' || c == '⁄')) {
        let char_display = format!("'{}' [U+{:04X}]", bad_slash, bad_slash as u32);
        return Err(ValidationError::new(
            &locale.cidr_bad_char, 
            &locale.cidr_bad_char_msg.replacen("{}", &char_display, 1)
        ));
    }

    // 3. Проверка на наличие слэша
    if !text.contains('/') {
        if text.contains(':') {
            return Err(ValidationError::new(&locale.cidr_missing_prefix_ipv6, &locale.cidr_missing_prefix_ipv6_msg));
        }
        return Err(ValidationError::new(&locale.cidr_missing_prefix, &locale.cidr_missing_prefix_msg));
    }

    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() != 2 {
        return Err(ValidationError::new(&locale.cidr_too_many_slashes, &locale.cidr_too_many_slashes_msg));
    }
    let ip_part = parts[0];
    let prefix_part = parts[1];

    if ip_part.is_empty() {
        return Err(ValidationError::new(&locale.cidr_empty_ip, &locale.cidr_empty_ip_msg));
    }

        if ip_part.contains('%') {
        return Err(ValidationError::new(
            &locale.cidr_zone_forbidden,
            &locale.cidr_zone_forbidden_msg,
        ));
    }

    // 4. Проверяем IP
    if ip_part.contains(':') {
        validate_ipv6(ip_part, locale)?;
    } else {
        if ip_part.contains("..") {
            return Err(ValidationError::new(&locale.cidr_double_dot, &locale.cidr_double_dot_msg));
        }
        validate_ipv4(ip_part, locale)?;
    }

    // 5. Проверка префикса
    if prefix_part.is_empty() {
        return Err(ValidationError::new(&locale.cidr_empty_prefix, &locale.cidr_empty_prefix_msg));
    }

    if prefix_part.contains('.') {
        return Err(ValidationError::new(&locale.cidr_subnet_mask, &locale.cidr_subnet_mask_msg));
    }

    // 6. Собираем все недопустимые символы в префиксе
    let mut bad_chars_info = Vec::new();
    for bad_char in prefix_part.chars().filter(|c| !c.is_ascii_digit()) {
        let char_display = format!("'{}' [U+{:04X}]", bad_char, bad_char as u32);
        bad_chars_info.push(char_display);
    }
    if !bad_chars_info.is_empty() {
        return Err(ValidationError::new(
            &locale.cidr_bad_char, 
            &format!("Префикс содержит недопустимые символы: {}", bad_chars_info.join(", "))
        ));
    }

    // 7. Проверка на ведущий ноль
    if prefix_part.chars().count() > 1 && prefix_part.starts_with('0') {
        return Err(ValidationError::new(
            &locale.cidr_leading_zero,
            &locale.cidr_leading_zero_msg
                .replacen("{}", prefix_part, 1)
                .replacen("{}", &prefix_part[1..], 1)
        ));
    }

    // 8. Парсинг и диапазон
    if prefix_part.len() > 3 { 
        if ip_part.contains(':') {
            return Err(ValidationError::new(&locale.cidr_prefix_too_large_ipv6, &locale.cidr_prefix_too_large_ipv6_msg));
        } else {
            return Err(ValidationError::new(&locale.cidr_prefix_too_large_ipv4, &locale.cidr_prefix_too_large_ipv4_msg));
        }
    }

    if let Ok(prefix) = prefix_part.parse::<u32>() {
        if ip_part.contains(':') {
            if prefix > 128 {
                return Err(ValidationError::new(
                    &locale.cidr_prefix_out_of_range_ipv6,
                    &locale.cidr_prefix_out_of_range_ipv6_msg.replacen("{}", &prefix.to_string(), 1)
                ));
            }
        } else {
            if prefix > 32 {
                return Err(ValidationError::new(
                    &locale.cidr_prefix_out_of_range_ipv4,
                    &locale.cidr_prefix_out_of_range_ipv4_msg.replacen("{}", &prefix.to_string(), 1)
                ));
            }
        }
    } else {
        return Err(ValidationError::new(
            &locale.cidr_invalid_prefix,
            &locale.cidr_invalid_prefix_msg.replacen("{}", prefix_part, 1)
        ));
    }
    
    Ok(())
}

// === MAC ===
pub fn validate_mac(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    // Убираем BOM
    let text = text.trim_start_matches('\u{FEFF}');
    
    if text.is_empty() {
        return Err(ValidationError::new(&locale.mac_empty, &locale.mac_empty_msg));
    }
    
    if text.contains('"') || text.contains('\'') {
        return Err(ValidationError::new(&locale.mac_quotes_forbidden, &locale.mac_quotes_forbidden_msg));
    }
    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.mac_whitespace, &locale.mac_whitespace_msg));
    }

    // 1. Проверка на недопустимые символы (выводим конкретный символ)
    let clean_text = text.replace(|c| c == ':' || c == '-' || c == '.', "");
    if let Some(bad_char) = clean_text.chars().find(|c| !c.is_ascii_hexdigit()) {
        let char_display = format!("'{}' [U+{:04X}]", bad_char, bad_char as u32);
        return Err(ValidationError::new(
            &locale.mac_bad_char, 
            &locale.mac_bad_char_msg.replacen("{0}", &char_display, 1)
        ));
    }

    let has_colon = text.contains(':');
    let has_dash = text.contains('-');
    let has_dot = text.contains('.');

    // 2. Проверка на смешанные разделители
    if (has_colon as u8 + has_dash as u8 + has_dot as u8) > 1 {
        return Err(ValidationError::new(&locale.mac_mixed_separators, &locale.mac_mixed_separators_msg));
    }

    // 3. Проверка на двойные разделители
    if text.contains("::") || text.contains("--") || text.contains("..") {
         return Err(ValidationError::new(&locale.mac_double_sep, &locale.mac_double_sep_msg));
    }

    let len = clean_text.len();
    let parts: Vec<&str> = if has_dot {
        text.split('.').collect()
    } else {
        text.split(|c| c == ':' || c == '-').collect()
    };

    // 4. Пустые группы
    if parts.first() == Some(&"") || parts.last() == Some(&"") {
        return Err(ValidationError::new(&locale.mac_invalid_grouping, &locale.mac_empty_group_msg));
    }

    // 5. Проверка длины
    if !has_colon && !has_dash && !has_dot {
        if len != 12 && len != 16 {
            return Err(ValidationError::new(
                &locale.mac_invalid_length,
                &locale.mac_unseparated_length_msg.replacen("{}", &len.to_string(), 1)
            ));
        }
        return Ok(());
    }

    if len != 12 && len != 16 && len != 20 && len != 40 {
        return Err(ValidationError::new(
            &locale.mac_invalid_length,
            &locale.mac_invalid_length_msg.replacen("{}", &len.to_string(), 1)
        ));
    }

    // 6. Проверка групп (используем {0}, {1}, {2})
    let expected_groups = if has_dot { len / 4 } else { len / 2 };
    
    if parts.len() != expected_groups {
        let format_example = if has_dot { "0011.2233.4455" } else { "00:11:22:33:44:55" };
        return Err(ValidationError::new(
            &locale.mac_invalid_grouping,
            &locale.mac_invalid_grouping_msg
                .replacen("{0}", &expected_groups.to_string(), 1)
                .replacen("{1}", &parts.len().to_string(), 1)
                .replacen("{2}", format_example, 1)
        ));
    }

    let group_size = if has_dot { 4 } else { 2 };
    for part in parts {
        if part.len() != group_size {
            return Err(ValidationError::new(
                &locale.mac_invalid_group_len,
                &locale.mac_invalid_group_len_msg
                    .replacen("{0}", part, 1)
                    .replacen("{1}", &part.len().to_string(), 1)
                    .replacen("{2}", &group_size.to_string(), 1)
            ));
        }
    }
    
    Ok(())
}