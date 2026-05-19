use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetector {
    pub mean: f64,
    pub variance: f64,
    pub n: u64,
    pub threshold_sigma: f64,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self {
            mean: 0.0,
            variance: 1.0,
            n: 0,
            threshold_sigma: 3.0,
        }
    }
}

impl AnomalyDetector {
    pub fn update(&mut self, x: f64) {
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as f64;
        let delta2 = x - self.mean;
        self.variance += delta * delta2;
    }

    pub fn is_anomaly(&self, x: f64) -> bool {
        if self.n < 2 {
            return false;
        }
        let var = self.variance / (self.n - 1) as f64;
        let std = var.sqrt().max(1e-9);
        (x - self.mean).abs() > self.threshold_sigma * std
    }
}
