use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub id: u64,
    pub value: f64,
    pub weights: Vec<f64>,
}

impl Neuron {
    pub fn new(id: u64, input_size: usize) -> Self {
        Self {
            id,
            value: 0.0,
            weights: (0..input_size).map(|_| rand::random::<f64>() * 2.0 - 1.0).collect(),
        }
    }

    pub fn activate(&mut self, inputs: &[f64]) -> f64 {
        let sum: f64 = inputs.iter().zip(&self.weights).map(|(i, w)| i * w).sum();
        self.value = 1.0 / (1.0 + (-sum).exp()); // сигмоида
        self.value
    }

    /// Простой шаг обучения (стохастический градиент для одной нейроны)
    /// Возвращает квадратичную ошибку 0.5*(expected - output)^2
    pub fn learn(&mut self, inputs: &[f64], expected: f64, lr: f64) -> f64 {
        let output = self.activate(inputs);
        let error = expected - output;
        // производная сигмоиды: output * (1 - output)
        let grad = error * output * (1.0 - output);
        for (w, &x) in self.weights.iter_mut().zip(inputs.iter()) {
            *w += lr * grad * x;
        }
        0.5 * error * error
    }
}