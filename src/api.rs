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
use crate::wallet::Wallet;
use crate::node::Node;
use crate::economy::NetworkFund;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
    pub fund: NetworkFund,
}

#[derive(Serialize)]
struct NodeInfo {
    name: String,
    energy: f64,
    balance: f64,
    efficiency: f64,
    altruism: f64,
    resilience: f64,
}

#[derive(Serialize)]
pub struct WalletInfo {
    pub name: String,
    pub balance: f64,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/nodes", get(get_nodes))
        .route("/mine/:id", post(mine_block))
        .route("/chain/:id", get(get_chain))
        .route("/update/:id", post(update_node))
        .route("/wallets", get(get_wallets))
        .with_state(state)
}

async fn root() -> &'static str {
    "🧬 Organism API is running"
}

pub async fn get_wallets(State(state): State<AppState>) -> Json<Vec<WalletInfo>> {
    let nodes = state.nodes.lock().await;
    let mut infos = Vec::new();

    for n in nodes.iter() {
        let node = n.lock().await;
        let balance = *node.wallet.balance.lock().await;
        infos.push(WalletInfo {
            name: node.name.clone(),
            balance,
        });
    }

    Json(infos)
}

async fn get_nodes(State(state): State<AppState>) -> Json<Vec<NodeInfo>> {
    let nodes = state.nodes.lock().await;

    let tasks = nodes.iter().map(|n| {
        let n = n.clone();
        async move {
            let node = n.lock().await; 
            let energy_guard = node.energy.lock().await;
            let balance_guard = node.wallet.balance.lock().await;

            NodeInfo {
                name: node.name.clone(),
                energy: energy_guard.level,
                balance: *balance_guard,
                efficiency: node.efficiency,
                altruism: node.altruism,
                resilience: node.resilience,
            }
        }
    });

    let infos = join_all(tasks).await;
    Json(infos)
}

/// Смоделировать "добычу" блока у конкретной ноды
async fn mine_block(State(state): State<AppState>, Path(id): Path<usize>) -> Json<String> {
    let nodes = state.nodes.lock().await;
    if let Some(node) = nodes.get(id) {
        let mut n = node.lock().await;

        // ⛏️ Симуляция майнинга блока
        let reward = 15.0;
        let validator_cut = 3.0;
        let fund_cut = 2.0;

        // 💰 Майнер получает вознаграждение
        n.wallet.reward(reward).await;

        // 🔍 Выбираем случайного валидатора
        if let Some(validator) = nodes.get(rand::random::<usize>() % nodes.len()) {
            let v = validator.lock().await;
            v.wallet.reward(validator_cut).await;
        }

        // 🏦 Добавляем в фонд
        state.fund.add(fund_cut);

        let response = format!(
            "⛏️ Блок добыт нодой {}: +{:.2} токенов, фонд +{:.2}",
            n.name, reward, fund_cut
        );
        return Json(response);
    }

    Json("❌ Нода не найдена".to_string())
}

/// Получить цепочку блоков
async fn get_chain(State(state): State<AppState>) -> Json<Vec<String>> {
    let nodes = state.nodes.lock().await;
    if let Some(node) = nodes.first() {
        let n = node.lock().await;
        let chain = n.get_chain_summary().await;
        Json(chain)
    } else {
        Json(vec![])
    }
}

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

        let fee = 1.0;
        let fund_cut = 0.5;
        state.fund.add(fund_cut);
        n.wallet.reward(fee - fund_cut).await;

        Json(format!("✅ Node {} updated", n.name))
    } else {
        Json("❌ Node not found".to_string())
    }
}
