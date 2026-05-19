//! Inference helpers for AQL `predict()` / PREDICT opcode.

use crate::linear::LinearModel;

pub struct Predictor {
    pub linear: LinearModel,
}

impl Default for Predictor {
    fn default() -> Self {
        Self {
            linear: LinearModel::default(),
        }
    }
}

impl Predictor {
    pub fn infer(&self, features: &[f64]) -> f64 {
        self.linear.predict(features)
    }

    pub fn train_step(&mut self, features: &[f64], label: f64, lr: f64) {
        self.linear.train_step(features, label, lr);
    }
}
