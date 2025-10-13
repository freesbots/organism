use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    pub from_id: u64,
    pub to_id: u64,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseChain {
    pub synapses: Vec<Synapse>,
}

impl SynapseChain {
    pub fn new() -> Self {
        Self { synapses: Vec::new() }
    }

    pub fn connect(&mut self, from_id: u64, to_id: u64, weight: f64) {
        let synapse = Synapse { from_id, to_id, weight };
        self.synapses.push(synapse);
    }

    pub fn print(&self) {
        println!("🔗 SynapseChain connections:");
        for s in &self.synapses {
            println!("  {} => {} [w = {:.4}]", s.from_id, s.to_id, s.weight);
        }
    }
    pub fn save_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.synapses)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("synapse_chain.json")?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    pub fn load_from_file() -> Self {
        if let Ok(mut file) = File::open("synapse_chain.json") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(synapses) = serde_json::from_str(&content) {
                    println!("♻️ Загружен SynapseChain из файла");
                    return Self { synapses };
                }
            }
        }
        println!("🧬 Создан новый SynapseChain");
        Self { synapses: vec![] }
    }
}