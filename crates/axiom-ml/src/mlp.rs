use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpModel {
    pub w1: Vec<Vec<f64>>,
    pub b1: Vec<f64>,
    pub w2: Vec<f64>,
    pub b2: f64,
}

impl MlpModel {
    pub fn new(input: usize, hidden: usize) -> Self {
        Self {
            w1: vec![vec![0.01; input]; hidden],
            b1: vec![0.0; hidden],
            w2: vec![0.01; hidden],
            b2: 0.0,
        }
    }

    fn relu(x: f64) -> f64 {
        x.max(0.0)
    }

    pub fn predict(&self, x: &[f64]) -> f64 {
        let mut h = vec![0.0; self.b1.len()];
        for (i, hi) in h.iter_mut().enumerate() {
            let mut s = self.b1[i];
            for (j, xj) in x.iter().enumerate() {
                s += self.w1[i][j] * xj;
            }
            *hi = Self::relu(s);
        }
        let mut out = self.b2;
        for (i, hi) in h.iter().enumerate() {
            out += self.w2[i] * hi;
        }
        out
    }
}
