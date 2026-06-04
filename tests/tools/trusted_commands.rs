use std::collections::HashMap;

use ai_agent::tools::{TrustedCommands, TrustedCommandsInterface};

fn make_trusted(tool: &str, commands: Vec<&str>) -> TrustedCommands {
    let mut map = HashMap::new();
    map.insert(tool.to_string(), commands.iter().map(|s| s.to_string()).collect());
    TrustedCommands::new(tool, map)
}

#[test]
fn test_empty_map_returns_false() {
    let t = TrustedCommands::new("tool", HashMap::new());
    assert!(!t.is_trusted(r#"{"action":"call"}"#));
}

#[test]
fn test_exact_match_returns_true() {
    let t = make_trusted("tool", vec![r#"{"action":"call"}"#]);
    assert!(t.is_trusted(r#"{"action":"call"}"#));
}

#[test]
fn test_pretty_printed_matches_compact() {
    // o confirmed_tool serializa com to_string_pretty; o config armazena compacto
    let t = make_trusted("tool", vec![r#"{"command":"free -h"}"#]);
    let pretty = "{\n  \"command\": \"free -h\"\n}";
    assert!(t.is_trusted(pretty));
}

#[test]
fn test_spaces_inside_values_are_preserved() {
    // espaços dentro do valor da string NÃO devem ser removidos
    let t = make_trusted("tool", vec![r#"{"command":"free -h"}"#]);
    assert!(!t.is_trusted(r#"{"command":"free-h"}"#));
}

#[test]
fn test_different_command_returns_false() {
    let t = make_trusted("tool", vec![r#"{"action":"call"}"#]);
    assert!(!t.is_trusted(r#"{"action":"other"}"#));
}

#[test]
fn test_commands_for_different_tool_returns_false() {
    let mut map = HashMap::new();
    map.insert("tool_a".to_string(), vec![r#"{"action":"call"}"#.to_string()]);
    let t = TrustedCommands::new("tool_b", map);
    assert!(!t.is_trusted(r#"{"action":"call"}"#));
}

#[test]
fn test_multiple_trusted_commands() {
    let t = make_trusted("tool", vec![r#"{"action":"a"}"#, r#"{"action":"b"}"#]);
    assert!(t.is_trusted(r#"{"action":"a"}"#));
    assert!(t.is_trusted(r#"{"action":"b"}"#));
    assert!(!t.is_trusted(r#"{"action":"c"}"#));
}
