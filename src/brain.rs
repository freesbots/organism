use std::sync::Arc;
use tokio::sync::Mutex;
use rand::{Rng, rngs::StdRng, SeedableRng};

use crate::node::Node;
use crate::economy::NetworkFund;

/// 🧠 Модуль сознания — координация действий между нодами.
/// Ноды "осознают" состояние других и принимают коллективные решения.
pub struct Brain;

impl Brain {
    /// Основная функция сознания (анализ и перераспределение).
    pub async fn run(nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>, fund: Arc<Mutex<NetworkFund>>) {
        let nodes_guard = nodes.lock().await;
        let mut energy_list = Vec::new();

        // 🧠 Собираем уровни энергии асинхронно
        for node in nodes_guard.iter() {
            let n = node.lock().await;
            let e = n.energy.lock().await;
            energy_list.push((n.name.clone(), e.level));
        }

        // 📊 Сортируем по уровню энергии
        energy_list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        println!("🧠 Сознание сети — уровни энергии:");
        for (name, level) in &energy_list {
            println!("   • {} → {:.2}", name, level);
        }

        // 🤝 Пример перераспределения энергии
        if let (Some((weak_name, weak_level)), Some((strong_name, strong_level))) =
            (energy_list.first(), energy_list.last())
        {
            if strong_level - weak_level > 10.0 {
                println!(
                    "⚡ Сознание решило перераспределить энергию: {} → {} (разница {:.2})",
                    strong_name, weak_name, strong_level - weak_level
                );

                // 🔍 Асинхронный поиск нод
                let mut weak_node_opt = None;
                let mut strong_node_opt = None;

                for n in nodes_guard.iter() {
                    let node = n.lock().await;
                    if node.name == *weak_name {
                        weak_node_opt = Some(n.clone());
                    } else if node.name == *strong_name {
                        strong_node_opt = Some(n.clone());
                    }
                }

                if let (Some(weak_node), Some(strong_node)) = (weak_node_opt, strong_node_opt) {
                    let mut w_guard = weak_node.lock().await;
                    let mut s_guard = strong_node.lock().await;

                    let mut w_energy = w_guard.energy.lock().await;
                    let mut s_energy = s_guard.energy.lock().await;

                    let transfer = 5.0;
                    if s_energy.level > transfer {
                        s_energy.level -= transfer;
                        w_energy.level += transfer;

                        println!(
                            "🔋 {} получил +{transfer}, {} отдал -{transfer}",
                            weak_name, strong_name
                        );
                    }
                }
            }
        }


        drop(nodes_guard);
        drop(fund);
    }

    /// 🔄 Балансировка энергии (вариант, вызываемый отдельно)
    pub async fn balance_energy(nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>) {
        let nodes_guard = nodes.lock().await;
        let mut energy_data = Vec::new();

        // Собираем энергию
        for n in nodes_guard.iter() {
            let node_guard = n.lock().await;
            let energy = node_guard.energy.lock().await;
            energy_data.push((node_guard.name.clone(), energy.level));
        }

        // Сортируем
        energy_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let (Some((weak_name, weak_level)), Some((strong_name, strong_level))) =
            (energy_data.first(), energy_data.last())
        {
            if strong_level - weak_level > 10.0 {
                println!(
                    "🤝 Балансировка энергии: {} → {} (разница {:.2})",
                    strong_name, weak_name, strong_level - weak_level
                );
            }
        }
    }
}
