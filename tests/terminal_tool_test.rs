use ai_agent::tools::terminal::{TerminalTool, TerminalArgs, TerminalOutput};
use rig::completion::ToolDefinition;
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
