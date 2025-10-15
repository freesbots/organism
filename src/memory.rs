use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BrainEvent {
    pub timestamp: u64,
    pub action: String,
    pub context: String,
    pub result: f64,
}

impl BrainEvent {
    pub fn new(action: &str, context: &str, result: f64) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            action: action.to_string(),
            context: context.to_string(),
            result,
        }
    }
}

#[derive(Clone)]
pub struct Memory {
    pub history: Arc<Mutex<Vec<BrainEvent>>>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn record(&self, event: BrainEvent) {
        let mut hist = self.history.lock().await;
        hist.push(event);
        if hist.len() > 1000 {
            hist.remove(0);
        }
    }

    pub async fn get_recent(&self, limit: usize) -> Vec<BrainEvent> {
        let hist = self.history.lock().await;
        let len = hist.len();
        hist.iter().skip(len.saturating_sub(limit)).cloned().collect()
    }

    pub async fn average_result(&self, last_n: usize) -> f64 {
        let hist = self.history.lock().await;
        let len = hist.len();
        let slice = hist.iter().skip(len.saturating_sub(last_n));
        let (sum, count) = slice.fold((0.0, 0), |(s, c), e| (s + e.result, c + 1));
        if count > 0 { sum / count as f64 } else { 0.0 }
    }
}
