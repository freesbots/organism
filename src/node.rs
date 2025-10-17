use crate::synapse::SynapseChain;
use crate::chain::Chain;
use crate::neuron::Neuron;
use crate::energy::Energy;
use crate::wallet::Wallet; 
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::info;
use serde_json::json;   
use crate::interaction::NetworkBus;
use crate::interaction::Message;

const DECAY_PER_TICK: f64 = 1.0;
const REPLICATION_THRESHOLD: f64 = 1.0; // вместо 80
const REPRODUCTION_COST: f64 = 0.0;
const MUTATION_RATE: f64 = 0.05;




#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub energy: Arc<Mutex<Energy>>, // ✅ теперь не f64, а полноценная энергия
    pub efficiency: f64,
    pub altruism: f64,
    pub resilience: f64,
    pub experience: f64,
    pub data_chain: Arc<Mutex<Chain>>,
    pub key_chain: Arc<Mutex<Chain>>,
    pub synapse_chain: Arc<Mutex<SynapseChain>>,
    pub connections: Arc<Mutex<Vec<String>>>, // ✅ добавляем
    pub neurons: Arc<Mutex<Vec<Neuron>>>,     // ✅ добавляем
    pub wallet: Wallet,
    pub rng: Arc<Mutex<StdRng>>,
}

impl Node {
    
    // === Создание новой ноды ===
    pub fn new(name: &str) -> Arc<Mutex<Node>> {
        Arc::new(Mutex::new(Node {
            name: name.to_string(),
            energy: Arc::new(Mutex::new(Energy::new(name))),
            efficiency: 1.0,
            altruism: 0.5,
            resilience: 0.5,
            experience: 0.0,
            data_chain: Arc::new(Mutex::new(Chain::new(&format!("{}_data", name)))),
            key_chain: Arc::new(Mutex::new(Chain::new(&format!("{}_key", name)))),
            synapse_chain: Arc::new(Mutex::new(SynapseChain::new())),
            connections: Arc::new(Mutex::new(vec![])),
            neurons: Arc::new(Mutex::new(vec![])),
            wallet: Wallet::new(),
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        }))
        /* Self {
            name: name.to_string(),
            energy: Arc::new(Mutex::new(Energy::new(name))), // ✅ теперь правильно
            efficiency: 1.0,
            altruism: 0.5,
            resilience: 1.0,
            experience: 0.0,
            data_chain: Arc::new(Mutex::new(Chain::new(&format!("{}_data", name)))),
            key_chain: Arc::new(Mutex::new(Chain::new(&format!("{}_key", name)))),
            synapse_chain: Arc::new(Mutex::new(SynapseChain::new())),
            connections: Arc::new(Mutex::new(vec![])),
            neurons: Arc::new(Mutex::new(vec![])),
            wallet: Wallet::new(),
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        } */
    }

    /// Экспортируем нейроны в JSON для обмена между нодами
    pub async fn export_neurons_json(&self) -> String {
        let neurons = self.neurons.lock().await;
        serde_json::to_string(&*neurons).unwrap_or_else(|_| "[]".into())
    }

    /// Импортируем нейроны из JSON (обновляем внутреннюю структуру)
    pub async fn import_neurons_json(&mut self, json_data: &str) {
        if let Ok(neurons) = serde_json::from_str::<Vec<Neuron>>(json_data) {
            println!("🧬 Импортировано {} нейронов", neurons.len());
            *self.neurons.lock().await = neurons;
        } else {
            println!("⚠️ Ошибка при импорте нейронов");
        }
    }
    pub fn mine_block(&mut self) {
        println!("⛏️  Node {} mined a new block!", self.name);
        // Здесь можешь добавить работу с цепочкой, энергией и т.п.
        self.wallet.deposit(2.0); // награда за блок
        println!("💰 {} получил 2 токена за добычу блока!", self.name);

    }
    pub async fn get_chain_summary(&self) -> Vec<String> {
        let chain = self.data_chain.lock().await; // асинхронный захват блокировки

            chain
                .blocks
                .iter()
                .map(|b| format!("Block {}: {}", b.index, b.hash))
                .collect()
    }
    
    pub async fn latest_block_json(&self) -> String {
        if let Some(last_block) = self.data_chain.lock().await.blocks.last() {
            serde_json::to_string(&json!({
                "index": last_block.index,
                "data_root": last_block.data_root,
                "key_root": last_block.key_root,
                "validator": last_block.validator,
                "hash": last_block.hash
            })).unwrap()
        } else {
            "{}".into()
        }
    }

