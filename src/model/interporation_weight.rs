//! Interpolation weight structures for voice morphing.

use std::ops::Deref;

/// Errors in setting interpolation weight.
#[derive(Debug, Clone, thiserror::Error)]
pub enum WeightError {
    /// The sum of weights is not 1.0.
    #[error("Weights do not sum to 1.0")]
    InvalidSum,
    /// The length of provided weights does not match that of current weights.
    #[error("Weights length is invalid; expected {0}, got {1}")]
    InvalidLength(usize, usize),
}

/// Interpolation weight for voice morphing.
///
/// ## Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Load multiple voices
/// let mut engine = jbonsai::Engine::load(&[
///     "models/tohoku-f01/tohoku-f01-sad.htsvoice",
///     "models/tohoku-f01/tohoku-f01-happy.htsvoice",
/// ])?;
///
/// // Get interpolation weight as mutable reference
/// let iw = engine.condition.get_interporation_weight_mut();
///
/// // Set the same weights for duration.
/// // The resulting duration will be in the middle of `sad` and `happy` style.
/// iw.set_duration(&[0.5, 0.5])?;
///
/// // Stream index 0: MCP (spectrum)
/// iw.set_parameter(0, &[0.5, 0.5])?;
///
/// // Stream index 1: LF0 (log F0)
/// iw.set_parameter(1, &[0.5, 0.5])?;
///
/// // Stream index 2: LPF parameters (LPF)
/// // For LPF, use `sad` style and ignore `happy` style.
/// iw.set_parameter(2, &[1.0, 0.0])?;
/// # Ok(())
/// # }
/// ```
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
    /// Create a new [`InterporationWeight`] which puts equal weights to each voices.
    pub fn new(nvoices: usize, nstream: usize) -> Self {
        let average = Weights::average(nvoices);
        Self {
            nvoices,
            parameter: vec![average.clone(); nstream],
            gv: vec![average.clone(); nstream],
            duration: average,
        }
    }

    /// Set duration weight.
    ///
    /// Conditions:
    /// - weight.len() == nvoices
    /// - weight.iter().sum() == 1.0
    pub fn set_duration(&mut self, weight: &[f64]) -> Result<(), WeightError> {
        let weights = Weights::new(weight)?;
        weights.check_length(self.nvoices)?;
        self.duration = weights;
        Ok(())
    }
    /// Set parameter weight.
    ///
    /// Conditions:
    /// - stream_index < nstream
    /// - weight.len() == nvoices
    /// - weight.iter().sum() == 1.0
    pub fn set_parameter(
        &mut self,
        stream_index: usize,
        weight: &[f64],
    ) -> Result<(), WeightError> {
        let weights = Weights::new(weight)?;
        weights.check_length(self.nvoices)?;
        self.parameter[stream_index] = weights;
        Ok(())
    }
    /// Set GV weight.
    ///
    /// Conditions:
    /// - stream_index < nstream
    /// - weights.len() == nvoices
    /// - weights.iter().sum() == 1.0
    pub fn set_gv(&mut self, stream_index: usize, weight: &[f64]) -> Result<(), WeightError> {
        let weights = Weights::new(weight)?;
        weights.check_length(self.nvoices)?;
        self.gv[stream_index] = weights;
        Ok(())
    }

    /// Get duration weight.
    pub fn get_duration(&self) -> &Weights {
        &self.duration
    }
    /// Get parameter weight.
    pub fn get_parameter(&self, stream_index: usize) -> &Weights {
        &self.parameter[stream_index]
    }
    /// Get GV weight.
    pub fn get_gv(&self, stream_index: usize) -> &Weights {
        &self.gv[stream_index]
    }
}

/// Individual weight vector.
///
/// Each element corresponds to a voice.
#[derive(Debug, Clone)]
pub struct Weights {
    weights: Vec<f64>,
}

impl Weights {
    /// Create a new [`Weights`] with provided weights.
    pub fn new(weight: &[f64]) -> Result<Self, WeightError> {
        let sum: f64 = weight.iter().sum();
        if approx::abs_diff_ne!(sum, 1.0) {
            return Err(WeightError::InvalidSum);
        }
        Ok(Self {
            weights: weight.to_vec(),
        })
    }

    fn average(nvoices: usize) -> Self {
        let average_weight = 1.0f64 / nvoices as f64;
        Self {
            weights: vec![average_weight; nvoices],
        }
    }

    fn check_length(&self, length: usize) -> Result<(), WeightError> {
        if self.weights.len() != length {
            return Err(WeightError::InvalidLength(length, self.weights.len()));
        }
        Ok(())
    }
}

impl Deref for Weights {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.weights
    }
}
