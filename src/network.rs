use crate::node::Node;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;  
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct Network {
    pub port: u16,
    pub peers: Vec<String>,
} 

impl Network {
    pub fn new(port: u16, peers: Vec<String>) -> Self {
        Network { port, peers }
    }

    pub async fn send_to_peer(&self, peer: &str, data: String) {
        if let Ok(mut stream) = TcpStream::connect(peer).await {
            if stream.write_all(data.as_bytes()).await.is_ok() {
                println!("📤 Отправлен блок -> {}", peer);
            }
        }
    }

    /// 🔥 Теперь принимаем node как Arc<Mutex<Node>>
     pub async fn start(&self, node: Arc<Mutex<Node>>) { 
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .expect("❌ Не удалось запустить TCP сервер");

        println!("🌐 Нода слушает на порту {}", self.port);

        // Обработка входящих соединений
        let node_clone = node.clone();
        tokio::spawn(async move {
            loop {
                if let Ok((socket, _)) = listener.accept().await {
                    let node_for_client = node_clone.clone();
                    tokio::spawn(async move {
                        handle_connection(socket, node_for_client).await;
                    });
                }
            }
        });

        // === Поток для обучения + синхронизации нейронов ===
        let peers = self.peers.clone();
        tokio::spawn({
            let node_ref = node.clone();
            async move {
                loop {
                    {
                        let mut n = node_ref.lock().await;
                        let (data_root, key_root) = n.mine_data();
                        n.try_commit_keyblock(data_root, key_root);
                    }

                    // Отправляем нейроны всем пирами
                    for peer in &peers {
                        let n = node_ref.lock().await;
                        let neurons_json = n.export_neurons_json();
                        drop(n);

                        let msg = json!({
                            "type": "neurons_sync",
                            "data": neurons_json
                        })
                        .to_string();

                        if let Ok(mut stream) = TcpStream::connect(peer).await {
                            let _ = stream.write_all(msg.as_bytes()).await;
                            println!("🔁 Отправлены нейроны → {}", peer);
                        }
                    }

                    sleep(Duration::from_secs(10)).await;
                }
            }
        });
 
    }
}

async fn handle_connection(mut socket: TcpStream, node: Arc<Mutex<Node>>) {
    let mut buffer = vec![0u8; 4096];
    if let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            return;
        }

        let data = String::from_utf8_lossy(&buffer[..n]);
        if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&data) {
            if json_msg["type"] == "neurons_sync" {
                let neurons_json = json_msg["data"].as_str().unwrap_or("");
                let mut n = node.lock().await;
                n.import_neurons_json(neurons_json);
                println!("🧠 Получена синхронизация нейронов");
                return;
            }
        }
    }
         
}