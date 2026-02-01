use serde::{Deserialize, Serialize};

mod initialize;
mod list_tools;
mod tools_call;

pub use initialize::*;
pub use list_tools::*;
pub use tools_call::*;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolOperations {
    Search,
    Batch,
}
