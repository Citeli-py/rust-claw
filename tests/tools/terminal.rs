use ai_agent::config::TerminalConfig;
use ai_agent::tools::confirmed_tool::ConfirmationMode;
use ai_agent::tools::terminal::{TerminalTool, TerminalArgs};
use rig::tool::Tool;


// ✅ Teste: comando simples funciona
#[tokio::test]
async fn test_terminal_echo() {
    let tool = TerminalTool;

    let args = TerminalArgs {
        command: "echo hello".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.stdout.contains("hello"));
    assert_eq!(result.stderr, "");
}

// ✅ Teste: comando inválido gera stderr
#[tokio::test]
async fn test_invalid_command() {
    let tool = TerminalTool;

    let args = TerminalArgs {
        command: "comando_que_nao_existe_123".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    // bash normalmente escreve erro no stderr
    assert!(!result.stderr.is_empty());
}

// ✅ Teste: stdout e stderr separados corretamente
#[tokio::test]
async fn test_stdout_and_stderr() {
    let tool = TerminalTool;

    let args = TerminalArgs {
        command: "echo out && echo err 1>&2".to_string(),
    };

    let result = tool.call(args).await.unwrap();

    assert!(result.stdout.contains("out"));
    assert!(result.stderr.contains("err"));
}

// ✅ Teste: comando sem output
#[tokio::test]
async fn test_empty_output() {
    let tool = TerminalTool;

    let args = TerminalArgs {
        command: "true".to_string(), // comando que não retorna nada
    };

    let result = tool.call(args).await.unwrap();

    assert_eq!(result.stdout, "");
    assert_eq!(result.stderr, "");
}

mod build {
    use super::*;

    fn cfg(trusted: Vec<&str>) -> TerminalConfig {
        TerminalConfig { trusted: trusted.iter().map(|s| s.to_string()).collect() }
    }

    #[test]
    fn test_build_sets_tool_name() {
        let tool = TerminalTool::build(cfg(vec![]), ConfirmationMode::AlwaysAllow, None);
        assert_eq!(tool.tool_name, "terminal");
    }

    #[tokio::test]
    async fn test_build_always_allow_runs_command() {
        let tool = TerminalTool::build(cfg(vec![]), ConfirmationMode::AlwaysAllow, None);
        let result = tool.call(TerminalArgs { command: "echo hello".to_string() }).await;
        assert!(result.is_ok());
        assert!(result.unwrap().stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_build_always_deny_blocks_command() {
        let tool = TerminalTool::build(cfg(vec![]), ConfirmationMode::AlwaysDeny, None);
        let result = tool.call(TerminalArgs { command: "echo hello".to_string() }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked"));
    }

    #[tokio::test]
    async fn test_build_trusted_command_bypasses_confirmation() {
        let trusted_json = r#"{"command":"echo hello"}"#;
        let tool = TerminalTool::build(cfg(vec![trusted_json]), ConfirmationMode::Ask, None);
        let result = tool.call(TerminalArgs { command: "echo hello".to_string() }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_untrusted_command_is_blocked_in_ask_mode() {
        let tool = TerminalTool::build(cfg(vec![]), ConfirmationMode::AlwaysDeny, None);
        let result = tool.call(TerminalArgs { command: "echo hello".to_string() }).await;
        assert!(result.is_err());
    }
}
