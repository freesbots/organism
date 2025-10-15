use std::sync::Arc;
use serde::{Serialize};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration}; 
use crate::memory::{Memory, BrainEvent};
use tokio::sync::Mutex as TMutex;
use chrono::Utc;
use rand::{SeedableRng, rngs::StdRng};
use rand::Rng; 

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
#[derive(Clone)]
pub struct Brain {
    pub memory: Memory,
    pub aggressiveness: f64,
    pub reward_history: Vec<f64>, 
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
        Self {
            aggressiveness: brain.aggressiveness,
            avg_recent_result: avg,
            recent_memory,
            last_update: Utc::now().timestamp(),
        }
    }

    pub async fn from_brain_lock(brain_lock: &Arc<RwLock<Brain>>) -> Self {
        let brain = brain_lock.read().await;
        let avg = brain.memory.average_result(20).await;
        let mem = brain.memory.get_recent(10).await;
        Self::from_brain(&brain, avg, mem)
    }
}



impl Brain {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(100, 10000, 604800), // short=100, long=10000
            aggressiveness: 1.0,
            reward_history: Vec::new(),
        }
    }
    /// Основной неблокирующий цикл сознания.
    /// Запускается как отдельная таска: Brain::run(...).await
    pub async fn run(
        &mut self,
        nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
        fund: Arc<Mutex<NetworkFund>>,
    ) {
        

        println!("🧠 [Brain::run] Цикл мозга запущен!");

        let mut ticker = interval(Duration::from_secs(5));
        let mut rng = StdRng::from_entropy();

        loop {
            ticker.tick().await;

            // === 1️⃣ Сканирование узлов ===
            let snapshot_nodes = {
                let guard = nodes.lock().await;
                guard.clone()
            };

            let mut energy_data = Vec::new();
            for n in snapshot_nodes.iter() {
                if let Ok(node) = n.try_lock() {
                    let e = node.energy.lock().await;
                    energy_data.push((node.name.clone(), e.level, node.experience));
                }
            }

            if energy_data.is_empty() {
                println!("⚠️ Нет активных нод для анализа");
                continue;
            }

            // === 2️⃣ Анализ состояния сети ===
            let avg_energy = energy_data.iter().map(|(_, e, _)| *e).sum::<f64>() / energy_data.len() as f64;
            self.memory.add_event(BrainEvent::new("analyze", "Средняя энергия сети", avg_energy)).await;

            // === 3️⃣ Принятие решения ===
            let action = if self.aggressiveness > 1.0 { "evolve" } else { "help" };
            println!("🧩 Решение: {}", action);

            // === 4️⃣ Исполнение действия ===
            let result_metric = match action {
                "help" => {
                    self.redistribute_energy(&snapshot_nodes, &fund, avg_energy).await;
                    rng.gen_range(0.7..1.0) // успешная помощь
                }
                "evolve" => {
                    self.evolve_network(&snapshot_nodes).await;
                    rng.gen_range(0.0..1.0) // эволюция может быть рискованной
                }
                _ => 0.5,
            };

            // === 5️⃣ Оценка результата ===
            self.memory
                .add_event(BrainEvent::new("feedback", "Результат действия", result_metric))
                .await;

            // === 6️⃣ Адаптация (обучение) ===
            self.learn_from_feedback(result_metric).await;

            // === 7️⃣ Мониторинг ===
            let recent_avg = self.memory.average_result(10).await;
            println!(
                "🧠 Brain: avg_energy = {:.2}, result = {:.2}, aggr = {:.2}, recent_avg = {:.2}",
                avg_energy, result_metric, self.aggressiveness, recent_avg
            );

            // === 8️⃣ Саморегуляция ===
            if recent_avg < 0.4 {
                self.aggressiveness *= 1.15;
                self.memory
                    .add_event(BrainEvent::new("adjust", "Рост реактивности", self.aggressiveness))
                    .await;
                println!("⚡ Увеличение агрессивности → {:.2}", self.aggressiveness);
            } else if recent_avg > 0.8 {
                self.aggressiveness *= 0.9;
                self.memory
                    .add_event(BrainEvent::new("adjust", "Снижение реактивности", self.aggressiveness))
                    .await;
                println!("🌿 Снижение агрессивности → {:.2}", self.aggressiveness);
            }

            // можно вставить небольшую паузу, чтобы не перегружать цикл
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// 🔄 Перераспределение энергии между узлами (help mode)
    pub async fn redistribute_energy(
        &mut self,
        snapshot_nodes: &Vec<Arc<Mutex<Node>>>,
        fund: &Arc<Mutex<NetworkFund>>,
        avg_energy: f64,
    ) {
        if snapshot_nodes.is_empty() {
            return;
        }

        // Сортируем по уровню энергии
        let mut energy_list: Vec<(String, f64)> = Vec::new();
        for n in snapshot_nodes.iter() {
            if let Ok(node) = n.try_lock() {
                let e = node.energy.lock().await;
                energy_list.push((node.name.clone(), e.level));
            }
        }

        if energy_list.len() < 2 {
            return;
        }

        energy_list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let (lowest_name, lowest_energy) = energy_list.first().unwrap();
        let (highest_name, highest_energy) = energy_list.last().unwrap();

        if highest_energy - lowest_energy < 5.0 {
            // Разброс мал — ничего не делаем
            return;
        }

        let delta = (highest_energy - lowest_energy) * 0.25;

        let from_opt = snapshot_nodes.iter().find(|n| {
            if let Ok(node) = n.try_lock() {
                node.name == *highest_name
            } else {
                false
            }
        });

        let to_opt = snapshot_nodes.iter().find(|n| {
            if let Ok(node) = n.try_lock() {
                node.name == *lowest_name
            } else {
                false
            }
        });

        if let (Some(from), Some(to)) = (from_opt, to_opt) {
            let mut from_node = from.lock().await;
            let mut to_node = to.lock().await;

            let mut from_energy = from_node.energy.lock().await;
            let mut to_energy = to_node.energy.lock().await;

            if from_energy.level >= delta {
                from_energy.level -= delta;
                to_energy.level += delta;

                println!(
                    "🤝 Brain: перераспределил {:.2} энергии {} → {}",
                    delta, from_node.name, to_node.name
                );

                self.memory
                    .add_event(BrainEvent::new(
                        "redistribution",
                        &format!("{} → {} (Δ={:.2})", from_node.name, to_node.name, delta),
                        delta,
                    ))
                    .await;
            }
        }
    }

    /// 🧬 Эволюционное обновление сети (evolve mode)
    pub async fn evolve_network(&mut self, snapshot_nodes: &Vec<Arc<Mutex<Node>>>) {
        
        let mut rng = StdRng::from_entropy();

        for n in snapshot_nodes.iter() {
            if let Ok(node) = n.try_lock() {
                let mut energy_guard = node.energy.lock().await;

                // Случайная "мутация" — немного увеличиваем или уменьшаем энергию
                let delta: f64 = rng.gen_range(-1.0..1.0);
                energy_guard.level = (energy_guard.level + delta).max(0.0);

                // Увеличиваем опыт узла
                drop(energy_guard);
                let mut nd = node.clone();
                nd.experience += rng.gen_range(0.0..0.2);

                self.memory
                    .add_event(BrainEvent::new(
                        "evolve_step",
                        &format!("{} изменён на {:.2}", node.name, delta),
                        delta,
                    ))
                    .await;
            }
        }

        println!("🧬 Brain провёл эволюцию узлов ({} нод)", snapshot_nodes.len());
    }


    /// Простая адаптация: скользящая корректировка aggressiveness на основе reward
    pub async fn learn_from_feedback(&mut self, reward: f64) {
        // Запомним
        self.reward_history.push(reward);
        if self.reward_history.len() > 200 {
            self.reward_history.remove(0);
        }
        // Простейшая логика: средний reward -> корректирует aggressiveness
        let avg: f64 = if !self.reward_history.is_empty() {
            let sum: f64 = self.reward_history.iter().sum();
            sum / (self.reward_history.len() as f64)
        } else { reward };

        // пример: если средний reward > 0.7 — немного повышаем агрессивность, иначе снижаем
        if avg > 0.7 {
            self.aggressiveness *= 1.02;
        } else {
            self.aggressiveness *= 0.98;
        }
        // clamp
        if self.aggressiveness < 0.2 { self.aggressiveness = 0.2; }
        if self.aggressiveness > 3.0 { self.aggressiveness = 3.0; }
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
