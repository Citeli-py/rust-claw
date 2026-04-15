[README]
# RustClaw 🦀

Um agente multimodal em Rust baseado na LLM Ollama (modelos Qwen3.5:2b).

## Características
- Agente multimodal com streaming de saída
- Integração com terminal via CLI commands
- Suporte a ferramentas customizadas
- Interface interativa por padrão

## Instalação
```bash
cargo run
```

## Uso Interativo

```bash
# Iniciar o agente
./target/debug/ai_agent

# Dica: pressione Enter para continuar após uma resposta
```

## Ferramentas Disponíveis

### + (math/add)
Soma dois números.

### terminal
Executa comandos no terminal Linux.

## Estrutura do Projeto

- [`Cargo.toml`](./Cargo.toml) - Configuração e dependências
- [`src/main.rs`](./src/main.rs) - Lógica do agente principal
Finished  profile [unoptimized + debuginfo] target(s) in 0.31s