    // === Клон для сети ===
    pub fn clone_for_net(&self) -> Self {
        Self {
            name: self.name.clone(),
            energy: Arc::clone(&self.energy),
            efficiency: self.efficiency,
            altruism: self.altruism,
            resilience: self.resilience,
            experience: self.experience,
            data_chain: Arc::clone(&self.data_chain),
            key_chain: Arc::clone(&self.key_chain),
            synapse_chain: Arc::clone(&self.synapse_chain),
            connections: Arc::clone(&self.connections),
            neurons: Arc::clone(&self.neurons), 
            wallet: self.wallet.clone(),
            rng: Arc::clone(&self.rng),
        }
    }
    // === Симуляция добычи данных ===
    pub async fn mine_data(&self) -> (String, String) {
        let data_root = format!("{:x}", rand::random::<u64>());
        let key_root = format!("{:x}", rand::random::<u64>());
        (data_root, key_root)
    }

    // === Завершение блока ===
    pub async fn finalize_keyblock(&self, data_root: String, key_root: String, winner: &str) {
        let mut kchain = self.key_chain.lock().await;
        kchain.add_block(data_root, key_root, winner.to_string());
        println!("✅ Валидатор {} закрыл KeyBlock", winner);
    }

    // === Получение последних блоков для синхронизации ===
    pub async fn last_blocks_json(&self) -> String {
        let d = self.data_chain.lock().await;
        let k = self.key_chain.lock().await;
        serde_json::json!({
            "data_chain": d.blocks,
            "key_chain": k.blocks,
        })
        .to_string()
    }

    // === Получение блоков от сети ===
    pub async fn add_blocks_from_json(&self, json_str: String) {
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        let mut d = self.data_chain.lock().await;
        let mut k = self.key_chain.lock().await;
        if let Some(data_blocks) = parsed["data_chain"].as_array() {
            for b in data_blocks {
                if let Some(val) = b["validator"].as_str() {
                    d.add_block("sync".into(), "sync".into(), val.to_string());
                }
            }
        }
        if let Some(key_blocks) = parsed["key_chain"].as_array() {
            for b in key_blocks {
                if let Some(val) = b["validator"].as_str() {
                    k.add_block("sync".into(), "sync".into(), val.to_string());
                }
            }
        }
        println!("🔗 Нода {} синхронизировала блоки из сети", self.name);
    }
    

