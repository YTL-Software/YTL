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
    let clean = text.trim_matches(|c| c == '"' || c == '\'' || c == ' ');

    if clean.parse::<Ipv4Addr>().is_ok() {
        return Ok(());
    }
    
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.len() != 4 {
        return Err(ValidationError::new(
            &locale.ipv4_invalid_format,
            &locale.ipv4_expected_octets.replace("{}", &parts.len().to_string())
        ));
    }
    
    let mut out_of_range = Vec::new();
    let mut invalid_chars = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        if !part.chars().all(|c| c.is_ascii_digit()) {
            invalid_chars.push(i + 1);
        } else if let Ok(num) = part.parse::<u16>() {
            if num > 255 {
                out_of_range.push(i + 1);
            }
        }
    }

    if out_of_range.len() == 4 {
        return Err(ValidationError::new(&locale.ipv4_all_out_of_range, &locale.ipv4_all_out_of_range_msg));
    }

    if out_of_range.len() > 1 {
        let octets_str = out_of_range.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        return Err(ValidationError::new(
            &locale.ipv4_multiple_out_of_range,
            &locale.ipv4_multiple_out_of_range_msg.replace("{}", &octets_str)
        ));
    }

    if !invalid_chars.is_empty() {
        let octets_str = invalid_chars.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        return Err(ValidationError::new(
            &locale.ipv4_invalid_chars,
            &locale.ipv4_invalid_chars_msg.replace("{}", &octets_str)
        ));
    }

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return Err(ValidationError::new(
                &locale.ipv4_empty_octet,
                &locale.ipv4_empty_octet_msg.replace("{}", &(i + 1).to_string())
            ));
        }
        if part.chars().count() > 3 {
            return Err(ValidationError::new(
                &locale.ipv4_octet_too_long,
                &locale.ipv4_octet_too_long_msg.replace("{}", &(i + 1).to_string()).replace("{}", &part.chars().count().to_string())
            ));
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(ValidationError::new(
                &locale.ipv4_leading_zero,
                &locale.ipv4_leading_zero_msg.replace("{}", &(i + 1).to_string()).replace("{}", part).replace("{}", &part[1..])
            ));
        }
        if let Ok(num) = part.parse::<u16>() {
            if num > 255 {
                return Err(ValidationError::new(
                    &locale.ipv4_out_of_range,
                    &locale.ipv4_out_of_range_msg.replace("{}", &(i + 1).to_string()).replace("{}", &num.to_string())
                ));
            }
        }
    }
    
    Err(ValidationError::new(&locale.ipv4_invalid_format, "Unknown error."))
}

// === IPv6 ===
pub fn validate_ipv6(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.ipv6_whitespace, &locale.ipv6_whitespace_msg));
    }
    if text.contains('[') || text.contains(']') {
        return Err(ValidationError::new(&locale.ipv6_brackets, &locale.ipv6_brackets_msg));
    }

    if text.parse::<Ipv6Addr>().is_ok() {
        return Ok(());
    }
    if text.contains(":::") {
        return Err(ValidationError::new(&locale.ipv6_triple_colons, &locale.ipv6_triple_colons_msg));
    }
    if text.matches("::").count() > 1 {
        return Err(ValidationError::new(
            &locale.ipv6_multiple_colons,
            &locale.ipv6_multiple_colons_msg.replace("{}", &text.matches("::").count().to_string())
        ));
    }

    let has_double_colon = text.contains("::");
    let groups: Vec<&str> = text.split(':').collect();
    let is_embedded_ipv4 = text.contains('.');
    let expected_groups = if is_embedded_ipv4 { 7 } else { 8 };

    if !has_double_colon && groups.len() != expected_groups {
        return Err(ValidationError::new(
            &locale.ipv6_missing_group,
            &locale.ipv6_missing_group_msg.replace("{}", &expected_groups.to_string()).replace("{}", &groups.len().to_string())
        ));
    }

    if has_double_colon && groups.len() > 8 {
        return Err(ValidationError::new(
            &locale.ipv6_too_many_groups,
            &locale.ipv6_too_many_groups_msg.replace("{}", &groups.len().to_string())
        ));
    }
    if text.starts_with(':') && !text.starts_with("::") {
        return Err(ValidationError::new(&locale.ipv6_empty_first_group, &locale.ipv6_empty_first_group_msg));
    }
    if text.ends_with(':') && !text.ends_with("::") {
        return Err(ValidationError::new(&locale.ipv6_empty_last_group, &locale.ipv6_empty_last_group_msg));
    }
    
    for (i, group) in groups.iter().enumerate() {
        if group.is_empty() { continue; }
        if group.contains('.') {
            if let Err(e) = validate_ipv4(group, locale) {
                return Err(ValidationError::new(&locale.ipv6_invalid_embedded_ipv4, &e.message));
            }
            continue;
        }
        if group.len() > 4 {
            return Err(ValidationError::new(
                &locale.ipv6_group_too_long,
                &locale.ipv6_group_too_long_msg.replace("{}", &(i + 1).to_string()).replace("{}", &group.len().to_string())
            ));
        }
        if !group.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::new(
                &locale.ipv6_invalid_char,
                &locale.ipv6_invalid_char_msg.replace("{}", &(i + 1).to_string()).replace("{}", group)
            ));
        }
    }
    
    Err(ValidationError::new(&locale.ipv6_invalid_format, &locale.ipv6_invalid_format_msg))
}

