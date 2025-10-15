use std::sync::Arc;
use serde::{Serialize};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use rand::{Rng, rngs::StdRng, SeedableRng};
use crate::memory::{Memory, BrainEvent};
use chrono::Utc;

use crate::node::Node;
use crate::economy::NetworkFund;
use tokio::sync::RwLock; 

/// 🧠 Модуль сознания — координация действий между нодами. 

#[derive(Serialize, Clone)]
pub struct NodeEnergyInfo {
    pub name: String,
    pub energy: f64,
    pub experience: f64,
}

#[derive(Serialize, Clone)]
pub struct BrainState {
    pub nodes: Vec<NodeEnergyInfo>,
    pub summary_avg_energy: f64,
}
pub struct Brain {
    pub memory: Memory,
    pub aggressiveness: f64,
}
 

#[derive(Clone, Debug, serde::Serialize)]
pub struct BrainSnapshot {
    pub aggressiveness: f64,
    pub avg_recent_result: f64,
    pub recent_memory: Vec<BrainEvent>, 
    pub last_update: i64,
}

impl BrainSnapshot {
    pub fn from_brain(brain: &Brain, avg: f64, recent_memory: Vec<BrainEvent>) -> Self {
        let recent_memory = futures::executor::block_on(brain.memory.get_recent(10));
        Self {
            aggressiveness: brain.aggressiveness,
            avg_recent_result: avg,
            recent_memory,
            last_update: Utc::now().timestamp(),
        }
    }
}
 



