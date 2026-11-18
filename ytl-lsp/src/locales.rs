// src/locales.rs
#[derive(Debug, Clone)] 
pub struct Locale {
    // IPv4
    pub ipv4_invalid_format: String,
    pub ipv4_expected_octets: String, // "Expected 4 octets, found {}."
    pub ipv4_empty_octet: String,
    pub ipv4_empty_octet_msg: String, // "Octet {} is empty. Check for double dots (..)."
    pub ipv4_octet_too_long: String,
    pub ipv4_octet_too_long_msg: String, // "Octet {} has {} characters, but max is 3."
    pub ipv4_leading_zero: String,
    pub ipv4_leading_zero_msg: String, // "Octet {} has a leading zero ('{}'). Use '{}'."
    pub ipv4_out_of_range: String,
    pub ipv4_out_of_range_msg: String, // "Octet {} is {}, but maximum is 255."
    pub ipv4_all_out_of_range: String,
    pub ipv4_all_out_of_range_msg: String,
    pub ipv4_multiple_out_of_range: String,
    pub ipv4_multiple_out_of_range_msg: String, // "Octets {} exceed the maximum value of 255."
    pub ipv4_invalid_chars: String,
    pub ipv4_invalid_chars_msg: String, // "Octets {} contain non-digit characters."
    
    // IPv6
    pub ipv6_whitespace: String,
    pub ipv6_whitespace_msg: String, // "Remove spaces inside the value."
    pub ipv6_brackets: String,
    pub ipv6_brackets_msg: String, // "Use '::1' instead of '[::1]' in configs."
    pub ipv6_multiple_colons: String,
    pub ipv6_multiple_colons_msg: String, // "Found {} occurrences of '::', but only 1 is allowed."
    pub ipv6_triple_colons: String,
    pub ipv6_triple_colons_msg: String, // "Too many consecutive colons (:::) are not allowed."
    pub ipv6_missing_group: String,
    pub ipv6_missing_group_msg: String, // "Expected {} groups, found {}."
    pub ipv6_too_many_groups: String,
    pub ipv6_too_many_groups_msg: String, // "Expected max 8 groups, found {}. Use '::' to compress zeros."
    pub ipv6_empty_first_group: String,
    pub ipv6_empty_first_group_msg: String, // "Address cannot start with a single ':' (use '::' to compress)."
    pub ipv6_empty_last_group: String,
    pub ipv6_empty_last_group_msg: String, // "Address cannot end with a single ':' (use '::' to compress)."
    pub ipv6_invalid_embedded_ipv4: String,
    // (сообщение берется из IPv4)
    pub ipv6_group_too_long: String,
    pub ipv6_group_too_long_msg: String, // "Group {} has {} hex digits, but maximum is 4."
    pub ipv6_invalid_char: String,
    pub ipv6_invalid_char_msg: String, // "Group {} contains '{}', but only hex digits are allowed."
    pub ipv6_invalid_format: String,
    pub ipv6_invalid_format_msg: String, // "Check the address structure."

    // UUID
    pub uuid_whitespace: String,
    pub uuid_whitespace_msg: String,
    pub uuid_urn: String,
    pub uuid_urn_msg: String, // "Use raw UUID without 'urn:uuid:'"
    pub uuid_braces: String,
    pub uuid_braces_msg: String, // "Use raw UUID without {}"
    pub uuid_invalid_length: String,
    pub uuid_invalid_length_msg: String, // "Expected 36 characters, found {}."
    pub uuid_missing_hyphens: String,
    pub uuid_missing_hyphens_msg: String, // "Expected 4 hyphens, found {}."
    pub uuid_invalid_format: String,
    pub uuid_invalid_format_msg: String, // "Hyphens must be at positions 8, 13, 18, and 23."
    pub uuid_invalid_char: String,
    pub uuid_invalid_char_msg: String, // "Contains non-hexadecimal characters. Only 0-9 and a-f are allowed."

    // MAC
    pub mac_empty: String,
    pub mac_empty_msg: String,
    pub mac_whitespace: String,
    pub mac_whitespace_msg: String,
    pub mac_invalid_char: String,
    pub mac_invalid_char_msg: String,
    pub mac_invalid_length: String,
    pub mac_invalid_length_msg: String, // "Expected 12, 16, 20 or 40 hex digits, found {}."
    pub mac_unseparated_length_msg: String, // "Unseparated MAC must be 12 or 16 hex digits, found {}."
    pub mac_mixed_separators: String,
    pub mac_mixed_separators_msg: String,
    pub mac_invalid_grouping: String,
    pub mac_invalid_grouping_msg: String, // "Expected {} groups, found {}. Example: {}"
    pub mac_empty_group_msg: String, // "MAC address cannot start or end with a separator."
    pub mac_invalid_group_len: String,
    pub mac_invalid_group_len_msg: String, // "Group '{}' has {} chars, but expected {}."
}

