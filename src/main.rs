use dotenv_rs::dotenv;
use std::env;
use std::sync::Arc;

use autoagents::core::runtime::Runtime;
use futures_util::StreamExt;

use autoagents::core::agent::task::Task;
use autoagents::core::{
    actor::Topic,
    agent::prebuilt::executor::ReActAgent,
    agent::{ActorAgent, AgentBuilder},
    environment::Environment,
    runtime::SingleThreadedRuntime,
};
use autoagents::llm::backends::ollama::Ollama;
use autoagents::prelude::{Event, TypedRuntime};

mod first_agent;
mod warmup_ollama;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Config
    let base_url = env::var("OLLAMA_HOST_URL")?;
    let model = env::var("MODEL")?;
    let temperature: f32 = env::var("TEMPERATURE")?.parse()?;
    let keep_alive: i32 = env::var("KEEP_ALIVE")?.parse()?;

    // Ollama LLM
    let llm: Arc<Ollama> = Arc::new(Ollama::new(
        base_url.clone(),
        Some(model.clone()),
        None, // model_path
        None, // context_length
        Some(temperature),
        None, // top_p
        None, // system_prompt
        None, // repetition_penalty
        None, // max_tokens
    ));

    // Warmup Ollama
    warmup_ollama::warmup_ollama(&base_url, &model, keep_alive).await?;

    // Runtime + environment
    let runtime = SingleThreadedRuntime::new(None);
    let mut env = Environment::new(None);
    env.register_runtime(runtime.clone()).await?;

    // Topic
    let math_topic = Topic::<Task>::new("math");

    // Build agent and subscribe to the topic
    let react = ReActAgent::new(first_agent::MathAgent::default());
    let _handle = AgentBuilder::<_, ActorAgent>::new(react)
        .llm(llm.clone())
        .runtime(runtime.clone())
        .subscribe(math_topic.clone())
        .build()
        .await?;

    // ✅ Take the event receiver **after agent is fully subscribed**
    let mut events = env.take_event_receiver(None).await?;
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            println!("EVENT: {:?}", event);
            if let Event::TaskComplete { result, .. } = &event {
                println!("🔹 LLM response: {}", result);
            }
        }
    });

    // Publish the task using TypedRuntime
    let typed_runtime = runtime.clone(); // Arc<SingleThreadedRuntime>
    let task = Task::new(
        "You are a math agent. Solve the problem by using the 'addition' tool when needed. \
        Only return JSON with 'value' and 'explanation'. \
        Problem: Add 20 and 5 and explain why.",
    );
    typed_runtime.publish(&math_topic, task).await?;

    // // Log the Ollama request
    // let ollama_request_payload = serde_json::json!({
    //     "model": model,
    //     "messages": [
    //         { "role": "user", "content": &task_prompt }
    //     ]
    // });
    // println!(
    //     "➡ Ollama request payload:\n{}",
    //     serde_json::to_string_pretty(&ollama_request_payload)?
    // );

    // Publish task to runtime
    // let task = Task::new(task_prompt);
    // runtime.publish(&math_topic, task).await?;
    let runtime_ref: &SingleThreadedRuntime = &*runtime;
    runtime_ref.run().await;

    // (&*runtime).run().await;
    println!("Actor system running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;

    Ok(())
}
