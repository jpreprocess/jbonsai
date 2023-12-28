#[derive(Debug, Clone, Default)]
pub struct InterporationWeight {
    duration: Vec<f64>,
    parameter: Vec<Vec<f64>>,
    gv: Vec<Vec<f64>>,
}

impl InterporationWeight {
    pub fn new(voice_len: usize, nstream: usize) -> Self {
        let average_weight = 1.0f64 / voice_len as f64;
        let default_weight = vec![average_weight; voice_len];
        Self {
            duration: default_weight.clone(),
            parameter: vec![default_weight.clone(); nstream],
            gv: vec![default_weight.clone(); nstream],
        }
    }

    /// weights.len() == nstream
    pub fn set_duration(&mut self, weights: Vec<f64>) {
        Self::assert_weights(&weights);
        self.duration = weights;
    }
    /// weights.len() == nstream
    pub fn set_parameter(&mut self, stream_index: usize, weights: Vec<f64>) {
        Self::assert_weights(&weights);
        self.parameter[stream_index] = weights;
    }
    /// weights.len() == nstream
    pub fn set_gv(&mut self, stream_index: usize, weights: Vec<f64>) {
        Self::assert_weights(&weights);
        self.gv[stream_index] = weights;
    }

    fn assert_weights(weights: &[f64]) {
        let sum: f64 = weights.iter().sum();
        if cfg!(debug_assertions) {
            approx::assert_abs_diff_eq!(sum, 1.0);
        } else {
            if approx::abs_diff_ne!(sum, 1.0) {
                eprintln!("Warn: the sum of weight should be 1.0, but got {}", sum)
            }
        }
    }

    pub fn get_duration(&self) -> &[f64] {
        &self.duration
    }
    pub fn get_parameter(&self, stream_index: usize) -> &[f64] {
        &self.parameter[stream_index]
    }
    pub fn get_gv(&self, stream_index: usize) -> &[f64] {
        &self.gv[stream_index]
    }
}
