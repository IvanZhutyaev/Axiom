use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearModel {
    pub weights: Vec<f64>,
    pub bias: f64,
}

impl Default for LinearModel {
    fn default() -> Self {
        Self {
            weights: Vec::new(),
            bias: 0.0,
        }
    }
}

impl LinearModel {
    pub fn predict(&self, features: &[f64]) -> f64 {
        let mut s = self.bias;
        for (w, x) in self.weights.iter().zip(features) {
            s += w * x;
        }
        s
    }

    pub fn train_step(&mut self, x: &[f64], y: f64, lr: f64) {
        if self.weights.len() < x.len() {
            self.weights.resize(x.len(), 0.0);
        }
        let pred = self.predict(x);
        let err = pred - y;
        for (w, xi) in self.weights.iter_mut().zip(x) {
            *w -= lr * err * xi;
        }
        self.bias -= lr * err;
    }
}
