use dotenv_rs::dotenv;
use std::env;

mod math_agent;
use autoagents::core::runtime::Runtime;

use autoagents::llm::backends::ollama::Ollama;
use autoagents::llm::builder::LLMBuilder;
// use autoagents::prelude::DirectAgent;

use autoagents::prelude::SlidingWindowMemory;

use crate::math_agent::MathAgent;
use autoagents::core::{
    actor::Topic,
    agent::prebuilt::executor::ReActAgent,
    agent::task::Task,
    agent::{ActorAgent, AgentBuilder},
    environment::Environment,
    runtime::{SingleThreadedRuntime, TypedRuntime},
};
use autoagents::prelude::Event;

use futures_util::StreamExt;
use std::sync::Arc;

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

    // 1) Create runtime and environment
    let runtime = SingleThreadedRuntime::new(None);
    let mut env = Environment::new(None);
    env.register_runtime(runtime.clone()).await?;
    tokio::spawn({
        let runtime = runtime.clone();
        async move {
            runtime.run().await;
        }
    });

    // 2) Build actor agent and subscribe to a topic
    let chat_topic = Topic::<Task>::new("chat");

    let handle = AgentBuilder::<_, ActorAgent>::new(agent)
        .llm(llm.clone())
        .runtime(runtime.clone())
        .memory(Box::new(SlidingWindowMemory::new(10)))
        .subscribe(chat_topic.clone())
        .build()
        .await?;

    println!("handle acquired");

    let mut events = env.take_event_receiver(None).await?;
    println!("event defined");

    tokio::spawn(async move {
        println!("spawned");
        while let Some(event) = events.next().await {
            println!("EVENT: {:?}", event);
            if let Event::TaskComplete { result, .. } = &event {
                println!("🔹 LLM response: {}", result);
            }
        }
    });

    println!("publishing task");

    runtime.publish(&chat_topic, Task::new(prompt)).await?;

    println!("Actor system running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

// OLLAMA_HOST_URL="http://localhost:11434"
// ## OLLAMA_LLM_MODEL="ministral-3:8b"
// ## OLLAMA_LLM_MODEL =  "ministral-3:3b"
// OLLAMA_LLM_MODEL= "llama3.2:3b"
// OLLAMA_TEMPERATURE = 0
// ## PROMPT="add 3.1, 2.9 and explain how you did it, do not calculate yourself, use avaiable tools"
// ## PROMPT = "list available tools"
// PROMPT = "Use the math tool to add 3.1 and 2.9 and return the tool output."
// ## PROMPT = "Thought: I need to add 3.1 and 2.9. I will use the addition tool. Action: addition {\"left\": 3.1, \"right\": 2.9}"
