use crate::node::Node;
use std::sync::{Arc, Mutex};
use rand::Rng;

pub struct EnergyEvolution;

impl EnergyEvolution {
    pub fn evolve(nodes: &mut Vec<Node>) {
        for node in nodes.iter_mut() {
            let mut energy = node.energy.lock().unwrap();

            // 🧮 Расход энергии на выживание
            let consumption = 5.0 * (1.0 - node.efficiency).max(0.1);
            energy.consume(consumption);

            // 🔋 Восстановление энергии (устойчивость)
            energy.restore(0.5 * node.resilience);

            // 🌱 Рост опыта, если достаточно энергии
            if energy.level > 50.0 {
                node.experience += 0.2;
            }

            // ⚙️ Корректировка параметров
            if energy.level < 10.0 {
                node.efficiency = (node.efficiency * 0.95).max(0.1);
            }

            // 🔄 Ограничиваем энергию диапазоном
            if energy.level > 100.0 {
                energy.level = 100.0;
            } else if energy.level < 0.0 {
                energy.level = 0.0;
            }

            println!(
                "⚡ {} → уровень энергии: {:.2}, опыт: {:.2}",
                node.name, energy.level, node.experience
            );
        }

        // 💡 Эволюция: нода с лучшей энергией повышает параметры
        if let Some((_, best)) = nodes.iter_mut()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let ea = a.energy.lock().unwrap().level * 0.3 + a.experience * 0.7;
                let eb = b.energy.lock().unwrap().level * 0.3 + b.experience * 0.7;
                ea.partial_cmp(&eb).unwrap()
            })
        {
            best.efficiency = (best.efficiency + 0.05).min(1.0);
            best.altruism = (best.altruism + 0.02).min(1.0);
            best.resilience = (best.resilience + 0.03).min(1.5);
            println!("🌟 {} эволюционирует! (eff={:.2}, alt={:.2}, res={:.2})",
                best.name, best.efficiency, best.altruism, best.resilience
            );
        }
    }
}