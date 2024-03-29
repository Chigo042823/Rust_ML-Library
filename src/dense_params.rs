use rand::{thread_rng, Rng};
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseParams {
    pub nodes_in: usize,
    pub nodes_out: usize,
    pub outputs: Vec<f64>, //nodes out
    pub inputs: Vec<f64>,
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
}

impl DenseParams {
    pub fn new(
        nodes_in: usize,
        nodes_out: usize,
    ) -> Self {
        let weights = vec![vec![0.0; nodes_out]; nodes_in];
        let biases = vec![0.0; nodes_out];
        let mut params = DenseParams {
            nodes_in,
            nodes_out,
            outputs: vec![],
            inputs: vec![],
            weights,
            biases,
        };
        params.init();
        params
    }
    pub fn init(&mut self) {
        for i in 0..self.weights.len() {
            for j in 0..self.weights[i].len() {
                self.weights[i][j] = thread_rng().gen_range(-1.0..1.0); //in (rows) - out (cols)
            }
        }
        for i in 0..self.biases.len() {
            self.biases[i] = thread_rng().gen_range(-1.0..1.0); //nodes out
        }
    }
}