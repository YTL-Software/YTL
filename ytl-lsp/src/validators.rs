use regex::Regex;

pub fn validate_ipv4(text: &str) -> Result<(), String> {
    if text.parse::<std::net::Ipv4Addr>().is_ok() {
        return Ok(());
    }
    
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
    if text.parse::<std::net::Ipv6Addr>().is_ok() {
        return Ok(());
    }

    if text.matches("::").count() > 1 {
        return Err("multiple '::' are not allowed".to_string());
    }

    let has_double_colon = text.contains("::");
    let groups: Vec<&str> = text.split(':').collect();

    if !has_double_colon && groups.len() != 8 {
        return Err(format!("expected 8 groups, found {}", groups.len()));
    }
    if has_double_colon && groups.len() > 8 {
        return Err(format!("too many groups (found {}, max 8 with '::')", groups.len()));
    }

    for (i, group) in groups.iter().enumerate() {
        if group.is_empty() {
            continue;
        }
        
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
    if text.parse::<ipnet::IpNet>().is_ok() {
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

pub fn is_version(text: &str) -> bool{
    let re = Regex::new(r"^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.+-]*)?$").unwrap();
    re.is_match(text)
}

pub fn validate_mac(text: &str) -> Result<(), String> {
    if text.contains(':') && text.contains('-') {
        return Err("mixed separators (use either ':' or '-')".to_string());
    }

    if text.contains('.') {
        let re = regex::Regex::new(r"^([0-9A-Fa-f]{4}\.){2}[0-9A-Fa-f]{4}$").unwrap();
        if re.is_match(text) {
            return Ok(());
        }
        return Err("invalid Cisco format (expected XXXX.XXXX.XXXX)".to_string());
    }

    let re = regex::Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}$").unwrap();
    if re.is_match(text) {
        return Ok(());
    }

    if text.len() != 17 {
        return Err(format!("expected 17 characters, found {}", text.len()));
    }
    if !text.chars().all(|c| c.is_ascii_hexdigit() || c == ':' || c == '-') {
        return Err("contains invalid characters (only 0-9, A-F, ':' or '-')".to_string());
    }

    Err("invalid MAC address format".to_string())
}