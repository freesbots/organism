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
mod memory;


use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
/* use std::time::Duration; */
use axum::{Router}; 
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};  
use tower_http::cors::{CorsLayer, Any};
use tokio::sync::RwLock;
use interaction::*;
use crate::node::Node;
use crate::api::{AppState, create_router};
use crate::energy_evolution::EnergyEvolution;
use crate::economy::NetworkFund;
use crate::economy_cycle::EconomyCycle; 
use crate::brain::{Brain, BrainSnapshot}; 
use rand::thread_rng;
use chrono::Utc;




#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("🚀 Запуск системы ORGANISM...");
    env_logger::init();

    // 🧠 Создаём общий канал связи
    let network = Arc::new(NetworkBus::new(100));

    let count = 10;
    // 🧩 Создаём несколько нод
   let nodes: Vec<_> = (0..count)
    .map(|i| Node::new(&format!("node{}", i)))
    .collect();

    // Оборачиваем в Arc<Mutex<Vec<...>>> — общий доступ
    let shared_nodes = Arc::new(Mutex::new(nodes));
     // ✅ создаём общий фонд
    let fund = Arc::new(Mutex::new(NetworkFund::new()));

    // ✅ создаём мозг
    let brain = Arc::new(RwLock::new(Brain::new()));

    let snapshot = Arc::new(RwLock::new(BrainSnapshot {
        aggressiveness: 1.0,
        avg_recent_result: 0.0,
        recent_memory: Vec::new(),
        last_update: chrono::Utc::now().timestamp(),
    }));
 

    // ✅ создаём клоны для потоков
    let nodes_ref = Arc::clone(&shared_nodes);
    let fund_ref = Arc::clone(&fund);
    let brain_ref: Arc<RwLock<Brain>> = Arc::clone(&brain);
    let brain_clone: Arc<RwLock<Brain>> = Arc::clone(&brain);
    let snapshot_ref: Arc<RwLock<BrainSnapshot>> = Arc::clone(&snapshot);
    let snapshot_task_ref = Arc::clone(&snapshot_ref);

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
        let net = network.clone();
        println!("🧠 Инициализация мозга");

        // === Задача 1: основной цикл мозга ===
        let brain_clone_for_run = Arc::clone(&brain_clone);
        let nodes_ref_clone = Arc::clone(&nodes_ref);
        let fund_ref_clone = Arc::clone(&fund_ref);
        let net_clone = Arc::clone(&net);
        println!("🧠 [INIT] Запуск основного цикла мозга...");  

        // === Запускаем run() для когнитивной активности мозга ===
        let brain_for_run_task = Arc::clone(&brain_clone_for_run);
        let nodes_for_run_task = Arc::clone(&nodes_ref_clone);
        let fund_for_run_task = Arc::clone(&fund_ref_clone);
        let net_for_run_task = Arc::clone(&net_clone);
        /* tokio::spawn(async move {
            let mut brain_guard = brain_for_run_task.write().await;
            brain_guard.run(nodes_for_run_task, fund_for_run_task, net_for_run_task).await;
        }); */
        tokio::spawn(async move {
            loop {
                {
                    // 🔹 Берём brain на короткий момент
                    let brain_arc = brain_for_run_task.clone();
                    let mut brain_guard = brain_arc.write().await;

                    // 🔹 Клонируем внутреннее состояние, чтобы освободить блокировку
                    let mut brain_copy = brain_guard.clone();
                    drop(brain_guard); // <— снимаем lock прямо здесь!

                    // 🔹 Теперь можно вызывать .await безопасно — brain не заблокирован
                    brain_copy.run(nodes_for_run_task.clone(), fund_for_run_task.clone(), net_for_run_task.clone()).await;
                }

                // 🔁 Небольшая пауза между циклами
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

        tokio::spawn(async move {
            let mut brain_copy = {
                let brain_ref = brain_clone_for_run.read().await;
                brain_ref.clone() // ✅ клонируем сам Brain, не guard
            };
            brain_copy.evolve_network(nodes_ref_clone, fund_ref_clone, net_clone).await;
        });
        // === Задача 2: периодическое обновление снимка состояния ===
        let brain_for_snapshot = Arc::clone(&brain_clone);
        let snapshot_task_ref_clone = Arc::clone(&snapshot_task_ref);
        println!("🧠 [DEBUG] Инициализация таски snapshot...");
       tokio::spawn(async move {
        println!("📸 [DEBUG] Snapshot task started!");
            // внутри snapshot таски
            loop {
                let snapshot = BrainSnapshot::from_brain_lock(&brain_for_snapshot).await;

                {
                    let mut snap = snapshot_task_ref_clone.write().await;
                    *snap = snapshot.clone();
                }

                println!(
                    "📸 [Snapshot] обновлён: память = {} событий, агрессивность = {:.2}",
                    snapshot.recent_memory.len(),
                    snapshot.aggressiveness
                );

                // ⏸️ обновляем раз в 10 секунд (а не в 1)
                tokio::time::sleep(Duration::from_secs(10)).await;
            }

        });
        
    }
    
    
    // 🌍 API сервер
    let state = api::AppState {
        nodes: shared_nodes.clone(),
        fund: Arc::clone(&fund),
        brain: brain_ref.clone(),
        snapshot: snapshot_ref.clone(),
    };
    let app: Router = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🌐 API доступно на http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}