use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SolveTspData {
    distances: Vec<Vec<f64>>,
    n_generations: usize,
}

impl SolveTspData {
    pub fn new(distances: Vec<Vec<f64>>, n_generations: usize) -> Self {
        Self {
            distances,
            n_generations,
        }
    }
}
