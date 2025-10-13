
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};


#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub index: u64,
    pub data_root: String,
    pub key_root: String,
    pub validator: String,
    pub hash: String, // 🔥 добавляем
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Chain {
    pub name: String,
    pub blocks: Vec<Block>,
}

impl Chain {
    pub fn new(name: &str) -> Self {
        let path = format!("{}_chain.json", name);
        if let Ok(chain) = Self::load_from_file(&path) {
            println!("♻️ Загружена существующая цепь: {}", name);
            return chain;
        }

        println!("🆕 Создана новая цепь: {}", name);
        Self {
            name: name.to_string(),
            blocks: vec![Block {
                index: 0,
                data_root: "genesis".into(),
                key_root: "genesis".into(),
                validator: "system".into(),
                hash: "genesis_hash".into(), // ✅
            }],
        }
    }

    pub fn last_hash(&self) -> String {
        self.blocks.last().unwrap().hash.clone()
    }

    pub fn add_block(&mut self, data_root: String, key_root: String, validator: String) {
        let index = self.blocks.len() as u64;
        let hash_input = format!("{}{}{}{}", index, data_root, key_root, validator);
        let hash = format!("{:x}", md5::compute(hash_input));

        let block = Block {
            index,
            data_root,
            key_root,
            validator,
            hash, // теперь это переменная
        };

        self.blocks.push(block);
    }

    pub fn save_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let filename = format!("{}_chain.json", self.name);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filename)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn save_block(&self, block: &Block) {
        let filename = format!("data/blocks/{}_{}.json", self.name, block.index);
        let json = serde_json::to_string_pretty(&block).unwrap();
        let mut f = File::create(filename).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let chain: Chain = serde_json::from_str(&content)?;
        Ok(chain)
    }

    pub fn print_chain(&self) {
        println!("\n==== {} ====", self.name);
        for b in &self.blocks {
            println!("#{} [{}] {}", b.index, b.validator, b.hash);
        }
    }
}