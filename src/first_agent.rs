use autoagents::async_trait;
use autoagents::core::agent::prebuilt::executor::{ReActAgent, ReActAgentOutput};
use autoagents::core::tool::{ToolCallError, ToolRuntime};
use autoagents::llm::LLMProvider;
use autoagents::prelude::{
    AgentBuilder, AgentOutputT, DirectAgent, SlidingWindowMemory, Task, ToolInputT, ToolT,
};
use autoagents_derive::{AgentHooks, AgentOutput, agent, tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// TOOL: Addition
#[derive(Serialize, Deserialize, Debug)]
pub struct AddArgs {
    pub left: i64,
    pub right: i64,
}

impl ToolInputT for AddArgs {
    fn io_schema() -> &'static str {
        r#"
        {
            "type": "object",
            "properties": {
                "left": { "type": "integer" },
                "right": { "type": "integer" }
            },
            "required": ["left", "right"]
        }
        "#
    }
}

#[tool(name = "addition", description = "Add two numbers", input = AddArgs)]
pub struct Addition;

#[async_trait]
impl ToolRuntime for Addition {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let a: AddArgs = serde_json::from_value(args)?;
        Ok((a.left + a.right).into())
    }
}

/// AGENT OUTPUT
#[derive(Debug, Serialize, Deserialize, AgentOutput)]
pub struct MathOut {
    #[output(description = "The result value")]
    pub value: i64,
    #[output(description = "Short explanation")]
    pub explanation: String,
}

/// AGENT DEFINITION
#[agent(
    name = "math_agent",
    description = "Solve basic math using tools and return JSON",
    tools = [Addition],
    output = MathOut
)]
#[derive(Clone, AgentHooks, Default)]
pub struct MathAgent {
    pub system_prompt: String,
}

impl MathAgent {
    pub fn new() -> Self {
        Self {
            system_prompt: "You are a math agent. Solve the problem by using the 'addition' tool when needed. Only return JSON with 'value' and 'explanation'.".to_string()
        }
    }

    /// Formats the task with system prompt
    pub fn format_task(&self, user_task: &str) -> String {
        format!("{}\nProblem: {}", self.system_prompt, user_task)
    }
}

impl From<ReActAgentOutput> for MathOut {
    fn from(out: ReActAgentOutput) -> Self {
        serde_json::from_str(&out.response).unwrap_or(MathOut {
            value: 0,
            explanation: out.response,
        })
    }
}
