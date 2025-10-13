use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub prev_hash: String,
    pub timestamp: i64,
    pub data_root: String,  // ссылка на DataChain
    pub key_root: String,   // ссылка на KeyChain
    pub validator: String,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, prev_hash: String, data_root: String, key_root: String, validator: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}{}{}", index, &prev_hash, timestamp, &data_root, &key_root));
        let hash = format!("{:x}", hasher.finalize());

        Block { index, prev_hash, timestamp, data_root, key_root, validator, hash }
    }
}