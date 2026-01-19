use crate::math_agent::tool::{Addition, MathOut};
use autoagents::core::agent::prebuilt::executor::ReActAgentOutput;
use autoagents::prelude::AgentOutputT;
use autoagents_derive::{AgentHooks, agent};

#[agent(
    name = "math_agent",
    description = r#"
You are a tool-using agent.

RULES:
- When calling `addition`, you MUST pass JSON with fields:
  { "left": number, "right": number }
- NEVER pass the final answer to a tool.
- After the tool returns, you MUST format the FINAL response
  as JSON matching:
  { "value": number, "explanation": string }
"#,
    tools = [Addition],
    output = MathOut
)]
#[derive(Clone, AgentHooks, Default)]
pub struct MathAgent;

impl From<ReActAgentOutput> for MathOut {
    fn from(out: ReActAgentOutput) -> Self {
        serde_json::from_str(&out.response).unwrap_or({
            MathOut {
                value: 0.0,
                explanation: out.response,
            }
        })
    }
}
