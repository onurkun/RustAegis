//! Tests for string obfuscation macro
//!
//! These tests verify that the `#[obfuscate_strings]` macro and `aegis_str!`
//! correctly encrypt strings at compile time and decrypt them at runtime.

use aegis_vm::{obfuscate_strings, aegis_str};

#[obfuscate_strings]
fn get_error_message(code: u32) -> &'static str {
    match code {
        1 => "Invalid input provided",
        2 => "Access denied - authentication required",
        3 => "Resource not found",
        4 => "Internal server error",
        _ => "Unknown error occurred",
    }
}

#[obfuscate_strings]
fn get_secret_key() -> &'static str {
    "super_secret_key_12345"
}

#[obfuscate_strings]
fn concat_strings() -> String {
    let prefix = "Error: ";
    let message = "Something went wrong";
    format!("{}{}", prefix, message)
}

#[test]
fn test_error_messages() {
    assert_eq!(get_error_message(1), "Invalid input provided");
    assert_eq!(get_error_message(2), "Access denied - authentication required");
    assert_eq!(get_error_message(3), "Resource not found");
    assert_eq!(get_error_message(4), "Internal server error");
    assert_eq!(get_error_message(99), "Unknown error occurred");
}

#[test]
fn test_secret_key() {
    let key = get_secret_key();
    assert_eq!(key, "super_secret_key_12345");
    assert_eq!(key.len(), 22);
}

#[test]
fn test_concat_strings() {
    let result = concat_strings();
    assert_eq!(result, "Error: Something went wrong");
}

#[test]
fn test_repeated_calls_return_same_value() {
    // Strings are decrypted each time (Box::leak creates new allocation)
    // But values should be identical
    let msg1 = get_error_message(1);
    let msg2 = get_error_message(1);

    // Values should be equal (even if different allocations)
    assert_eq!(msg1, msg2);
}

#[obfuscate_strings]
fn empty_string_handling() -> &'static str {
    let _empty = ""; // Empty strings should be skipped
    "non-empty"
}

#[test]
fn test_empty_string_handling() {
    assert_eq!(empty_string_handling(), "non-empty");
}

#[obfuscate_strings]
fn unicode_strings() -> &'static str {
    "Merhaba Dünya! 🌍 日本語"
}

#[test]
fn test_unicode_strings() {
    let s = unicode_strings();
    assert_eq!(s, "Merhaba Dünya! 🌍 日本語");
    assert!(s.contains("🌍"));
}

#[obfuscate_strings]
fn multiline_string() -> &'static str {
    "This is a
multi-line
string with
several lines"
}

#[test]
fn test_multiline_string() {
    let s = multiline_string();
    assert!(s.contains('\n'));
    assert_eq!(s.lines().count(), 4);
}

// Test that special characters are handled correctly
#[obfuscate_strings]
fn special_chars() -> &'static str {
    "Special: \t\r\n\\\"'`~!@#$%^&*()"
}

#[test]
fn test_special_chars() {
    let s = special_chars();
    assert!(s.contains('\t'));
    assert!(s.contains('\\'));
    assert!(s.contains('"'));
}

// ============================================================================
// aegis_str! macro tests
// ============================================================================

#[test]
fn test_aegis_str_basic() {
    let s = aegis_str!("Hello, World!");
    assert_eq!(s, "Hello, World!");
}

#[test]
fn test_aegis_str_in_match() {
    fn get_message(code: u32) -> &'static str {
        match code {
            1 => aegis_str!("Error: Invalid input"),
            2 => aegis_str!("Error: Access denied"),
            3 => aegis_str!("Error: Not found"),
            _ => aegis_str!("Error: Unknown"),
        }
    }

    assert_eq!(get_message(1), "Error: Invalid input");
    assert_eq!(get_message(2), "Error: Access denied");
    assert_eq!(get_message(3), "Error: Not found");
    assert_eq!(get_message(99), "Error: Unknown");
}

#[test]
fn test_aegis_str_unicode() {
    let s = aegis_str!("Merhaba Dünya! 🌍 日本語");
    assert_eq!(s, "Merhaba Dünya! 🌍 日本語");
}

#[test]
fn test_aegis_str_in_format() {
    let msg = format!("Status: {}", aegis_str!("OK"));
    assert_eq!(msg, "Status: OK");
}

#[test]
fn test_aegis_str_empty() {
    let s = aegis_str!("");
    assert_eq!(s, "");
}

#[test]
fn test_aegis_str_with_escapes() {
    let s = aegis_str!("Line1\nLine2\tTabbed");
    assert!(s.contains('\n'));
    assert!(s.contains('\t'));
}

#[test]
fn test_aegis_str_long_string() {
    let s = aegis_str!("This is a very long string that contains many characters and should still be encrypted and decrypted correctly at runtime without any issues whatsoever.");
    assert!(s.len() > 100);
    assert!(s.starts_with("This is"));
    assert!(s.ends_with("whatsoever."));
}

// Test that same string content produces same result
#[test]
fn test_aegis_str_deterministic() {
    let s1 = aegis_str!("test string");
    let s2 = aegis_str!("test string");
    assert_eq!(s1, s2);
}
