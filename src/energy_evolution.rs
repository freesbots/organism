use crate::node::Node;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::{SeedableRng, rngs::StdRng};


pub struct EnergyEvolution;

impl EnergyEvolution {
    pub async fn evolve(nodes: &mut Vec<Arc<Mutex<Node>>>) {
        /* let mut rng = rand::thread_rng(); */
        let mut rng = StdRng::from_entropy();

        for node in nodes.iter() {
            let mut n = node.lock().await;

            // --- Работаем с энергией ---
            let (energy_level, efficiency, resilience) = {
                let mut energy = n.energy.lock().await;

                // 🧮 Энергозатраты
                let consumption = 5.0 * (1.0 - n.efficiency).max(0.1);
                energy.consume(consumption);

                // 🔋 Восстановление
                energy.restore(0.5 * n.resilience);

                // Ограничиваем энергию
                energy.level = energy.level.clamp(0.0, 100.0);

                (energy.level, n.efficiency, n.resilience)
            }; // <-- Здесь блокировка энергии завершается!

            // --- Работаем с остальными параметрами ---
            if energy_level > 50.0 {
                n.experience += 0.2;
            }

              // ⚙️ Корректировка при низкой энергии
            if energy_level < 10.0 {
                n.efficiency = (n.efficiency * 0.95).max(0.1);
            }

            println!(
                "⚡ {} → энергия: {:.2}, опыт: {:.2}",
                n.name, energy_level, n.experience
            );
        }

        // 💡 Поиск лучшей ноды для эволюции
        let mut best_index = 0;
        let mut best_score = f64::MIN;

        for (i, node) in nodes.iter().enumerate() {
            let n = node.lock().await;
            let e = n.energy.lock().await;

            let score = e.level * 0.3
                + n.experience * 0.6
                + n.altruism * 20.0
                + rng.gen_range(-5.0..5.0);

            if score > best_score {
                best_score = score;
                best_index = i;
            }
        }

        // 🌟 Эволюционирует лучшая нода
        if let Some(best_node) = nodes.get(best_index) {
            let mut n = best_node.lock().await;

            // ✳️ Эффект усталости лидера
            if n.efficiency > 0.9 {
                n.efficiency *= 0.98;
                println!( "✳️ → Эффект усталости лидера");
            }
            
            n.efficiency = (n.efficiency + rng.gen_range(0.02..0.07)).min(1.0);
            n.altruism = (n.altruism + rng.gen_range(0.01..0.04)).min(1.0);
            n.resilience = (n.resilience + rng.gen_range(0.02..0.06)).min(1.5);

            // 💰 Награда за эволюцию
            n.wallet.reward(10.0).await;

            println!(
                "🌟 {} эволюционирует! (eff={:.2}, alt={:.2}, res={:.2})",
                n.name, n.efficiency, n.altruism, n.resilience
            );
        }
    }
}