    // === Основной PoC и обучение нейронов ===
    pub async fn try_commit_keyblock(&mut self, data_root: String, key_root: String) -> (u64, bool) {
        let mut energy = self.energy.lock().await;

        if energy.level <= 0.0 {
            println!("😴 {} устал и не участвует", self.name);
            return (0, false);
        }

        energy.consume(5.0);
        energy.restore(0.5);
        println!("🧠 {} обучает DataChain... (энергия = {:.2})", self.name, energy.level);
         

        let proof_value: u64 = rand::random::<u64>() % 500;
        println!("⚔️ {} участвует в PoC, значение: {}", self.name, proof_value);
        println!("⚙️ {} предложил commit {}", self.name, proof_value);

        if proof_value > 250 {
            println!("👑 Победитель PoC — {}", self.name);

            let chain_len = {
                let chain = self.data_chain.lock().await;
                chain.blocks.len()
            };

            let mut neuron = Neuron::new(chain_len as u64, 3);
            let inputs = vec![rand::random::<f64>(), rand::random::<f64>(), rand::random::<f64>()];
            let expected = if inputs.iter().sum::<f64>() > 1.5 { 1.0 } else { 0.0 };
            let lr = 0.1;
            let loss = neuron.learn(&inputs, expected, lr);

            println!(
                "🧩 Нейрон #{} => входы: {:?}, выход: {:.4}, ошибка: {:.6}",
                neuron.id, inputs, neuron.value, loss
            );
            // === 🧬 создаём синапс между новым и предыдущим нейроном ===
            let mut synapses = self.synapse_chain.lock().await;
            if neuron.id > 0 {
                let weight: f64 = rand::random();
                synapses.connect(neuron.id - 1, neuron.id, weight);
                synapses.save_to_file().unwrap_or_else(|e| println!("⚠️ Ошибка сохранения SynapseChain: {}", e));

                println!("🔗 Создан синапс: {} -> {} (вес {:.4})", neuron.id - 1, neuron.id, weight);
            }

            let neuron_json = serde_json::to_string(&neuron).unwrap();

            {
                let mut dchain = self.data_chain.lock().await;
                dchain.add_block(neuron_json, "key_placeholder".into(), self.name.clone());
            }

            {
                let mut kchain = self.key_chain.lock().await;
                kchain.add_block("data_hash".into(), key_root.clone(), self.name.clone());
            }

            println!("✅ Создан новый блок Data+Key (нейрон обучен)");

            energy.restore(10.0);
             
            println!("⚡ {} восстановил энергию: {:.2}", self.name, energy.level);
             

            // Попробуем помочь соседу (случайно)
            use rand::seq::SliceRandom;
            let mut rng = StdRng::from_entropy();
            let maybe_peer = ["Node-4000", "Node-4001", "Node-4002"].choose(&mut rng).unwrap();

            if maybe_peer != &self.name {
                println!("🤝 {} ищет возможность помочь {}", self.name, maybe_peer);
                // Здесь можно будет подключить обмен по сети, а пока — просто вывод
            }
        }
 
        if energy.level > 40.0 {
            // шанс размножиться растёт с энергией
            let chance = (energy.level / 200.0).clamp(0.05, 0.5); // 5–50%
            if rand::random::<f64>() < chance {
                println!("🧬 [tick] {} создаёт потомка (шанс {:.2})", self.name, chance);
                // вызываем логику создания цепи
                // ...
            }
        }
        let mut rng = StdRng::from_entropy();
        let commit_value: u64 = rng.gen_range(1..=1000); 
        info!("⚔️ {} участвует в PoC, значение: {}", self.name, commit_value); 
        
        // ✅ возвращаем в самом конце
        (commit_value, true)
    }
    pub async fn try_merge_chain_json(&mut self, json: String) {
        if let Ok(other_chain) = serde_json::from_str::<crate::chain::Chain>(&json) {
            // 🔒 Получаем доступ к заблокированной цепочке
            let mut current_chain = self.data_chain.lock().await;

            if other_chain.blocks.len() > current_chain.blocks.len() {
                println!(
                    "🔄 Обновляем локальную цепочку: {} → {} блоков",
                    current_chain.blocks.len(),
                    other_chain.blocks.len()
                );

                *current_chain = other_chain;
                let _ = current_chain.save_to_file();
            } else {
                println!(
                    "✅ Локальная цепочка актуальна ({} блоков)",
                    current_chain.blocks.len()
                );
            }
        } else {
            println!("⚠️ Ошибка при десериализации цепочки для синхронизации");
        }
    }

    // === Энергообмен между нодами ===
    pub async fn share_energy(&mut self, target: &mut Node) {

        let mut my_energy = self.energy.lock().await;
        let mut target_energy = target.energy.lock().await;

        // Минимальный порог для помощи
        if my_energy.level < 30.0 {
            println!("💤 {} слишком слаб, чтобы делиться энергией", self.name);
            return;
        }

        // Помогаем, если у цели меньше 20
        if target_energy.level < 20.0 {
            let transfer = (my_energy.level * 0.1).min(10.0);
            my_energy.consume(transfer);
            target_energy.restore(transfer);

            println!(
                "🔋 {} передал {:.2} энергии ноде {} (теперь у {}: {:.2}, у {}: {:.2})",
                self.name,
                transfer,
                target.name,
                self.name,
                my_energy.level,
                target.name,
                target_energy.level
            );
        }
    }

    /// Один "шаг жизни" узла — метаболизм, действие, обучение, репликация
    pub async fn tick(
        node_arc: Arc<Mutex<Node>>,
        net: Arc<NetworkBus>,
        nodes_ref: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
        tick_counter: u64,
    ) -> Option<Arc<Mutex<Node>>> {
        let mut guard = node_arc.lock().await;
        guard.tick_node(net, nodes_ref, tick_counter).await
    }

