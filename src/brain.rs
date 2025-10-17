use std::sync::Arc;
use serde::{Serialize}; 
use tokio::time::{timeout, interval, Duration}; 
use crate::memory::{Memory, BrainEvent}; 
use chrono::Utc; 
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::interaction::NetworkBus;
use crate::economy::NetworkFund; 
use tokio::sync::Mutex; 

use crate::node::Node; 
use tokio::sync::RwLock;  
use tokio::sync::{Semaphore};
use futures::future::join_all;
use rand::thread_rng;

 

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
    pub memory: Arc<Mutex<Memory>>,
    pub aggressiveness: f64,
    pub reward_history: Vec<f64>,
    pub tick_counter: u64, 
} 
 

#[derive(Clone, Debug, serde::Serialize)]
pub struct BrainSnapshot {
    pub aggressiveness: f64,
    pub avg_recent_result: f64,
    pub recent_memory: Vec<BrainEvent>, 
    pub last_update: i64,
}

impl BrainSnapshot { 
    pub async fn from_brain_lock(brain: &Arc<RwLock<Brain>>) -> Self {
        let brain_guard = brain.read().await;
        let memory_guard = brain_guard.memory.lock().await;
        let recent_memory = memory_guard.get_recent(10).await;
        let avg_result = memory_guard.average_result(10).await;

        println!(
            "📊 [DEBUG] Snapshot: recent_memory.len = {}, avg_result = {:.2}",
            recent_memory.len(),
            avg_result
        );

        Self {
            aggressiveness: brain_guard.aggressiveness,
            avg_recent_result: avg_result,
            recent_memory,
            last_update: chrono::Utc::now().timestamp(),
        }
    }
}



