mod node;
mod network;
mod chain;
mod synapse;
mod energy;
mod neuron;
mod energy_evolution;
mod api;
mod interaction;
mod wallet;
mod economy;
mod economy_cycle;
mod brain;


use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
/* use std::time::Duration; */
use axum::{Router}; 
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};  
use tower_http::cors::{CorsLayer, Any};
use interaction::*;
use crate::node::Node;
use crate::api::{AppState, create_router};
use crate::energy_evolution::EnergyEvolution;
use crate::economy::NetworkFund;
use crate::economy_cycle::EconomyCycle;
use crate::brain::Brain;



#[tokio::main]
async fn main() {
    println!("🚀 Запуск системы ORGANISM...");
    env_logger::init();

    // 🧠 Создаём общий канал связи
    let network = Arc::new(NetworkBus::new(100));

    // 🧩 Создаём несколько нод
    let nodes: Vec<Arc<Mutex<Node>>> = (0..5)
        .map(|i| Arc::new(Mutex::new(Node::new(&format!("Node-{}", i)))))
        .collect();

    // Оборачиваем в Arc<Mutex<Vec<...>>> — общий доступ
    let shared_nodes = Arc::new(Mutex::new(nodes));
    let fund = Arc::new(Mutex::new(NetworkFund::new()));

    // 🧬 Запускаем обработчик сообщений
    for node in shared_nodes.lock().await.iter().cloned() {
        let net = network.clone();
        task::spawn(async move {
            loop {
                if let Some(msg) = net.receive().await {
                    handle_message(node.clone(), msg, net.clone()).await;
                }
            }
        });
    }

    // 🔁 Периодическое взаимодействие между нодами
    {
        let net = network.clone();
        let nodes_ref = shared_nodes.clone();
        task::spawn(async move {
            let mut tick = 0;
            loop {
                sleep(Duration::from_secs(5)).await;
                tick += 1;

                let nodes = nodes_ref.lock().await;
                let idx = tick % nodes.len();
                let sender = nodes[idx].lock().await;
                let msg = Message::new(
                    &sender.name,
                    None,
                    MessageType::HelpRequest,
                    0.0,
                    Some("Мне нужна энергия ⚡"),
                );
                println!("📡 {} отправил сигнал помощи", sender.name);
                net.send(msg).await;
            }
        });
    }

    // 🌱 Запускаем фоновую эволюцию
    {
        let evolution_nodes = shared_nodes.clone();
        task::spawn(async move {
            loop {
                {
                    let mut guard = evolution_nodes.lock().await;
                    EnergyEvolution::evolve(&mut guard).await;
                }
                println!("💓 Пульс организма (эволюция прошла)");
                sleep(Duration::from_secs(10)).await;
            }
        });
    }
    

    {
        let nodes_ref = shared_nodes.clone();
        let fund_ref = fund.clone();
        task::spawn(async move {
            loop {  
                {
                    EconomyCycle::run(nodes_ref.clone(), fund_ref.clone()).await;
                }
                tokio::time::sleep(Duration::from_secs(8)).await;
                println!("💫 [DEBUG] Цикл экономики активен...");
            }
        });
    }
    // 🧠 Запускаем сознание (координация между нодами)
    {
        let nodes_ref = shared_nodes.clone();
        let fund_ref = fund.clone();
        task::spawn(async move {
            loop {
                Brain::run(nodes_ref.clone(), fund_ref.clone()).await;
                tokio::time::sleep(Duration::from_secs(12)).await;
                println!("🪶 [DEBUG] Сознание активировано...");
            }
        });
    }
    
    /* {

        let nodes_clone = nodes.clone();
        let fund_clone = fund.clone();
        tokio::spawn(async move {
            EconomyCycle::run(nodes_clone, fund_clone).await;
        });
    }
 */
    // 🌍 API сервер
    let state = api::AppState {
        nodes: shared_nodes.clone(),
        fund: NetworkFund::new(),
    };
    let app: Router = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🌐 API доступно на http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}