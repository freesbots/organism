mod node;   // добавляем модуль node
mod network; // добавляем модуль network
mod chain;
mod synapse;
mod energy;
mod neuron; 
mod energy_evolution;
mod api;

use axum::{
    routing::get,
    Json,
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;
use tokio::task;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use network::Network;
use node::Node;  
use api::{create_router, AppState};

#[derive(Serialize)]
struct NodeView {
    name: String,
    energy: f64,
    efficiency: f64,
    altruism: f64,
    resilience: f64,
    experience: f64,
    x: f64,
    y: f64,
}

#[tokio::main]
async fn main() {
    // 🔹 Инициализация списка нод
    let nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>> = Arc::new(Mutex::new(Vec::new()));

    // 🔹 Добавляем несколько тестовых нод
    for i in 0..3 {
        let node = Arc::new(Mutex::new(Node::new(format!("Node-{}", i))));
        nodes.lock().unwrap().push(node);
    }

    // 🔹 Создаём состояние приложения
    let state = AppState { nodes: nodes.clone() };

    // 🔹 Поднимаем HTTP API сервер
    let app = create_router(state);

    println!("🌐 API запущено на http://127.0.0.1:3000");
    Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}