mod node;
mod network;
mod chain;
mod synapse;
mod energy;
mod neuron;
mod energy_evolution;
mod api;

use std::sync::Arc;
use tokio::sync::Mutex;
use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::node::Node;
use crate::api::{AppState, create_router};

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.get(1).unwrap_or(&"4000".to_string()).parse().unwrap_or(4000);
    let api_port: u16 = args.get(2).unwrap_or(&"8080".to_string()).parse().unwrap_or(8080);

    println!("🚀 Запуск Organism Node на порту {}", port);

    // === Создаём ноды ===
    let mut node_list = Vec::new();
    for i in 0..3 {
        let node = Arc::new(Mutex::new(Node::new(&format!("Node-{}", i))));
        node_list.push(node);
    }
    let nodes = Arc::new(Mutex::new(node_list));

    // === Запускаем API ===
    let state = AppState::new(nodes.clone());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(state).layer(cors);

    println!("🌐 API доступно на http://127.0.0.1:{}/nodes", api_port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", api_port))
        .await
        .expect("Не удалось запустить API");
    axum::serve(listener, app)
        .await
        .expect("Ошибка при запуске сервера");
}