pub fn load_locale(lang: &str) -> Locale {
    if lang == "ru" {
        Locale {
            ipv4_invalid_format: "IPv4: неверный формат".to_string(),
            ipv4_expected_octets: "Ожидалось 4 октета, найдено {}.".to_string(),
            ipv4_empty_octet: "IPv4: пустой октет".to_string(),
            ipv4_empty_octet_msg: "Октет {} пуст. Проверьте на двойные точки (..).".to_string(),
            ipv4_octet_too_long: "IPv4: октет слишком длинный".to_string(),
            ipv4_octet_too_long_msg: "Октет {} содержит {} символов, но максимум 3.".to_string(),
            ipv4_leading_zero: "IPv4: ведущий ноль".to_string(),
            ipv4_leading_zero_msg: "Октет {} имеет ведущий ноль ('{}'). Используйте '{}'.".to_string(),
            ipv4_out_of_range: "IPv4: октет вне диапазона".to_string(),
            ipv4_out_of_range_msg: "Октет {} равен {}, но максимум 255.".to_string(),
            ipv4_all_out_of_range: "IPv4: все октеты вне диапазона".to_string(),
            ipv4_all_out_of_range_msg: "Все 4 октета превышают максимальное значение 255.".to_string(),
            ipv4_multiple_out_of_range: "IPv4: несколько октетов вне диапазона".to_string(),
            ipv4_multiple_out_of_range_msg: "Октеты {} превышают максимальное значение 255.".to_string(),
            ipv4_invalid_chars: "IPv4: недопустимый символ".to_string(),
            ipv4_invalid_chars_msg: "Октеты {} содержат нецифровые символы.".to_string(),
            
            ipv6_whitespace: "IPv6: пробелы запрещены".to_string(),
            ipv6_whitespace_msg: "Удалите пробелы внутри значения.".to_string(),
            ipv6_brackets: "IPv6: скобки запрещены".to_string(),
            ipv6_brackets_msg: "Используйте '::1' вместо '[::1]' в конфигах.".to_string(),
            ipv6_multiple_colons: "IPv6: несколько '::'".to_string(),
            ipv6_multiple_colons_msg: "Найдено {} вхождений '::', но разрешено только 1.".to_string(),
            ipv6_triple_colons: "IPv6: неверный формат (:::)".to_string(),
            ipv6_triple_colons_msg: "Слишком много идущих подряд двоеточий (:::).".to_string(),
            ipv6_missing_group: "IPv6: не хватает группы".to_string(),
            ipv6_missing_group_msg: "Ожидалось {} групп, найдено {}.".to_string(),
            ipv6_too_many_groups: "IPv6: слишком много групп".to_string(),
            ipv6_too_many_groups_msg: "Ожидалось максимум 8 групп, найдено {}. Используйте '::' для сжатия нулей.".to_string(),
            ipv6_empty_first_group: "IPv6: пустая первая группа".to_string(),
            ipv6_empty_first_group_msg: "Адрес не может начинаться с одного ':' (используйте '::' для сжатия).".to_string(),
            ipv6_empty_last_group: "IPv6: пустая последняя группа".to_string(),
            ipv6_empty_last_group_msg: "Адрес не может заканчиваться на один ':' (используйте '::' для сжатия).".to_string(),
            ipv6_invalid_embedded_ipv4: "IPv6: неверный вложенный IPv4".to_string(),
            ipv6_group_too_long: "IPv6: группа слишком длинная".to_string(),
            ipv6_group_too_long_msg: "Группа {} содержит {} hex-цифр, но максимум 4.".to_string(),
            ipv6_invalid_char: "IPv6: недопустимый символ".to_string(),
            ipv6_invalid_char_msg: "Группа {} содержит '{}', но разрешены только hex-цифры.".to_string(),
            ipv6_invalid_format: "IPv6: неверный формат".to_string(),
            ipv6_invalid_format_msg: "Проверьте структуру адреса.".to_string(),

            uuid_whitespace: "UUID: пробелы запрещены".to_string(),
            uuid_whitespace_msg: "Удалите пробелы из UUID.".to_string(),
            uuid_urn: "UUID: префикс URN запрещен".to_string(),
            uuid_urn_msg: "Используйте чистый UUID без 'urn:uuid:'".to_string(),
            uuid_braces: "UUID: фигурные скобки запрещены".to_string(),
            uuid_braces_msg: "Используйте чистый UUID без {}".to_string(),
            uuid_invalid_length: "UUID: неверная длина".to_string(),
            uuid_invalid_length_msg: "Ожидалось 36 символов, найдено {}.".to_string(),
            uuid_missing_hyphens: "UUID: не хватает дефисов".to_string(),
            uuid_missing_hyphens_msg: "Ожидалось 4 дефиса, найдено {}.".to_string(),
            uuid_invalid_format: "UUID: неверный формат".to_string(),
            uuid_invalid_format_msg: "Дефисы должны стоять на позициях 8, 13, 18 и 23.".to_string(),
            uuid_invalid_char: "UUID: недопустимый символ".to_string(),
            uuid_invalid_char_msg: "Содержит не-hex-символы. Разрешены только 0-9 и a-f.".to_string(),

            mac_empty: "MAC: пустой адрес".to_string(),
            mac_empty_msg: "MAC-адрес не может быть пустым.".to_string(),
            mac_whitespace: "MAC: содержит пробелы".to_string(),
            mac_whitespace_msg: "Удалите пробелы из MAC-адреса.".to_string(),
            mac_invalid_char: "MAC: недопустимый символ".to_string(),
            mac_invalid_char_msg: "Разрешены только hex-цифры (0-9, a-f).".to_string(),
            mac_invalid_length: "MAC: неверная длина".to_string(),
            mac_invalid_length_msg: "Ожидалось 12, 16, 20 или 40 hex-цифр, найдено {}.".to_string(),
            mac_unseparated_length_msg: "MAC без разделителей должен быть 12 или 16 hex-цифр, найдено {}.".to_string(),
            mac_mixed_separators: "MAC: смешанные разделители".to_string(),
            mac_mixed_separators_msg: "Используйте только один тип разделителя (':', '-' или '.').".to_string(),
            mac_invalid_grouping: "MAC: неверная группировка".to_string(),
            mac_invalid_grouping_msg: "Ожидалось {} групп, найдено {}. Пример: {}".to_string(),
            mac_empty_group_msg: "MAC-адрес не может начинаться или заканчиваться разделителем.".to_string(),
            mac_invalid_group_len: "MAC: неверная длина группы".to_string(),
            mac_invalid_group_len_msg: "Группа '{}' содержит {} символов, но ожидалось {}.".to_string(),
        }
    } else {
        // English (оставляем как было в прошлый раз, но с добавленными _msg)
        Locale {
            ipv4_invalid_format: "IPv4: invalid format".to_string(),
            ipv4_expected_octets: "Expected 4 octets, found {}.".to_string(),
            ipv4_empty_octet: "IPv4: empty octet".to_string(),
            ipv4_empty_octet_msg: "Octet {} is empty. Check for double dots (..).".to_string(),
            ipv4_octet_too_long: "IPv4: octet too long".to_string(),
            ipv4_octet_too_long_msg: "Octet {} has {} characters, but max is 3.".to_string(),
            ipv4_leading_zero: "IPv4: leading zero".to_string(),
            ipv4_leading_zero_msg: "Octet {} has a leading zero ('{}'). Use '{}'.".to_string(),
            ipv4_out_of_range: "IPv4: octet out of range".to_string(),
            ipv4_out_of_range_msg: "Octet {} is {}, but maximum is 255.".to_string(),
            ipv4_all_out_of_range: "IPv4: all octets out of range".to_string(),
            ipv4_all_out_of_range_msg: "All 4 octets exceed the maximum value of 255.".to_string(),
            ipv4_multiple_out_of_range: "IPv4: multiple octets out of range".to_string(),
            ipv4_multiple_out_of_range_msg: "Octets {} exceed the maximum value of 255.".to_string(),
            ipv4_invalid_chars: "IPv4: invalid character".to_string(),
            ipv4_invalid_chars_msg: "Octets {} contain non-digit characters.".to_string(),
            
            ipv6_whitespace: "IPv6: whitespace not allowed".to_string(),
            ipv6_whitespace_msg: "Remove spaces inside the value.".to_string(),
            ipv6_brackets: "IPv6: brackets not allowed".to_string(),
            ipv6_brackets_msg: "Use '::1' instead of '[::1]' in configs.".to_string(),
            ipv6_multiple_colons: "IPv6: multiple '::'".to_string(),
            ipv6_multiple_colons_msg: "Found {} occurrences of '::', but only 1 is allowed.".to_string(),
            ipv6_triple_colons: "IPv6: invalid format (:::)".to_string(),
            ipv6_triple_colons_msg: "Too many consecutive colons (:::) are not allowed.".to_string(),
            ipv6_missing_group: "IPv6: missing group".to_string(),
            ipv6_missing_group_msg: "Expected {} groups, found {}.".to_string(),
            ipv6_too_many_groups: "IPv6: too many groups".to_string(),
            ipv6_too_many_groups_msg: "Expected max 8 groups, found {}. Use '::' to compress zeros.".to_string(),
            ipv6_empty_first_group: "IPv6: empty first group".to_string(),
            ipv6_empty_first_group_msg: "Address cannot start with a single ':' (use '::' to compress).".to_string(),
            ipv6_empty_last_group: "IPv6: empty last group".to_string(),
            ipv6_empty_last_group_msg: "Address cannot end with a single ':' (use '::' to compress).".to_string(),
            ipv6_invalid_embedded_ipv4: "IPv6: invalid embedded IPv4".to_string(),
            ipv6_group_too_long: "IPv6: group too long".to_string(),
            ipv6_group_too_long_msg: "Group {} has {} hex digits, but maximum is 4.".to_string(),
            ipv6_invalid_char: "IPv6: invalid character".to_string(),
            ipv6_invalid_char_msg: "Group {} contains '{}', but only hex digits are allowed.".to_string(),
            ipv6_invalid_format: "IPv6: invalid format".to_string(),
            ipv6_invalid_format_msg: "Check the address structure.".to_string(),

            uuid_whitespace: "UUID: whitespace not allowed".to_string(),
            uuid_whitespace_msg: "Remove spaces from the UUID.".to_string(),
            uuid_urn: "UUID: URN prefix not allowed".to_string(),
            uuid_urn_msg: "Use raw UUID without 'urn:uuid:'".to_string(),
            uuid_braces: "UUID: curly braces not allowed".to_string(),
            uuid_braces_msg: "Use raw UUID without {}".to_string(),
            uuid_invalid_length: "UUID: invalid length".to_string(),
            uuid_invalid_length_msg: "Expected 36 characters, found {}.".to_string(),
            uuid_missing_hyphens: "UUID: missing hyphens".to_string(),
            uuid_missing_hyphens_msg: "Expected 4 hyphens, found {}.".to_string(),
            uuid_invalid_format: "UUID: invalid format".to_string(),
            uuid_invalid_format_msg: "Hyphens must be at positions 8, 13, 18, and 23.".to_string(),
            uuid_invalid_char: "UUID: invalid character".to_string(),
            uuid_invalid_char_msg: "Contains non-hexadecimal characters. Only 0-9 and a-f are allowed.".to_string(),

            mac_empty: "MAC: empty address".to_string(),
            mac_empty_msg: "MAC address cannot be empty.".to_string(),
            mac_whitespace: "MAC: contains whitespace".to_string(),
            mac_whitespace_msg: "Remove spaces from the MAC address.".to_string(),
            mac_invalid_char: "MAC: invalid character".to_string(),
            mac_invalid_char_msg: "Only hex digits (0-9, a-f) are allowed.".to_string(),
            mac_invalid_length: "MAC: invalid length".to_string(),
            mac_invalid_length_msg: "Expected 12, 16, 20 or 40 hex digits, found {}.".to_string(),
            mac_unseparated_length_msg: "Unseparated MAC must be 12 or 16 hex digits, found {}.".to_string(),
            mac_mixed_separators: "MAC: mixed separators".to_string(),
            mac_mixed_separators_msg: "Use only one type of separator (' : ', ' - ' or ' . ').".to_string(),
            mac_invalid_grouping: "MAC: invalid grouping".to_string(),
            mac_invalid_grouping_msg: "Expected {} groups, found {}. Example: {}".to_string(),
            mac_empty_group_msg: "MAC address cannot start or end with a separator.".to_string(),
            mac_invalid_group_len: "MAC: invalid group length".to_string(),
            mac_invalid_group_len_msg: "Group '{}' has {} chars, but expected {}.".to_string(),
        }
    }
}