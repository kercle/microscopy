use std::{collections::HashMap, sync::Arc};

use common::ws::compute_node::{ComputeNode, ComputeNodeCapabilities};
use tokio::sync::RwLock;

pub type ComputeNodeContainer = Arc<RwLock<HashMap<String, ComputeNodeCapabilities>>>;

pub trait ComputeNodeContainerExt {
    async fn register(
        &self,
        capabilities: &ComputeNodeCapabilities,
    ) -> (String, Vec<ComputeNode>);
    async fn unregister(&self, node_id: &str) -> Vec<ComputeNode>;
    async fn list(&self) -> Vec<ComputeNode>;
}

impl ComputeNodeContainerExt for ComputeNodeContainer {
    async fn register(
        &self,
        capabilities: &ComputeNodeCapabilities,
    ) -> (String, Vec<ComputeNode>) {
        let node_id = uuid::Uuid::new_v4().to_string();

        let mut compute_nodes = self.write().await;
        compute_nodes.insert(node_id.clone(), capabilities.clone());
        (node_id, list_compute_nodes(&compute_nodes))
    }

    async fn unregister(&self, node_id: &str) -> Vec<ComputeNode> {
        let mut compute_nodes = self.write().await;
        compute_nodes.remove(node_id);
        list_compute_nodes(&compute_nodes)
    }

    async fn list(&self) -> Vec<ComputeNode> {
        let compute_nodes = self.read().await;
        list_compute_nodes(&compute_nodes)
    }
}

fn list_compute_nodes(
    compute_nodes: &HashMap<String, ComputeNodeCapabilities>,
) -> Vec<ComputeNode> {
    compute_nodes
        .iter()
        .map(|(node_id, capabilities)| ComputeNode {
            node_id: node_id.clone(),
            capabilities: capabilities.clone(),
        })
        .collect()
}