    pub async fn tick_node(
        &mut self,
        net: Arc<NetworkBus>,
        nodes_ref: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
        tick_counter: u64,
    ) -> Option<Arc<Mutex<Node>>> {
        // === 1. Энергетический decay ===
        {
            let mut e = self.energy.lock().await;
            e.level = (e.level - DECAY_PER_TICK).max(0.0);
        }

        // === 2. Смерть при нехватке энергии ===
        if self.energy.lock().await.level <= 0.0 {
            println!("☠️ Node {} died at tick {}", self.name, tick_counter);
            return None;
        }

        // === 3. Поведение: помощь или работа === 
        let mut action = String::from("idle");
        let energy_level = { 
            let e = self.energy.lock().await;
            e.level
        };

         
        let mut rng = StdRng::from_entropy(); // создаём RNG уже после await
        if rng.gen::<f64>() < self.altruism && energy_level > 1.0 {
            // сотрудничество — передать немного энергии
            let node_list_copy: Vec<Arc<Mutex<Node>>> = {
                let nodes_locked = nodes_ref.lock().await;
                nodes_locked.clone()
            };

            let mut candidates: Vec<Arc<Mutex<Node>>> = Vec::new();
            for node_ref in node_list_copy.iter() {
                if let Ok(node_guard) = node_ref.try_lock() {  
                    let energy_alive = node_guard.energy.lock().await.level > 0.0;
                    if node_guard.name != self.name && energy_alive {
                        candidates.push(node_ref.clone());
                    }
                }
            }

            if !candidates.is_empty() {
                let target_arc = {
                    let mut rng = StdRng::from_entropy();
                    candidates[rng.gen_range(0..candidates.len())].clone()
                };

                if target_arc.lock().await.name == self.name {
                    return None; // не отправляем сообщение самому себе
                }
                let target_name = target_arc.lock().await.name.clone();
                let msg = Message::new_energy_transfer(&self.name, &target_name, 5.0);
                net.send(msg).await;

                let mut my_energy = self.energy.lock().await;
                my_energy.level = (my_energy.level - 1.0).max(0.0);
                println!("🔋 {} shared energy with {}", self.name, target_name);
            }
        } else {
            // работа — получить награду
            let reward = rng.gen_range(2.0..5.0) * (1.0 + self.efficiency);
            let mut e = self.energy.lock().await;
            e.level += reward;
            action = format!("worked +{:.2}", reward);
        }
        
        // === 4. Обучение ===
        self.local_learn().await;
        
        // === 5. Репликация ===
        let energy_val = self.energy.lock().await.level;
        println!(
            "🔎 [DEBUG] {} energy before replication check = {:.2} (threshold = {:.2})",
            self.name, energy_val, REPLICATION_THRESHOLD
        );
        if energy_val > REPLICATION_THRESHOLD {
             
            let child = self.spawn_child().await;
            {
                
                let mut e = self.energy.lock().await;
                e.level = (e.level - REPRODUCTION_COST).max(0.0);
                
            }
            let (child_name, eff, alt) = {
                let c = child.lock().await;
                (c.name.clone(), c.efficiency, c.altruism)
            };

            println!(
                "🌱 {} replicated -> {} (eff={:.2}, alt={:.2})",
                self.name, child_name, eff, alt
            );
            
            return Some(child);
        }
         
        // === 6. Логирование ===
        println!(
            "🧠 {} action: {} | energy: {:.2}",
            self.name,
            action,
            self.energy.lock().await.level
        );

        None
    }
    async fn local_learn(&mut self) {
        // Простая адаптация altruism на основе текущей энергии
        let energy = self.energy.lock().await.level;
        if energy > 150.0 {
            self.altruism = (self.altruism + 0.002).min(1.0);
        } else if energy < 50.0 {
            self.altruism = (self.altruism - 0.002).max(0.0);
        }
    }
    async fn spawn_child(&self) -> Arc<Mutex<Node>> {

        println!("↪ [tick] node={} before action energy={:.2} altruism={:.2} efficiency={:.2}",  self.name, self.energy.lock().await.level, self.altruism, self.efficiency);
 
        let child_name = format!("{}_child_{}", self.name, chrono::Utc::now().timestamp_millis());

        // `Node::new` уже возвращает Arc<Mutex<Node>>
        let child = Node::new(&child_name);

        {
            let parent_energy = { self.energy.lock().await.level };

            // затем создаём RNG
            let mut rng = StdRng::from_entropy();
            let extra_energy = rng.gen_range(5.0..15.0);

            // теперь lock child и применяем
            let mut child_guard = child.lock().await;
            child_guard.altruism = (self.altruism + rng.gen_range(-MUTATION_RATE..MUTATION_RATE)).clamp(0.0, 1.0);
            let mut child_energy = child_guard.energy.lock().await;
            child_energy.level = parent_energy * 0.3;
            child_energy.level += extra_energy;
        }
        println!("↩ [tick] node={} after action energy={:.2}", self.name, self.energy.lock().await.level);  
        child
    }
}