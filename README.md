# RustClaw 🦀

Um agente de IA em Rust com suporte a múltiplos providers, ferramentas customizadas, modo yolo para execução autônoma e streaming de respostas.

## Características

- **Múltiplos providers**: OpenRouter, Gemini, Ollama
- **Modo yolo**: Execução autônoma sem confirmação de ferramentas (`--yolo`)
- **Ferramentas com confirmação humana**: Prompt de aprovação antes de executar comandos
- **Streaming de respostas**: Output em tempo real com suporte a reasoning e tool calls
- **Agentes configuráveis**: Defina provider, modelo, tools e system prompt via arquivos
- **Interface CLI**: Comandos `/history`, `/clear` para gerenciar conversas
- **Testes robustos**: Suite com 25+ testes incluindo regressão para edge cases

## Instalação

```bash
cargo build --release
```

## Uso

### Chat interativo

```bash
# Chat com agente padrão
cargo run chat default

# Chat com modo yolo (sem confirmação de ferramentas)
cargo run chat default --yolo
```

### Criando agentes personalizados

Crie um diretório em `agents/<nome>/` com:

**`config.json`**:
```json
{
  "provider": "openrouter",
  "model": "nvidia/nemotron-3-super-120b-a12b:free",
  "api_key": "sua-api-key",
  "tools": ["terminal"]
}
```

**`PROMPT.md`**:
```markdown
# System Prompt
You are a specialized agent for...
```

Então use:
```bash
cargo run chat <nome-do-agente>
```

### Comandos do chat

| Comando | Descrição |
|---------|-----------|
| `/history` | Mostra o histórico de mensagens |
| `/clear` | Limpa o histórico de conversas |
| `y` / `n` | Confirma ou nega o uso de uma ferramenta |

## Ferramentas Disponíveis

### terminal
Executa comandos no terminal. Retorna `stdout` e `stderr` separados.

## Confirmação de Ferramentas

Por padrão, o agente pede confirmação antes de executar qualquer ferramenta. O comportamento é controlado por `ConfirmationMode`:

- **Ask** (padrão): Pede confirmação ao usuário
- **AlwaysAllow**: Executa sem perguntar (usado em yolo mode)
- **AlwaysDeny**: Bloqueia a execução (usado para testes)

Quando uma ferramenta é bloqueada, o agente recebe `[BLOCKED] Tool execution blocked by user` e ajusta seu comportamento.

## Estrutura do Projeto

```
src/
├── main.rs                    # Entry point e CLI (clap)
├── cli/
│   └── chat.rs               # Loop de chat interativo
├── agents/
│   ├── mod.rs                # Re-exports do módulo
│   ├── config.rs             # Carregamento de config.json e PROMPT.md
│   ├── factory.rs            # Factory para criação de agentes
│   ├── provider.rs           # Match de providers (OpenRouter, Gemini, Ollama)
│   ├── builder.rs            # Builder pattern para agentes
│   ├── wrapper.rs            # Implementação de AgentInterface
│   ├── interface.rs          # Trait AgentInterface
│   └── stream_handler/       # Processamento de streams
│       ├── handler.rs        # Lógica de parsing do stream
│       ├── types.rs          # Tipos (DynStream, UserInterruptionError)
│       └── mod.rs            # Re-exports
└── tools/
    ├── mod.rs                # Re-exports de ferramentas
    ├── terminal.rs           # Ferramenta de terminal
    └── confirmed_tool.rs     # Wrapper com confirmação humana

agents/
├── default/                  # Agente padrão
├── test_agent/               # Agente de testes
└── pr_creator/               # Agente de criação de PRs

tests/
├── agents/                   # Testes de agentes
├── tools/                    # Testes de ferramentas
└── cli/                      # Testes de CLI
```

## Dependências

- [`rig-core`](https://crates.io/crates/rig-core) - Framework de agentes de IA
- [`clap`](https://crates.io/crates/clap) - CLI parsing
- [`tokio`](https://crates.io/crates/tokio) - Runtime assíncrono
- [`serde`](https://crates.io/crates/serde) - Serialização
- [`dotenvy`](https://crates.io/crates/dotenvy) - Variáveis de ambiente

## Rodando testes

```bash
cargo test
```

25 testes passando (exclui chamadas externas flaky à API do Gemini).
