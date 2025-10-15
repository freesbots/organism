use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Wallet {
    pub balance: Arc<Mutex<f64>>,
}

impl Wallet {
    pub fn new() -> Self {
        Self {
            balance: Arc::new(Mutex::new(0.0)),
        }
    }

    pub async fn deposit(&self, amount: f64) {
        let mut balance = self.balance.lock().await;
        *balance += amount;
    }

    pub async fn withdraw(&self, amount: f64) -> bool {
        let mut balance = self.balance.lock().await;
        if *balance >= amount {
            *balance -= amount;
            true
        } else {
            false
        }
    }

    /// 💰 Начислить токены
    pub async fn reward(&self, amount: f64) {
        let mut b = self.balance.lock().await;
        *b += amount;
        println!("💎 Кошелёк пополнен на {:.2} токенов (всего: {:.2})", amount, *b);
    }

    /// 💸 Списать токены
    pub async fn spend(&self, amount: f64) -> bool {
        let mut b = self.balance.lock().await;
        if *b >= amount {
            *b -= amount;
            println!("💸 Списано {:.2} токенов (остаток: {:.2})", amount, *b);
            true
        } else {
            println!("⚠️ Недостаточно средств (нужно {:.2}, есть {:.2})", amount, *b);
            false
        }
    }

    /// Проверить баланс
    pub async fn get_balance(&self) -> f64 {
        *self.balance.lock().await
    }
}
