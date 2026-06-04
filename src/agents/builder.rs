use rig::agent::{Agent, AgentBuilder, PromptHook, WithBuilderTools};
use rig::completion::CompletionModel;
use rig::message::Message;
use rig::tool::Tool;

use crate::config::{AgentConfig, ToolConfig};
use crate::agents::wrapper::AgentWrapper;
use crate::AgentInterface;
use crate::tools::confirmed_tool::ConfirmationMode;
use crate::tools::{TerminalTool, WebBrowserTool, WebDriverHandler};

pub(super) async fn build_agent<M, P>(
    builder: AgentBuilder<M, P>,
    config: AgentConfig,
    history: Vec<Message>,
) -> Box<dyn AgentInterface>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{
    let builder = builder
        .preamble(&config.pre_prompt)
        .default_max_turns(20);

    let agent = build_with_tools(builder, config).await;

    Box::new(AgentWrapper::new(agent, history))
}

async fn build_with_tools<M, P>(
    builder: AgentBuilder<M, P>,
    config: AgentConfig,
) -> Agent<M, P>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{
    let mode = if config.yolo { ConfirmationMode::AlwaysAllow } else { ConfirmationMode::Ask };
    let config_path = config.config_path.as_deref();

    let mut builder = Some(builder);
    let mut agent_builder_tool: Option<AgentBuilder<M, P, WithBuilderTools>> = None;

    for tool_cfg in config.tools_config.tools {
        match tool_cfg {
            ToolConfig::Terminal(cfg) => {
                let tool = TerminalTool::build(cfg, mode, config_path);
                agent_builder_tool = apply_tool(&mut builder, agent_builder_tool, tool);
            }
            ToolConfig::WebBrowser(cfg) => {
                let tool = WebBrowserTool::<WebDriverHandler>::build(cfg, mode, config_path).await;
                agent_builder_tool = apply_tool(&mut builder, agent_builder_tool, tool);
            }
        }
    }

    match agent_builder_tool {
        Some(b) => b.build(),
        None => builder.unwrap().build(),
    }
}

fn apply_tool<M, P, T>(
    builder: &mut Option<AgentBuilder<M, P>>,
    agent_builder_tool: Option<AgentBuilder<M, P, WithBuilderTools>>,
    tool: T,
) -> Option<AgentBuilder<M, P, WithBuilderTools>>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
    T: Tool + 'static,
{
    Some(match agent_builder_tool {
        Some(b) => b.tool(tool),
        None => builder.take().expect("builder já foi usado").tool(tool),
    })
}
