use std::collections::HashMap;

use async_trait::async_trait;

use common::ws::{compute_node::ProcedureUiDescription, value::Value};

#[async_trait]
pub trait Procedure {
    async fn describe(&self, params: HashMap<String, Value>) -> ProcedureUiDescription;
    async fn execute(&self, params: HashMap<String, Value>);
}
