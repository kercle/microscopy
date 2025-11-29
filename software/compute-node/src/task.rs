use std::collections::HashMap;

use async_trait::async_trait;

use common::ws::{compute_node::TaskUiDescription, value::Value};

#[async_trait]
pub trait Task {
    async fn describe(&self, params: HashMap<String, Value>) -> TaskUiDescription;
    async fn execute(&self, params: HashMap<String, Value>);
}
