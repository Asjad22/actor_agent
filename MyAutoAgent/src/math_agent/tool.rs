use autoagents::async_trait;
use autoagents::core::tool::{ToolCallError, ToolInputT, ToolRuntime, ToolT};
use autoagents_derive::{ToolInput, tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, ToolInput, Debug)]
pub struct AddArgs {
    #[input(description = "Left operand for addition")]
    pub left: f64,
    #[input(description = "Right operand for addition")]
    pub right: f64,
}

#[tool(name = "addition", description = "Add two numbers", input = AddArgs)]
pub struct Addition;

#[async_trait]
impl ToolRuntime for Addition {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let a: AddArgs = serde_json::from_value(args)?;
        println!("Adding a.left + a.right");
        Ok((a.left + a.right).into())
    }
}

use autoagents_derive::AgentOutput;
#[derive(Debug, Serialize, Deserialize, AgentOutput)]
pub struct MathOut {
    #[output(description = "The result value")]
    pub value: f64,
    #[output(description = "Short explanation")]
    pub explanation: String,
}

impl std::fmt::Display for MathOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tool Result: {}\nExplanation:\n{}",
            self.value, self.explanation
        )
    }
}
