//! Parameter in a stream (LF0, MCP, or LPF).

use std::ops::Deref;

use super::MeanVari;

/// Vector of stream and MSD (multi-space probability distribution) parameters.
///
/// The outer Vec's length is `label_length * nstate`, and the inner Vec's length is `window_length * vector_length`.
#[derive(Debug, Clone)]
pub struct StreamParameter(Vec<(Vec<MeanVari>, f64)>);

impl Deref for StreamParameter {
    type Target = [(Vec<MeanVari>, f64)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl StreamParameter {
    /// Create a new [`StreamParameter`].
    pub fn new(inner: Vec<(Vec<MeanVari>, f64)>) -> Self {
        Self(inner)
    }

    /// Add `additional_half_tone * HALF_TONE` to mean.
    pub fn apply_additional_half_tone(&mut self, additional_half_tone: f64) {
        use crate::constants::{HALF_TONE, MAX_LF0, MIN_LF0};
        if additional_half_tone == 0.0 {
            return;
        }
        for (p, _) in &mut self.0 {
            p[0].0 = (p[0].0 + additional_half_tone * HALF_TONE).clamp(MIN_LF0, MAX_LF0);
        }
    }
}