impl Brain {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(Memory::new(100, 10000, 604800))), // short=100, long=10000
            aggressiveness: 1.0,
            reward_history: Vec::new(),
            tick_counter: 0, 
        }
    }
    /// Основной неблокирующий цикл сознания.
    /// Запускается как отдельная таска: Brain::run(...).await
    pub async fn run(
        &mut self,
        nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
        fund: Arc<Mutex<NetworkFund>>,
        net: Arc<NetworkBus>,
    ) {
        

        println!("🧠 [Brain::run] Цикл мозга запущен!");

        let mut ticker = interval(Duration::from_secs(5)); 
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
            self.memory.lock().await.add_event(
                BrainEvent::new("analyze", "Средняя энергия сети", avg_energy)
            ).await;
            println!("✅ [DEBUG] Событие отправлено в память!");

             
            // === 3️⃣ Принятие решения ===
            let evolve_chance = (avg_energy / 100.0).clamp(0.1, 0.9);
            let action = if rand::random::<f64>() < evolve_chance {
                "evolve"
            } else if avg_energy > 40.0 {
                "help"
            } else {
                "rest"
            };
            println!("🧩 Решение: {}", action);

            let mut rng = StdRng::from_entropy();
            // === 4️⃣ Исполнение действия ===
            let result_metric = match action {
                "help" => {
                    self.redistribute_energy(&snapshot_nodes, &fund, avg_energy).await;
                    rng.gen_range(0.7..1.0)
                }
                "evolve" => { 
                    println!("🧩🧠 [Brain::run::spawn] evolve start");
                    let nodes_clone = nodes.clone();
                    let fund_clone = fund.clone();
                    let net_clone = net.clone();
                    let mut brain_clone = self.clone();

                    tokio::spawn(async move {
                        brain_clone.evolve_network(nodes_clone, fund_clone, net_clone).await;
                        println!("🧠 [Brain::run::spawn] evolve_network завершена");
                    });

                    for n in snapshot_nodes.iter() {
                        if let Ok(node) = n.try_lock() {
                            let mut e = node.energy.lock().await;
                            e.level += rng.gen_range(0.5..2.0);
                        }
                    }

                    rng.gen_range(0.7..1.0)
                }
                "rest" => { 
                    println!("😴 Brain: сеть отдыхает...");
                    for n in snapshot_nodes.iter() {
                        if let Ok(node) = n.try_lock() {
                            let mut e = node.energy.lock().await;
                            e.level += rng.gen_range(0.5..2.0);
                        }
                    }
                    rng.gen_range(0.5..0.8)
                }
                _ => 0.5,
            };
 
             
            self.memory.lock().await.add_event(BrainEvent::new(
                "feedback",
                "Результат действия",
                result_metric,
            )).await;
            println!("✅ [DEBUG] Событие отправлено в память!");

            // === 6️⃣ Адаптация (обучение) ===
            self.learn_from_feedback(result_metric).await;

            // === 7️⃣ Мониторинг ===
            let recent_avg = self.memory.lock().await.average_result(10).await;
            println!(
                "🧠 Brain: avg_energy = {:.2}, result = {:.2}, aggr = {:.2}, recent_avg = {:.2}",
                avg_energy, result_metric, self.aggressiveness, recent_avg
            );

            // === 8️⃣ Саморегуляция ===
            if recent_avg < 0.4 {
                self.aggressiveness *= 1.15;
                self.memory.lock().await.add_event(
                    BrainEvent::new("feedback", "Рост реактивности", self.aggressiveness)
                ).await; 
                println!("⚡ Увеличение агрессивности → {:.2}", self.aggressiveness);
            } else if recent_avg > 0.8 {
                self.aggressiveness *= 0.9;
                self.memory.lock().await.add_event(
                    BrainEvent::new("feedback", "Снижение реактивности", self.aggressiveness)
                ).await;  
                println!("🌿 Снижение агрессивности → {:.2}", self.aggressiveness);
            }else { 
                self.aggressiveness *= 1.02;
                self.memory.lock().await.add_event(
                    BrainEvent::new("feedback", "поддерживаем динамику", self.aggressiveness)
                ).await; 
            }
            
            if rand::random::<f64>() < 0.2 {
                let mut aggr = self.aggressiveness;
                aggr += (rand::random::<f64>() - 0.5) * 0.1;
                self.aggressiveness = aggr.clamp(0.1, 2.0);
                println!("🔥 [Mutation] агрессивность случайно изменилась → {:.2}", self.aggressiveness);
            }

            // 🧩 Каждые 10 тиков — самоанализ мозга
            if self.tick_counter % 10 == 0 {
                let event = BrainEvent::new("reflect", "Самоанализ цикла", self.aggressiveness);
                self.memory.lock().await.add_event(event).await;
                println!("💭 [Brain::reflect] Самоанализ выполнен (агрессивность {:.2})", self.aggressiveness);
            }

            // 🌀 Самовосстановление импульса 
            if self.tick_counter % 5 == 0 {
                // каждые 5 циклов слегка поднимаем агрессивность
                self.aggressiveness += 0.1 * (1.0 - self.aggressiveness);
                self.aggressiveness = self.aggressiveness.clamp(0.2, 2.0);
                println!("💥 [Impulse] восстановление импульса: агрессивность {:.2}", self.aggressiveness);
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

                let mut delta = (from_energy.level - to_energy.level) * 0.2;

                // 💖 если цель — потомок, усиливаем помощь
                if to_node.name.contains("_child_") {
                    delta *= 1.5; // помогать потомкам чуть больше
                }

                if delta > 1.0 {
                    from_energy.consume(delta);
                    to_energy.restore(delta); 
                }
                println!(
                    "🤝 Brain: перераспределил {:.2} энергии {} → {}",
                    delta, from_node.name, to_node.name
                );

                self.memory.lock().await.add_event(
                    BrainEvent::new(
                        "redistribution",
                        &format!("{} → {} (Δ={:.2})", from_node.name, to_node.name, delta),
                        delta,
                    )
                ).await;
 
            }
        }
    }

    /// 🧬 Эволюционное обновление сети (evolve mode)
    pub async fn evolve_network(
        &mut self,
        nodes_ref: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
        _fund: Arc<Mutex<NetworkFund>>,
        net: Arc<NetworkBus>,
    ) {
        println!("🧠 [DEBUG] evolve_network START tick={}", self.tick_counter);

        // --- 1️⃣ Снимок текущих нод ---
        let snapshot_nodes = {
            let nodes = nodes_ref.lock().await;
            nodes.clone()
        };
        let total_before = snapshot_nodes.len();
        if total_before == 0 {
            println!("⚠️ Нет активных нод для эволюции");
            return;
        }

        // --- 2️⃣ Контроль перенаселения ---
        {
            let total_now = nodes_ref.lock().await.len();
            if total_now > 400 {
                println!("⚠️ [EVOLUTION] Перенаселение: {} нод — эволюция пропущена", total_now);
                return;
            }
        }

        println!("🧠 Эволюция узлов ({} нод)", total_before);

        // --- 3️⃣ Ограничиваем параллельные tick-и ---
        use tokio::sync::Semaphore;
        let semaphore = Arc::new(Semaphore::new(12)); // максимум 12 одновременно
        let mut handles = Vec::new();

        // --- 4️⃣ Запуск tick для каждой ноды ---
        for n_arc in snapshot_nodes.into_iter() {
            let sem = semaphore.clone();
            let net = net.clone();
            let nodes_ref_clone = nodes_ref.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                // таймаут — если тик зависнет, пропускаем
                match tokio::time::timeout(Duration::from_secs(6),
                    Node::tick(n_arc.clone(), net.clone(), nodes_ref_clone.clone(), 0)
                ).await {
                    Ok(res) => res,       // Option<Arc<Mutex<Node>>>
                    Err(_) => {
                        println!("⚠️ [Tick Timeout] Node tick took too long, skipped");
                        None
                    }
                }
            });
            handles.push(handle);
        }

        // --- 5️⃣ Собираем новых детей ---
        let mut new_children: Vec<Arc<Mutex<Node>>> = Vec::new();
        for h in handles {
            if let Ok(Some(child)) = h.await {
                new_children.push(child);
            }
        }

        // --- 6️⃣ Удаляем "мёртвые" ноды ---
        {
            let mut nodes_locked = nodes_ref.lock().await;
            let mut survivors: Vec<Arc<Mutex<Node>>> = Vec::new();

            for n in nodes_locked.iter() {
                let node = n.lock().await;
                let e = node.energy.lock().await;
                if e.level > 5.0 {
                    survivors.push(n.clone());
                }
            }

            let removed = nodes_locked.len().saturating_sub(survivors.len());
            *nodes_locked = survivors;

            if removed > 0 {
                println!("🧹 Удалено {} мёртвых нод", removed);
            }
        }

        // --- 7️⃣ Добавляем новых потомков ---
        if !new_children.is_empty() {
            let added = new_children.len();
            let mut nodes_locked = nodes_ref.lock().await;
            nodes_locked.extend(new_children);
            println!("🧬 Добавлено потомков: {}, теперь всего {}", added, nodes_locked.len());
        } else {
            println!("🧬 Эволюция прошла без новых нод");
        }

        // --- 8️⃣ Контроль перенаселения + удаление слабых ---
        {
            let mut nodes = nodes_ref.lock().await;
            if nodes.len() > 120 {
                println!("⚠️ Перенаселение ({} нод): удаляем слабейших...", nodes.len());

                // безопасный снимок энергий
                let mut energy_snapshot = vec![];
                for n in nodes.iter() {
                    let node = n.lock().await;
                    let e = node.energy.lock().await;
                    energy_snapshot.push((n.clone(), e.level));
                }

                // сортировка по энергии
                energy_snapshot.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                // оставляем 80 самых сильных
                let survivors: Vec<_> = energy_snapshot.into_iter().rev().take(80).map(|(n, _)| n).collect();
                let removed = nodes.len().saturating_sub(survivors.len());
                *nodes = survivors;

                println!("🧹 Удалено {} слабых нод (truncate до 80)", removed);
            }
        }

        // --- 9️⃣ Восстанавливаем энергию выживших ---
        {
            let mut rng = StdRng::from_entropy();
            let mut nodes_locked = nodes_ref.lock().await;

            for n in nodes_locked.iter() {
                let mut node = n.lock().await;
                let mut e = node.energy.lock().await;
                e.level += 5.0 + rng.gen_range(0.0..10.0);
                if e.level > 120.0 {
                    e.level = 120.0;
                }
            }
        }

        println!("✅ [DEBUG] evolve_network DONE");
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
