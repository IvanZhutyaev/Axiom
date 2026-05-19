use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStub {
    pub threshold: f64,
    pub left: f64,
    pub right: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamingGbdt {
    pub trees: Vec<TreeStub>,
    pub learning_rate: f64,
}

impl StreamingGbdt {
    pub fn predict(&self, x: f64) -> f64 {
        let mut pred = 0.0;
        for t in &self.trees {
            pred += self.learning_rate * if x <= t.threshold { t.left } else { t.right };
        }
        pred
    }

    pub fn add_tree(&mut self, tree: TreeStub) {
        self.trees.push(tree);
    }
}
