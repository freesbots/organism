use axum::{
    routing::{get, post},
    Router, Json,
    extract::State,
};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use crate::node::Node;
use crate::chain::Chain;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
}

#[derive(Serialize)]
struct NodeInfo {
    name: String,
    energy: f64,
    chain_length: usize,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/nodes", get(get_nodes))
        .route("/mine", post(mine_block))
        .route("/sync", post(sync_chains))
        .with_state(state)
}

async fn get_nodes(State(state): State<AppState>) -> Json<Vec<NodeInfo>> {
    let nodes = state.nodes.lock().unwrap();
    let info = nodes
        .iter()
        .map(|n| {
            let node = n.lock().unwrap();
            let e = node.energy.lock().unwrap();
            NodeInfo {
                name: node.name.clone(),
                energy: e.level,
                chain_length: node.chain.lock().unwrap().blocks.len(),
            }
        })
        .collect();
    Json(info)
}

#[derive(Deserialize)]
struct MineRequest {
    node_index: usize,
}

async fn mine_block(State(state): State<AppState>, Json(req): Json<MineRequest>) -> Json<String> {
    let nodes = state.nodes.lock().unwrap();
    if let Some(node) = nodes.get(req.node_index) {
        let mut n = node.lock().unwrap();
        n.mine_block();
        Json(format!("Node {} mined a new block", n.name))
    } else {
        Json("Invalid node index".to_string())
    }
}

async fn sync_chains(State(state): State<AppState>) -> Json<String> {
    let mut nodes = state.nodes.lock().unwrap();

    if nodes.is_empty() {
        return Json("No nodes available".to_string());
    }

    let longest_chain = {
        let mut max_chain: Option<Chain> = None;
        for n in nodes.iter() {
            let chain = n.lock().unwrap().chain.lock().unwrap().clone();
            if max_chain
                .as_ref()
                .map_or(true, |c| chain.blocks.len() > c.blocks.len())
            {
                max_chain = Some(chain);
            }
        }
        max_chain
    };

    if let Some(chain) = longest_chain {
        for n in nodes.iter() {
            n.lock().unwrap().chain.lock().unwrap().replace_with(chain.clone());
        }
        Json("All nodes synchronized".to_string())
    } else {
        Json("Failed to sync chains".to_string())
    }
}
