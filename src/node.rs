use crate::synapse::SynapseChain;
use crate::chain::Chain;
use crate::neuron::Neuron;
use crate::energy::Energy;
use rand::{thread_rng, Rng};
use serde_json;
use std::sync::{Arc, Mutex};
use log::info;
use serde_json::json; 



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
}

impl Node {
    
    // === Создание новой ноды ===
    pub fn new(name: &str) -> Self {
        Self {
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
        }
    }

    /// Экспортируем нейроны в JSON для обмена между нодами
    pub fn export_neurons_json(&self) -> String {
        let neurons = self.neurons.lock().unwrap();
        serde_json::to_string(&*neurons).unwrap_or_else(|_| "[]".into())
    }

    /// Импортируем нейроны из JSON (обновляем внутреннюю структуру)
    pub fn import_neurons_json(&mut self, json_data: &str) {
        if let Ok(neurons) = serde_json::from_str::<Vec<Neuron>>(json_data) {
            println!("🧬 Импортировано {} нейронов", neurons.len());
            *self.neurons.lock().unwrap() = neurons;
        } else {
            println!("⚠️ Ошибка при импорте нейронов");
        }
    }
    
    pub fn latest_block_json(&self) -> String {
        if let Some(b) = self.data_chain.lock().unwrap().blocks.last() {
            serde_json::to_string(&json!({
                "index": b.index,
                "data_root": b.data_root,
                "key_root": b.key_root,
                "validator": b.validator,
                "hash": b.hash
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
        }
    }
    // === Симуляция добычи данных ===
    pub fn mine_data(&self) -> (String, String) {
        let data_root = format!("{:x}", rand::random::<u64>());
        let key_root = format!("{:x}", rand::random::<u64>());
        (data_root, key_root)
    }

    // === Завершение блока ===
    pub fn finalize_keyblock(&self, data_root: String, key_root: String, winner: &str) {
        let mut kchain = self.key_chain.lock().unwrap();
        kchain.add_block(data_root, key_root, winner.to_string());
        println!("✅ Валидатор {} закрыл KeyBlock", winner);
    }

    // === Получение последних блоков для синхронизации ===
    pub fn last_blocks_json(&self) -> String {
        let d = self.data_chain.lock().unwrap();
        let k = self.key_chain.lock().unwrap();
        serde_json::json!({
            "data_chain": d.blocks,
            "key_chain": k.blocks,
        })
        .to_string()
    }

    // === Получение блоков от сети ===
    pub fn add_blocks_from_json(&self, json_str: String) {
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        let mut d = self.data_chain.lock().unwrap();
        let mut k = self.key_chain.lock().unwrap();
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
    pub fn try_commit_keyblock(&mut self, data_root: String, key_root: String) -> (u64, bool) {
        let mut energy = self.energy.lock().unwrap();

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
                let chain = self.data_chain.lock().unwrap();
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
            let mut synapses = self.synapse_chain.lock().unwrap();
            if neuron.id > 0 {
                let weight: f64 = rand::random();
                synapses.connect(neuron.id - 1, neuron.id, weight);
                synapses.save_to_file().unwrap_or_else(|e| println!("⚠️ Ошибка сохранения SynapseChain: {}", e));

                println!("🔗 Создан синапс: {} -> {} (вес {:.4})", neuron.id - 1, neuron.id, weight);
            }

            let neuron_json = serde_json::to_string(&neuron).unwrap();

            {
                let mut dchain = self.data_chain.lock().unwrap();
                dchain.add_block(neuron_json, "key_placeholder".into(), self.name.clone());
            }

            {
                let mut kchain = self.key_chain.lock().unwrap();
                kchain.add_block("data_hash".into(), key_root.clone(), self.name.clone());
            }

            println!("✅ Создан новый блок Data+Key (нейрон обучен)");

            energy.restore(10.0);
             
            println!("⚡ {} восстановил энергию: {:.2}", self.name, energy.level);
             

            // Попробуем помочь соседу (случайно)
            use rand::seq::SliceRandom;
            let mut rng = thread_rng();
            let maybe_peer = ["Node-4000", "Node-4001", "Node-4002"].choose(&mut rng).unwrap();

            if maybe_peer != &self.name {
                println!("🤝 {} ищет возможность помочь {}", self.name, maybe_peer);
                // Здесь можно будет подключить обмен по сети, а пока — просто вывод
            }
        }
 
        if energy.level > 100.0 {
            energy.level = 100.0;
        }
        let mut rng = thread_rng();
        let commit_value: u64 = rng.gen_range(1..=1000);
        info!("⚔️ {} участвует в PoC, значение: {}", self.name, commit_value); 
        
        // ✅ возвращаем в самом конце
        (commit_value, true)
    }
    pub fn try_merge_chain_json(&mut self, json: String) {
        if let Ok(other_chain) = serde_json::from_str::<crate::chain::Chain>(&json) {
            // 🔒 Получаем доступ к заблокированной цепочке
            let mut current_chain = self.data_chain.lock().unwrap();

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
    pub fn share_energy(&mut self, target: &mut Node) {

        let mut my_energy = self.energy.lock().unwrap();
        let mut target_energy = target.energy.lock().unwrap();

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
}