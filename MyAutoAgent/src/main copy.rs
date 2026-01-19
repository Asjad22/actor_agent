use dotenv_rs::dotenv;
use std::env;

mod math_agent;

use autoagents::llm::backends::ollama::Ollama;
use autoagents::llm::builder::LLMBuilder;
use autoagents::prelude::DirectAgent;
use autoagents::prelude::SlidingWindowMemory;

use autoagents::core::{
    actor::Topic,
    agent::prebuilt::executor::ReActAgent,
    agent::task::Task,
    agent::{ActorAgent, AgentBuilder},
    environment::Environment,
    runtime::{SingleThreadedRuntime, TypedRuntime},
};
use std::sync::Arc;

use crate::math_agent::MathAgent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let ollama_host_url: String =
        env::var("OLLAMA_HOST_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model: String =
        env::var("OLLAMA_LLM_MODEL").unwrap_or_else(|_| "llama3.2:4b".to_string());
    let ollama_temperature: f32 = {
        env::var("OLLAMA_TEMPERATURE")
            .unwrap_or("0.7".to_string())
            .parse()
            .unwrap_or(0.7)
    };
    let prompt: String = env::var("PROMPT").unwrap_or_else(|_| "hi".to_string());

    let llm: Arc<Ollama> = LLMBuilder::<Ollama>::new()
        .base_url(ollama_host_url)
        .model(ollama_model)
        .temperature(ollama_temperature)
        .build()?;

    let agent = ReActAgent::new(MathAgent);
    let handle = AgentBuilder::<_, DirectAgent>::new(agent)
        .llm(llm)
        .memory(Box::new(SlidingWindowMemory::new(10)))
        .build()
        .await?;

    let out = handle.agent.run(Task::new(prompt)).await?;
    println!("{}", out);

    Ok(())
}
