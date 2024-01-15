#[derive(Debug, Clone, thiserror::Error)]
pub enum WeightError {
    #[error("Weights do not sum to 1.0")]
    InvalidSum,
    #[error("Weights length is invalid; expected {0}, got {1}")]
    InvalidLength(usize, usize),
}

#[derive(Debug, Clone)]
pub struct InterporationWeight {
    nvoices: usize,

    duration: Weights,
    parameter: Vec<Weights>,
    gv: Vec<Weights>,
}

impl Default for InterporationWeight {
    fn default() -> Self {
        Self::new(1, 0)
    }
}

impl InterporationWeight {
    pub fn new(nvoices: usize, nstream: usize) -> Self {
        let default_weight = Weights::average(nvoices);
        Self {
            nvoices,
            duration: default_weight.clone(),
            parameter: vec![default_weight.clone(); nstream],
            gv: vec![default_weight.clone(); nstream],
        }
    }

    /// Set duration weight
    /// weights.len() == nvoices
    /// weights.iter().sum() == 1.0
    pub fn set_duration(&mut self, weights: Weights) -> Result<(), WeightError> {
        weights.assert_length(self.nvoices)?;
        self.duration = weights;
        Ok(())
    }
    /// Set parameter weight
    /// weights.len() == nvoices
    /// weights.iter().sum() == 1.0
    pub fn set_parameter(
        &mut self,
        stream_index: usize,
        weights: Weights,
    ) -> Result<(), WeightError> {
        weights.assert_length(self.nvoices)?;
        self.parameter[stream_index] = weights;
        Ok(())
    }
    /// Set GV weight
    /// weights.len() == nvoices
    /// weights.iter().sum() == 1.0
    pub fn set_gv(&mut self, stream_index: usize, weights: Weights) -> Result<(), WeightError> {
        weights.assert_length(self.nvoices)?;
        self.gv[stream_index] = weights;
        Ok(())
    }

    /// Get duration weight
    pub fn get_duration(&self) -> &Weights {
        &self.duration
    }
    /// Get parameter weight
    pub fn get_parameter(&self, stream_index: usize) -> &Weights {
        &self.parameter[stream_index]
    }
    /// Get GV weight
    pub fn get_gv(&self, stream_index: usize) -> &Weights {
        &self.gv[stream_index]
    }
}

#[derive(Debug, Clone)]
pub struct Weights {
    weights: Vec<f64>,
}

impl Weights {
    pub fn new(weight: &[f64]) -> Result<Self, WeightError> {
        let sum: f64 = weight.iter().sum();
        if approx::abs_diff_ne!(sum, 1.0) {
            return Err(WeightError::InvalidSum);
        }
        Ok(Self {
            weights: weight.to_vec(),
        })
    }

    pub fn get_weights(&self) -> &[f64] {
        &self.weights
    }

    fn average(nvoices: usize) -> Self {
        let average_weight = 1.0f64 / nvoices as f64;
        Self {
            weights: vec![average_weight; nvoices],
        }
    }

    fn assert_length(&self, length: usize) -> Result<(), WeightError> {
        if self.weights.len() != length {
            return Err(WeightError::InvalidLength(length, self.weights.len()));
        }
        Ok(())
    }
}
