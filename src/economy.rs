use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct NetworkFund {
    pub total: Arc<Mutex<f64>>,
}

impl NetworkFund {
    pub fn new() -> Self {
        Self {
            total: Arc::new(Mutex::new(0.0)),
        }
    }

    pub async fn add(&self, amount: f64) {
        let mut fund = self.total.lock().await;
        *fund += amount;
        println!("🏦 Фонд развития пополнен на {:.2} (всего: {:.2})", amount, *fund);
    }

    pub async fn get_balance(&self) -> f64 {
        *self.total.lock().await
    }
}
