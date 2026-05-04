use crate::agents::AgentInterface;
use std::{io, io::Write};

pub async fn chat(agent: &mut Box<dyn AgentInterface>) -> anyhow::Result<()> {

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut question = String::new();
        io::stdin().read_line(&mut question)?;
        let question = question.trim();

        if question.starts_with("/history") {
            println!("{:?}", agent.history());
            continue;
        }

        if question.starts_with("/clear"){
            agent.clean_history();
            println!("History clean");
            continue;
        }

        let resposta = agent.stream(question).await;

        if let Err(e) = resposta {
            eprintln!("failed to generate response: {e}");
            continue;
        }
    }
}