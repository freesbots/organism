use std::sync::Arc;
use tokio::sync::Mutex;
use rand::{Rng, rngs::StdRng, SeedableRng};
use crate::{node::Node, economy::NetworkFund};

pub struct EconomyCycle;

impl EconomyCycle {
    /// Главный цикл перераспределения энергии и ресурсов
    pub async fn run(nodes: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>, fund: Arc<Mutex<NetworkFund>>) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;

            let mut total_energy = 0.0;
            let mut active_nodes = 0;

            {
                println!("💫 [DEBUG] Цикл экономики активен...");
                let nodes_guard = nodes.lock().await;
                for node in nodes_guard.iter() {
                    let n = node.lock().await;
                    let mut energy = n.energy.lock().await;
                    let balance = *n.wallet.balance.lock().await;

                    // 🔋 Естественные потери энергии
                    energy.level -= 2.0;

                    // ⚙️ Эффективность влияет на потери
                    energy.level *= n.efficiency.max(0.1);

                    // 💚 Минимальный порог — не позволяем умереть
                    if energy.level < 5.0 {
                        energy.level = 5.0;
                    }

                    total_energy += energy.level;
                    active_nodes += 1;

                    // 💸 Энергия влияет на токен: немного расходов
                    if balance > 0.5 {
                        n.wallet.spend(0.5).await;
                    }

                    // 🤝 Попробуем помочь слабому
                    if energy.level < 20.0 {
                        Self::help_weak_node(&nodes_guard, n.name.clone()).await;
                    }
                }
            }

            let avg_energy = total_energy / active_nodes.max(1) as f64;

            // ⚡ Если вся сеть устала — подпитываем из фонда
            if avg_energy < 25.0 {
                let fund_guard = fund.lock().await;
                let mut total = fund_guard.total.lock().await; // получаем доступ к значению f64

                if *total > 5.0 {
                    println!("⚡ Сеть получает подпитку от NetworkFund!");
                    for node in nodes.lock().await.iter() {
                        let n = node.lock().await;
                        let mut e = n.energy.lock().await;
                        e.level += 10.0;
                    }
                    *total -= 5.0; // уменьшаем значение фонда
                } else {
                    println!("⚠️ Фонд пуст — сеть слабеет...");
                }
            }



            println!("🌍 Средняя энергия сети: {:.2}", avg_energy);
        }
    }

    /// Функция помощи слабому узлу
    async fn help_weak_node(nodes: &Vec<Arc<Mutex<Node>>>, weak_name: String) {
        let mut rng = StdRng::from_entropy(); 

        if let Some(helper) = nodes.get(rng.gen_range(0..nodes.len())) {
            let h = helper.lock().await;
            if h.name == weak_name {
                return;
            }

            let helper_balance = *h.wallet.balance.lock().await;
            let mut helper_energy = h.energy.lock().await;

            // 💡 Помощь возможна, только если у хелпера достаточно ресурсов
            if helper_energy.level > 40.0 && helper_balance > 10.0 {
                helper_energy.level -= 5.0;
                h.wallet.spend(2.0).await;

                println!("🤝 {} помогает {}", h.name, weak_name);
            }
        }
    }
}