impl Brain {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            aggressiveness: 1.0,
        }
    }
    /// Основной неблокирующий цикл сознания.
    /// Запускается как отдельная таска: Brain::run(...).await
    pub async fn run(&mut self, nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>, fund: Arc<Mutex<NetworkFund>>) {

        let mut ticker = interval(Duration::from_secs(5));
        let mut rng = StdRng::from_entropy();
        /* let cycle_counter = 0; */

        loop {
            ticker.tick().await;
            // Snapshot nodes list quickly without holding long locks
            let snapshot_nodes = {
                let guard = nodes.lock().await;
                guard.clone()
            };

            // Collect energy data concurrently
            let mut energy_data = Vec::with_capacity(snapshot_nodes.len());
            for n in snapshot_nodes.iter() {
                // each node is Arc<Mutex<Node>>
                if let Ok(node) = n.try_lock() {
                    // fast path: if not contested, read quickly
                    let energy_guard = node.energy.lock().await;
                    let energy = energy_guard.level;
                    let exp = node.experience;
                    energy_data.push((node.name.clone(), energy, exp));
                    // drop lock quickly 
                } else {
                    // fallback: await lock to read stable data
                    let node = n.lock().await;
                    let energy_guard = node.energy.lock().await;
                    let energy = energy_guard.level;
                    let exp = node.experience;
                    energy_data.push((node.name.clone(), energy, exp)); 
                }
            }

            if energy_data.is_empty() {
                continue;
            }

            // compute average energy
            let avg = energy_data.iter().map(|(_,e,_)| *e).sum::<f64>() / (energy_data.len() as f64);

            // decide redistribution if imbalance detected
            let mut energy_list: Vec<(String, f64)> = energy_data.iter().map(|(n,e,_)| (n.clone(), *e)).collect();
            energy_list.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            let lowest = energy_list.first().unwrap();
            let highest = energy_list.last().unwrap();

            if highest.1 - lowest.1 > 10.0 {
                // transfer a fraction from high to low
                let delta = (highest.1 - lowest.1) * 0.25;

                // найти ссылки на ноды по именам
                let from_opt = snapshot_nodes.iter().find(|n| {
                    if let Ok(node) = n.try_lock() {
                        let nm = node.name.clone();
                        drop(node);
                        nm == highest.0
                    } else {
                        false
                    }
                }).cloned();

                let to_opt = snapshot_nodes.iter().find(|n| {
                    if let Ok(node) = n.try_lock() {
                        let nm = node.name.clone();
                        drop(node);
                        nm == lowest.0
                    } else {
                        false
                    }
                }).cloned();

                // если обе ноды найдены
                if let (Some(from_arc), Some(to_arc)) = (from_opt, to_opt) {
                    // берём блокировки
                    let from_node = from_arc.lock().await;
                    let to_node = to_arc.lock().await;

                    // берём энергию обеих нод
                    let mut from_energy = from_node.energy.lock().await;
                    let mut to_energy = to_node.energy.lock().await;

                    // перераспределяем энергию
                    if from_energy.level >= delta {
                        from_energy.level -= delta;
                        to_energy.level += delta;
                        println!(
                            "🤝 Brain: перераспределил {:.2} энергии: {} -> {}",
                            delta, from_node.name, to_node.name
                        );
                        self.memory.record(BrainEvent::new(
                            "redistribution",
                            &format!("{} -> {} Δ={:.2}", from_node.name, to_node.name, delta),
                            delta,
                        )).await;
                    }

                    // явно отпускаем блокировки (не обязательно, но аккуратно)
                    drop(to_energy);
                    drop(from_energy);
                    drop(to_node);
                    drop(from_node);
                }

            } else {
                // occasional random small adjustments to simulate decisions
                let idx = rng.gen_range(0..energy_data.len());
                let (name, level, _exp) = &energy_data[idx];
                if *level < avg {
                    // find node and boost slightly from fund if available
                    // use fund in a non-blocking way
                    let fund_guard = fund.lock().await;
                    if fund_guard.get_balance().await > 0.1 {
                        let give = 0.5;

                        // Проверим, достаточно ли средств, и уменьшим фонд вручную
                        let mut total = fund_guard.total.lock().await;
                        if *total >= give {
                            *total -= give;
                            drop(total); // освободили мьютекс фонда перед обновлением ноды

                            // найти нужную ноду и добавить ей энергию
                            if let Some(narc) = snapshot_nodes.iter().find(|n| {
                                if let Ok(node) = n.try_lock() {
                                    let nm = node.name.clone();
                                    drop(node);
                                    nm == *name
                                } else { false }
                            }) {
                                // создаём отдельную область, чтобы гарантировать порядок drop
                                // захватываем сам узел
                                let node = narc.lock().await;

                                {
                                    let mut energy_guard = node.energy.lock().await;
                                    energy_guard.level += give;
                                }

                                // используем node.name, теперь он доступен
                                self.memory
                                    .record(BrainEvent::new(
                                        "fund_support",
                                        &format!("Fund gave {:.2} energy to {}", give, node.name),
                                        give,
                                    ))
                                    .await;
                            }
                        } else {
                            println!("⚠️ Фонд не имеет достаточно средств для выделения {:.2}", give);
                        }
                    }


                }
            }

            let nodes_guard = nodes.lock().await;
            let mut total_energy = 0.0;
            let mut count = 0;

            for node_arc in nodes_guard.iter() {
                let node = node_arc.lock().await;
                let energy_guard = node.energy.lock().await;
                total_energy += energy_guard.level;
                count += 1;
            }

            let avg_energy = if count > 0 {
                total_energy / count as f64
            } else {
                0.0
            };
            println!("🧠 Сознание — средняя энергия: {:.2}", avg_energy);

            self.memory.record(
                BrainEvent::new(
                    "analyze",
                    &format!("Средняя энергия сети: {:.2}", avg),
                    avg,
                )
            ).await;

            let recent_avg = self.memory.average_result(5).await;

            if recent_avg < 50.0 {
                self.aggressiveness *= 1.1;
                self.memory.record(
                    BrainEvent::new("adjust", "Увеличение реактивности", self.aggressiveness)
                ).await;
                println!("🧬 Brain усиливает реактивность (aggr={:.2})", self.aggressiveness);
            } else if recent_avg > 90.0 {
                self.aggressiveness *= 0.9;
                self.memory.record(
                    BrainEvent::new("adjust", "Снижение реактивности", self.aggressiveness)
                ).await;
                println!("🌿 Brain снижает реактивность (aggr={:.2})", self.aggressiveness);
            }

            
        }
    }

    pub async fn run_step(&mut self, nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>, fund: Arc<Mutex<NetworkFund>>) {
        // одна итерация логики мозга
        self.memory.record(
            BrainEvent {
                timestamp: chrono::Utc::now().timestamp() as u64,
                action: "thinking".to_string(),
                context: "analyzing".to_string(),
                result: 1.0,
            }
        ).await;
        // какая-то логика, например вычисление средней энергии
    }
    /// Возвращает структуру состояния мозга для API (snapshot).
    pub async fn snapshot(nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>) -> BrainState {
        let snapshot_nodes = {
            let guard = nodes.lock().await;
            guard.clone()
        };

        let mut infos = Vec::with_capacity(snapshot_nodes.len());
        let mut sum = 0.0;
        for n in snapshot_nodes.iter() {
            let node = n.lock().await;
            let energy_guard = node.energy.lock().await;
            let energy = energy_guard.level;
            let exp = node.experience;
            infos.push(NodeEnergyInfo {
                name: node.name.clone(),
                energy,
                experience: exp,
            });
            sum += energy; 
        }
        let avg = if infos.is_empty() { 0.0 } else { sum / (infos.len() as f64) };
        BrainState { nodes: infos, summary_avg_energy: avg }
    }
}
