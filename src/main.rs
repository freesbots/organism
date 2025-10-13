mod node;   // добавляем модуль node
mod network; // добавляем модуль network
mod chain;
mod synapse;
mod energy;
mod neuron; 
mod energy_evolution;

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
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    let port: u16 = args.get(1).unwrap_or(&"4000".to_string()).parse().unwrap();
    let peer_opt: Option<String> = args.get(2).cloned();
    let api_port: u16 = args.get(3).unwrap_or(&"8080".to_string()).parse().unwrap();

    let peers = if let Some(p) = peer_opt.clone() {
        vec![p]
    } else {
        vec![]
    };

    let node_name = format!("Node-{}", port);
    let node = Arc::new(Mutex::new(Node::new(&node_name)));
    let net = Network::new(port, peers);
 
    let node_for_net = node.clone();

    println!("🚀 Запуск {} на порту {}", node_name, port);
    tokio::spawn(async move {
        net.start(node_for_net).await;
    });

    // === API ===
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = axum::Router::new()
        .route("/nodes", get({
            let node = node.clone();
            move || async move {
                let n = node.lock().await;
                let e = n.energy.lock().unwrap().level; // ✅ берём f64

                let data = vec![serde_json::json!({
                    "name": n.name,
                    "energy": e,
                    "efficiency": n.efficiency,
                    "altruism": n.altruism,
                    "resilience": n.resilience,
                    "experience": n.experience,
                })];
                Json(data)
            }
        }))
        .layer(tower::ServiceBuilder::new().layer(cors));

    println!("🌐 API доступно на http://127.0.0.1:{}/nodes", api_port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", api_port))
        .await
        .expect("Не удалось запустить API");
    axum::serve(listener, app).await.unwrap();
}


async fn get_nodes(state: Arc<Mutex<Vec<Node>>>) -> Json<Vec<NodeView>> {
    let guard = state.lock().await;

    let data: Vec<NodeView> = guard
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let energy = n.energy.lock().unwrap().level; // 🔹 достаём уровень энергии

            NodeView {
                name: n.name.clone(),
                energy, // теперь f64
                efficiency: n.efficiency,
                altruism: n.altruism,
                resilience: n.resilience,
                experience: n.experience,
                x: (i as f64) * 200.0 + 100.0,
                y: ((i as f64).sin() * 150.0 + 300.0),
            }
        })
        .collect();

    Json(data)
}
