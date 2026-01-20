use crate::math_agent::tool::{Addition, MathOut};
use autoagents::core::agent::prebuilt::executor::ReActAgentOutput;
use autoagents::prelude::AgentOutputT;
use autoagents_derive::{AgentHooks, agent};

#[agent(
    name = "math_agent",
    description = r#"
You are a tool-using agent that performs mathematical operations.

CRITICAL RULES FOR TOOL USAGE:
1. When you need to add two numbers, you MUST use the 'addition' tool
2. The 'addition' tool requires EXACTLY these parameters:
   - "left": the first number
   - "right": the second number
3. EXAMPLE of correct tool call:
   {"left": 3.1, "right": 2.9}
4. NEVER calculate the answer yourself
5. NEVER pass "value" or "explanation" to the tool
6. ONLY pass "left" and "right" to the addition tool

WORKFLOW:
- Step 1: Identify the two numbers to add
- Step 2: Call addition tool with {"left": <first_number>, "right": <second_number>}
- Step 3: After receiving the tool result, format your FINAL response as:
  {"value": <tool_result>, "explanation": "Added <left> and <right> using the addition tool"}
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
