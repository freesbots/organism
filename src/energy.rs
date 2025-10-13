use crate::node::Node;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::energy_evolution::EnergyEvolution;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::join_all;

 

#[derive(Clone, Debug)]
pub struct Energy {
    pub level: f64,
    pub node_name: String,
}

impl Energy {
    pub fn new(name: &str) -> Self {
        Self {
            level: 100.0, // стартовая энергия
            node_name: name.to_string(),
        }
    }

    pub fn consume(&mut self, amount: f64) {
        self.level = (self.level - amount).max(0.0);
    }

    pub fn restore(&mut self, amount: f64) {
        self.level = (self.level + amount).min(100.0);
    }

    pub fn is_exhausted(&self) -> bool {
        self.level <= 0.0
    }

    pub fn status(&self) -> String {
        format!("⚡ {}: {:.2}", self.node_name, self.level)
    }
}

impl fmt::Display for Energy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Energy(level: {:.2}, node: {})", self.level, self.node_name)
    }
}
pub struct EnergySystem;

impl EnergySystem {
    pub async fn tick(nodes: &mut Vec<Node>) {
        // 1️⃣ Сначала делаем снимок уровней энергии (чтобы избежать пересечений borrow)
        let energy_snapshot: Vec<f64> = join_all(
            nodes.iter().map(|n| async {
                let e = n.energy.lock().await;
                e.level
            })
        ).await;

        // 2️⃣ Теперь спокойно проходим по всем нодам
        let total_nodes = nodes.len();
        for i in 0..total_nodes {
            // Естественное восстановление энергии
            {
                let mut e = nodes[i].energy.lock().await;
                e.restore(0.3);
                if e.level > 100.0 {
                    e.level = 100.0;
                }
            }

            // Выбираем ноды, у которых мало энергии (кроме текущей)
            let weak_nodes: Vec<usize> = (0..total_nodes)
                .filter(|&j| j != i && energy_snapshot[j] < 20.0)
                .collect();

            if weak_nodes.is_empty() {
                continue;
            }

            // Выбираем случайную слабую ноду
            let mut rng = thread_rng();
            if let Some(&target_idx) = weak_nodes.choose(&mut rng) {
                let giver_level = {
                    let e = nodes[i].energy.lock().await;
                    e.level
                };

                if giver_level > 30.0 {
                    let transfer = (giver_level * 0.1).min(10.0);

                    {
                        let mut giver_e = nodes[i].energy.lock().await;
                        giver_e.consume(transfer);
                    }

                    {
                        let mut receiver_e = nodes[target_idx].energy.lock().await;
                        receiver_e.restore(transfer);
                    }

                    let giver_now = nodes[i].energy.lock().await.level;
                    let receiver_now = nodes[target_idx].energy.lock().await.level;

                    println!(
                        "🔋 {} → {} передача {:.2} энергии (теперь {:.2}/{:.2})",
                        nodes[i].name, nodes[target_idx].name, transfer, giver_now, receiver_now
                    );
                }
            }
        }
    }
}