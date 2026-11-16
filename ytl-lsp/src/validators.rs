use std::net::{Ipv4Addr, Ipv6Addr};
use ipnet::IpNet;
use uuid::Uuid;
use eui48::MacAddress;

pub fn validate_ipv4(text: &str) -> Result<(), String> {
    if text.parse::<Ipv4Addr>().is_ok() {
        return Ok(());
    }
    
    // Если стандартный парсер не справился, даем более понятную ошибку
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 4 {
        return Err(format!("expected 4 octets, found {}", parts.len()));
    }
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return Err(format!("octet {} is empty", i + 1));
        }
        if part.len() > 3 {
            return Err(format!("octet {} is too long ({} > 3 digits)", i + 1, part.len()));
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(format!("octet {} has a leading zero ('{}')", i + 1, part));
        }
        if let Ok(num) = part.parse::<u16>() {
            if num > 255 {
                return Err(format!("octet {} is out of range ({} > 255)", i + 1, num));
            }
        } else {
            return Err(format!("octet {} contains invalid characters", i + 1));
        }
    }
    Err("invalid IPv4 format".to_string())
}

pub fn validate_ipv6(text: &str) -> Result<(), String> {
    let clean_text = if text.starts_with('[') {
        text.split(']').next().unwrap_or(text).trim_start_matches('[')
    } else {
        text
    };

    if clean_text.parse::<Ipv6Addr>().is_ok() {
        return Ok(());
    }

    if clean_text.matches("::").count() > 1 {
        return Err("multiple '::' are not allowed".to_string());
    }

    let has_double_colon = clean_text.contains("::");
    let groups: Vec<&str> = clean_text.split(':').collect();

    if !has_double_colon && groups.len() != 8 {
        return Err(format!("expected 8 groups, found {}", groups.len()));
    }
    if has_double_colon && groups.len() > 8 {
        return Err(format!("too many groups (found {}, max 8 with '::')", groups.len()));
    }

    for (i, group) in groups.iter().enumerate() {
        if group.is_empty() { continue; }
        if group.len() > 4 {
            return Err(format!("group {} is too long ({} > 4 characters)", i + 1, group.len()));
        }
        if !group.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("group {} contains invalid characters ('{}')", i + 1, group));
        }
    }

    Err("invalid IPv6 format".to_string())
}

pub fn validate_cidr(text: &str) -> Result<(), String> {
    if text.parse::<IpNet>().is_ok() {
        return Ok(());
    }

    if !text.contains('/') {
        return Err("CIDR must contain a prefix (e.g. /24)".to_string());
    }

    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() != 2 {
        return Err("invalid CIDR format".to_string());
    }

    if parts[1].is_empty() {
        return Err("prefix is missing (e.g. /24)".to_string());
    }

    if let Ok(prefix) = parts[1].parse::<u8>() {
        if parts[0].contains(':') {
            if prefix > 128 {
                return Err(format!("IPv6 prefix must be between 0 and 128 (got {})", prefix));
            }
        } else {
            if prefix > 32 {
                return Err(format!("IPv4 prefix must be between 0 and 32 (got {})", prefix));
            }
        }
    } else {
        return Err(format!("invalid prefix '{}' (must be a number)", parts[1]));
    }

    Err("invalid CIDR subnet".to_string())
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

// ==========================================
// НОВАЯ ВАЛИДАЦИЯ (через крейты)
// ==========================================

pub fn validate_mac(text: &str) -> Result<(), String> {
    // eui48 умеет парсить "00:11:22:33:44:55", "00-11-22-33-44-55" и Cisco "0011.2233.4455"
    if MacAddress::parse_str(text).is_ok() {
        return Ok(());
    }
    
    // Если не распарсилось, даем понять, что не так (проверяем базовые вещи)
    if text.contains(':') && text.contains('-') {
        return Err("mixed separators (use either ':' or '-')".to_string());
    }
    
    let expected_len = if text.contains('.') { 14 } else { 17 };
    if text.len() != expected_len {
        return Err(format!("invalid format (expected {} characters)", expected_len));
    }
    
    Err("invalid MAC address format".to_string())
}

pub fn validate_uuid(text: &str) -> Result<(), String> {
    if Uuid::parse_str(text).is_ok() {
        return Ok(());
    }

    // Если не распарсилось, даем понятную ошибку
    let expected_len = 36;
    if text.len() != expected_len {
        return Err(format!("expected 36 characters, found {}", text.len()));
    }

    if text.matches('-').count() != 4 {
        return Err(format!("expected 4 hyphens, found {}", text.matches('-').count()));
    }

    Err("invalid UUID format".to_string())
}