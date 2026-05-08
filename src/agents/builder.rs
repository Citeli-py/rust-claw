use rig::{agent::{Agent, AgentBuilder, PromptHook, WithBuilderTools}, completion::CompletionModel, message::Message};
use rig::tool::Tool;

use crate::agents::wrapper::AgentWrapper;
use crate::tools::*;
use crate::tools::confirmed_tool::{ConfirmedTool, ConfirmationMode};

pub(super) async fn build_agent<M, P>(builder: AgentBuilder<M, P>, pre_prompt: &str, history: Vec<Message>, yolo: bool, tools: Vec<String>) -> Box<dyn crate::agents::interface::AgentInterface>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{

    let builder = builder
    .preamble(pre_prompt)
    .default_max_turns(20);

    let agent = build_with_tools(builder, tools, yolo).await;

    Box::new(AgentWrapper::new(agent, history))
}

pub(super) async fn build_with_tools<M, P>(builder: AgentBuilder<M, P>, tools: Vec<String>, yolo: bool) -> Agent<M, P>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{

    let confirmation_mode = if yolo {ConfirmationMode::AlwaysAllow} else {ConfirmationMode::Ask};

    let mut builder = Some(builder);
    let mut agent_builder_tool: Option<AgentBuilder<M, P, WithBuilderTools>> = None;

    for tool_name in tools {
        match tool_name.as_str() {
            "terminal" => {
                let mut tool = ConfirmedTool::new(TerminalTool);
                tool.set_mode(confirmation_mode);
                agent_builder_tool = apply_tool(&mut builder, agent_builder_tool, tool);
            }
            "web_browser" => {
                let web_driver = WebDriverHandler::new(false).await.unwrap();
                let mut tool = ConfirmedTool::new(WebBrowserTool::new(web_driver));
                tool.set_mode(confirmation_mode);
                agent_builder_tool = apply_tool(&mut builder, agent_builder_tool, tool);
            }
            _ => println!("Unknown tool: {}", tool_name),
        }
    }

    match agent_builder_tool {
        Some(agent_builder) => agent_builder.build(),
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
        Some(agent) => agent.tool(tool),
        None => {
            let b = builder.take().expect("builder já foi usado");
            b.tool(tool)
        }
    })
}
