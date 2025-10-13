use axum::{
    extract::{State, Path},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::join_all;

use crate::node::Node;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
}

#[derive(Serialize)]
struct NodeInfo {
    name: String,
    energy: f64,
    efficiency: f64,
    altruism: f64,
    resilience: f64,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/nodes", get(get_nodes))
        .route("/mine/:id", post(mine_block))
        .route("/chain/:id", get(get_chain))
        .route("/update/:id", post(update_node))
        .with_state(state)
}

async fn root() -> &'static str {
    "🧬 Organism API is running"
}
async fn get_nodes(State(state): State<AppState>) -> Json<Vec<NodeInfo>> {
    let nodes = state.nodes.lock().await;

    // создаём список асинхронных задач
    let tasks = nodes.iter().map(|n| {
        let n = n.clone();
        async move {
            let n = n.lock().await;
            let energy = n.energy.lock().await;

            NodeInfo {
                name: n.name.clone(),
                energy: energy.level,
                efficiency: n.efficiency,
                altruism: n.altruism,
                resilience: n.resilience,
            }
        }
    });

    // ждём выполнения всех задач параллельно
    let infos = join_all(tasks).await;

    Json(infos)
}


/// Смоделировать "добычу" блока у конкретной ноды
async fn mine_block(State(state): State<AppState>, Path(id): Path<usize>) -> Json<String> {
    let nodes = state.nodes.lock().await;
    if let Some(node) = nodes.get(id) {
        let mut n = node.lock().await;
        n.mine_block(); // метод должен быть sync
        Json(format!("✅ Node {} mined a block", n.name))
    } else {
        Json("❌ Node not found".to_string())
    }
}

/// Получить цепочку блоков
async fn get_chain(State(state): State<AppState>) -> Json<Vec<String>> {
    let nodes = state.nodes.lock().await;
    if let Some(node) = nodes.first() {
        let n = node.lock().await;
        let chain = n.get_chain_summary().await; // ✅ добавляем await
        Json(chain)
    } else {
        Json(vec![])
    }
}

/// Обновить ноду (например, перерасчёт параметров)
#[derive(Deserialize)]
struct UpdateRequest {
    energy: Option<f64>,
    efficiency: Option<f64>,
    altruism: Option<f64>,
    resilience: Option<f64>,
}

async fn update_node(
    State(state): State<AppState>,
    Path(id): Path<usize>,
    Json(payload): Json<UpdateRequest>,
) -> Json<String> {
    let nodes = state.nodes.lock().await;
    if let Some(node) = nodes.get(id) {
        let mut n = node.lock().await;
        if let Some(e) = payload.energy {
            let mut energy = n.energy.lock().await;
            energy.level = e;
        }
        if let Some(v) = payload.efficiency {
            n.efficiency = v;
        }
        if let Some(v) = payload.altruism {
            n.altruism = v;
        }
        if let Some(v) = payload.resilience {
            n.resilience = v;
        }
        Json(format!("✅ Node {} updated", n.name))
    } else {
        Json("❌ Node not found".to_string())
    }
}

impl AppState {
    pub fn new(nodes: Arc<tokio::sync::Mutex<Vec<Arc<tokio::sync::Mutex<Node>>>>>) -> Self {
        Self { nodes }
    }
}