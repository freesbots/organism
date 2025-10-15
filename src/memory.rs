use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::cmp::Reverse;
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainEvent {
    pub timestamp: i64,
    pub action: String,
    pub context: String,
    pub result: f64,
}

impl BrainEvent {
    pub fn new(action: &str, context: &str, result: f64) -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp(),
            action: action.to_string(),
            context: context.to_string(),
            result,
        }
    }
}
 
#[derive(Clone)]
pub struct Memory {
    pub short: Arc<Mutex<Vec<BrainEvent>>>, // recent events (bounded)
    pub long: Arc<Mutex<Vec<BrainEvent>>>,  // aggregated history (append-only, trim if нужно)
    pub max_short: usize,
    pub max_long: usize,
    pub retention_time: i64,
}

impl Memory { 
    pub fn new(max_short: usize, max_long: usize, retention_time: i64) -> Self {
        Self { 
            short: Arc::new(Mutex::new(Vec::new())),
            long: Arc::new(Mutex::new(Vec::new())),
            max_short,
            max_long,
            retention_time,
        }
    }

    pub async fn record(&self, event: BrainEvent) {
        self.add_event(event).await;
    }
    // 💾 Добавление события в память
    pub async fn add_event(&self, mut event: BrainEvent) {
        let now = Utc::now().timestamp();
        event.timestamp = now;

        println!("🧠 [Memory::add_event] Добавлено событие: {} | {} | {:.2}",
        event.action, event.context, event.result);

        // short
        {
            let mut s = self.short.lock().await;
            s.push(event.clone());
            let len = s.len();
            if len > self.max_short {
                let excess = len - self.max_short;
                s.drain(0..excess);
            }
            // удаляем старые записи по времени
            s.retain(|e| now - e.timestamp < self.retention_time);
        }

        // long: добавляем только позитивные события, очищаем по лимиту
        if event.result > 0.8 {
            let mut l = self.long.lock().await;
            l.push(event);
            let len = l.len();
            if len > self.max_long {
                let excess = len - self.max_long;
                l.drain(0..excess);
            }
        } else if event.result < 0.3 {
            // негативные — убираем из долгой памяти (политика)
            let mut l = self.long.lock().await;
            l.retain(|e| e.result > 0.3);
        }
    }

     
    /// 🧩 Получение последних N событий из короткой памяти
    pub async fn get_recent(&self, count: usize) -> Vec<BrainEvent> {
        // берем копию под локом
        let events: Vec<BrainEvent> = {
            let s = self.short.lock().await;
            s.iter().rev().take(count).cloned().collect()
        };
        // мьютекс уже освобожден
        events
    }

    /// 🧩 Получение последних N событий из долгосрочной памяти
    pub async fn get_long(&self, count: usize) -> Vec<BrainEvent> {
        let events: Vec<BrainEvent> = {
            let l = self.long.lock().await;
            l.iter().rev().take(count).cloned().collect()
        };
        // мьютекс уже освобожден
        events
    }

    /// 🧮 Среднее значение result последних N событий
    pub async fn average_result(&self, count: usize) -> f64 {
        let s = self.short.lock().await;
        if s.is_empty() {
            return 0.0;
        }
        let take_n = usize::min(count, s.len());
        let sum: f64 = s.iter().rev().take(take_n).map(|e| e.result).sum();
        sum / (take_n as f64)
    }

    /// 🧠 Комбинированная память — объединяет short + long память
    pub async fn get_combined_memory(&self, count: usize) -> Vec<BrainEvent> {
        let half = count / 2;
        let mut combined = self.get_recent(half).await;
        let mut long = self.get_long(count - half).await;
        combined.append(&mut long);
        // сортируем по убыванию timestamp: используем Reverse
        combined.sort_by_key(|e| Reverse(e.timestamp));
        combined
    }
}
