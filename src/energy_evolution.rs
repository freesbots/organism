use crate::node::Node;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct EnergyEvolution;

impl EnergyEvolution {
    pub async fn evolve(nodes: &mut Vec<Arc<Mutex<Node>>>) {
        // 🔁 Энергетический цикл для каждой ноды
        for node in nodes.iter() {
            let mut node_guard = node.lock().await;

            // 🔋 Работаем с энергией отдельно
            {
                let mut energy = node_guard.energy.lock().await;

                // 🧮 Расход энергии
                let consumption = 5.0 * (1.0 - node_guard.efficiency).max(0.1);
                energy.consume(consumption);

                // 🔋 Восстановление
                energy.restore(0.5 * node_guard.resilience);

                // 🔄 Ограничиваем диапазон
                energy.level = energy.level.clamp(0.0, 100.0);

                // 🔁 Сохраняем значение для дальнейшей логики
                if energy.level > 50.0 {
                    // Освобождаем energy перед изменением node_guard
                    drop(energy);
                    node_guard.experience += 0.2;
                } else if energy.level < 10.0 {
                    drop(energy);
                    node_guard.efficiency = (node_guard.efficiency * 0.95).max(0.1);
                } else {
                    drop(energy);
                }
            }

            let energy_level = node_guard.energy.lock().await.level;

            println!(
                "⚡ {} → энергия: {:.2}, опыт: {:.2}",
                node_guard.name, energy_level, node_guard.experience
            );
        }
        // 💡 Эволюция: поиск лучшей ноды
        let mut best_index = 0;
        let mut best_score = f64::MIN;

        for (i, node) in nodes.iter().enumerate() {
            let node_guard = node.lock().await;
            let energy = node_guard.energy.lock().await;
            let score = energy.level * 0.3 + node_guard.experience * 0.7;

            if score > best_score {
                best_score = score;
                best_index = i;
            }
        }

        // 🌟 Эволюция — усиление параметров лучшей ноды
        if let Some(best_node) = nodes.get(best_index) {
            let mut n = best_node.lock().await;
            n.efficiency = (n.efficiency + 0.05).min(1.0);
            n.altruism = (n.altruism + 0.02).min(1.0);
            n.resilience = (n.resilience + 0.03).min(1.5);

            println!(
                "🌟 {} эволюционирует! (eff={:.2}, alt={:.2}, res={:.2})",
                n.name, n.efficiency, n.altruism, n.resilience
            );
        }
    }
}