// === UUID ===
pub fn validate_uuid(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.uuid_whitespace, &locale.uuid_whitespace_msg));
    }
    if text.starts_with("urn:") {
        return Err(ValidationError::new(&locale.uuid_urn, &locale.uuid_urn_msg));
    }
    if text.starts_with('{') || text.ends_with('}') {
        return Err(ValidationError::new(&locale.uuid_braces, &locale.uuid_braces_msg));
    }
    if text.len() != 36 {
        return Err(ValidationError::new(
            &locale.uuid_invalid_length,
            &locale.uuid_invalid_length_msg.replace("{}", &text.len().to_string())
        ));
    }
    if text.matches('-').count() != 4 {
        return Err(ValidationError::new(
            &locale.uuid_missing_hyphens,
            &locale.uuid_missing_hyphens_msg.replace("{}", &text.matches('-').count().to_string())
        ));
    }
    let chars: Vec<char> = text.chars().collect();
    if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
        return Err(ValidationError::new(&locale.uuid_invalid_format, &locale.uuid_invalid_format_msg));
    }
    let hex_only = text.replace("-", "");
    if !hex_only.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::new(&locale.uuid_invalid_char, &locale.uuid_invalid_char_msg));
    }
    if Uuid::parse_str(text).is_ok() {
        return Ok(());
    }
    
    Err(ValidationError::new(&locale.uuid_invalid_format, "Unknown error."))
}

// === CIDR ===
pub fn validate_cidr(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    if !text.contains('/') {
        return Err(ValidationError::new("CIDR: missing prefix", "CIDR must contain a prefix (e.g., 192.168.1.0/24)."));
    }
    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() != 2 {
        return Err(ValidationError::new("CIDR: too many slashes", "Use only one '/' (e.g., 192.168.1.0/24)."));
    }
    let ip_part = parts[0];
    let prefix_part = parts[1];

    if ip_part.contains(':') {
        if let Err(e) = validate_ipv6(ip_part, locale) {
            return Err(ValidationError::new("CIDR: invalid IPv6", &e.message));
        }
    } else {
        if let Err(e) = validate_ipv4(ip_part, locale) {
            return Err(ValidationError::new("CIDR: invalid IPv4", &e.message));
        }
    }

    if let Ok(prefix) = prefix_part.parse::<u8>() {
        if ip_part.contains(':') {
            if prefix > 128 {
                return Err(ValidationError::new("CIDR: IPv6 prefix out of range", &format!("IPv6 prefix must be between 0 and 128, got {}.", prefix)));
            }
        } else {
            if prefix > 32 {
                return Err(ValidationError::new("CIDR: IPv4 prefix out of range", &format!("IPv4 prefix must be between 0 and 32, got {}.", prefix)));
            }
        }
    } else {
        return Err(ValidationError::new("CIDR: invalid prefix", &format!("'{}' is not a valid number. Prefix must be a positive integer.", prefix_part)));
    }
    Ok(())
}

// === MAC ===
pub fn validate_mac(text: &str, locale: &Locale) -> Result<(), ValidationError> {
    if text.is_empty() {
        return Err(ValidationError::new(&locale.mac_empty, &locale.mac_empty_msg));
    }
    if text.chars().any(|c| c.is_whitespace()) {
        return Err(ValidationError::new(&locale.mac_whitespace, &locale.mac_whitespace_msg));
    }
    let clean_text = text.replace(|c| c == ':' || c == '-' || c == '.', "");
    if !clean_text.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::new(&locale.mac_invalid_char, &locale.mac_invalid_char_msg));
    }

    let has_colon = text.contains(':');
    let has_dash = text.contains('-');
    let has_dot = text.contains('.');

    if (has_colon as u8 + has_dash as u8 + has_dot as u8) > 1 {
        return Err(ValidationError::new(&locale.mac_mixed_separators, &locale.mac_mixed_separators_msg));
    }

    let len = clean_text.len();
    let expected_groups = if has_dot { len / 4 } else { len / 2 };

    if !has_colon && !has_dash && !has_dot {
        if len != 12 && len != 16 {
            return Err(ValidationError::new(
                &locale.mac_invalid_length,
                &locale.mac_unseparated_length_msg.replace("{}", &len.to_string())
            ));
        }
        return Ok(());
    }

    if len != 12 && len != 16 && len != 20 && len != 40 {
        return Err(ValidationError::new(
            &locale.mac_invalid_length,
            &locale.mac_invalid_length_msg.replace("{}", &len.to_string())
        ));
    }

    let parts: Vec<&str> = if has_dot {
        text.split('.').collect()
    } else {
        text.split(|c| c == ':' || c == '-').collect()
    };

    if parts.first() == Some(&"") || parts.last() == Some(&"") {
        return Err(ValidationError::new(&locale.mac_invalid_grouping, &locale.mac_empty_group_msg));
    }

    if parts.len() != expected_groups {
        let format_example = if has_dot { "0011.2233.4455" } else { "00:11:22:33:44:55" };
        return Err(ValidationError::new(
            &locale.mac_invalid_grouping,
            &locale.mac_invalid_grouping_msg.replace("{}", &expected_groups.to_string()).replace("{}", &parts.len().to_string()).replace("{}", format_example)
        ));
    }

    let group_size = if has_dot { 4 } else { 2 };
    for part in parts {
        if part.len() != group_size {
            return Err(ValidationError::new(
                &locale.mac_invalid_group_len,
                &locale.mac_invalid_group_len_msg.replace("{}", part).replace("{}", &part.len().to_string()).replace("{}", &group_size.to_string())
            ));
        }
    }

    Ok(())
}

pub fn is_version(text: &str) -> bool {
    let base = if let Some(idx) = text.find(|c| c == '-' || c == '+') {
        &text[..idx]
    } else {
        text
    };
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() != 3 { return false; }
    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    true
}