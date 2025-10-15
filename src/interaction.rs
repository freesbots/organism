use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::node::Node;

/// Типы сообщений между нодами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    HelpRequest,       // просьба о помощи (энергии)
    EnergyTransfer,    // передача энергии
    BlockAnnouncement, // новый блок найден
    ValidateBlock,     // запрос на валидацию
}

/// Сообщение, пересылаемое между нодами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: Option<String>, // None = широковещательно
    pub msg_type: MessageType,
    pub value: f64,         // энергия или значимость
    pub content: Option<String>,
}

impl Message {
    pub fn new(from: &str, to: Option<&str>, msg_type: MessageType, value: f64, content: Option<&str>) -> Self {
        Self {
            from: from.to_string(),
            to: to.map(|s| s.to_string()),
            msg_type,
            value,
            content: content.map(|s| s.to_string()),
        }
    }
}

/// Каналы связи между нодами
pub struct NetworkBus {
    pub sender: mpsc::Sender<Message>,
    pub receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
}

impl NetworkBus {
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer);
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    /// Отправить сообщение
    pub async fn send(&self, msg: Message) {
        if let Err(e) = self.sender.send(msg).await {
            eprintln!("❌ Ошибка отправки сообщения: {}", e);
        }
    }

    /// Получить следующее сообщение
    pub async fn receive(&self) -> Option<Message> {
        let mut rx = self.receiver.lock().await;
        rx.recv().await
    }
}

/// Пример поведения ноды при получении сообщения
pub async fn handle_message(node: Arc<Mutex<Node>>, msg: Message, network: Arc<NetworkBus>) {
    let mut n = node.lock().await;

    match msg.msg_type {
        MessageType::HelpRequest => {
            // Если у нас достаточно энергии — помогаем
            let current_energy = n.energy.lock().await.level;
            if current_energy > 30.0 && n.altruism > 0.5 {
                let response = Message::new(
                    &n.name,
                    msg.to.as_deref(),
                    MessageType::EnergyTransfer,
                    5.0,
                    Some("Помогаю соседу 🔋"),
                );
                network.send(response).await;
                println!("🤝 {} отправил энергию по запросу {}", n.name, msg.from);
            }
        }

        MessageType::EnergyTransfer => {
            let mut energy = n.energy.lock().await;
            energy.restore(msg.value);
            println!("🔋 {} получил {:.1} энергии от {}", n.name, msg.value, msg.from);
        }

        MessageType::BlockAnnouncement => {
            println!("🧱 {} получил уведомление о новом блоке от {}", n.name, msg.from);
        }

        MessageType::ValidateBlock => {
            println!("🧐 {} проверяет блок от {}", n.name, msg.from);
        }
    }
}
