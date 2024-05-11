//! Set of voice model that are morphable.

use std::{ops::Deref, sync::Arc};

use super::{
    interporation_weight::Weights, voice::model::ModelParameter, GlobalModelMetadata, ModelError,
    StreamModelMetadata, Voice, Windows,
};

/// Set of voice model that can be morphed with each other.
///
/// Assumptions:
/// - Has at least one element
/// - Consistent with metadata
/// - Has identical stream metadata
#[derive(Debug, Clone)]
pub struct VoiceSet(Vec<Arc<Voice>>);
impl VoiceSet {
    /// Create a new [`VoiceSet`] from vector of [`Voice`]'s.
    ///
    /// If the "Assumptions" of [`VoiceSet`] is not met, this function will return an error.
    pub fn new(voices: Vec<Arc<Voice>>) -> Result<Self, ModelError> {
        let first = voices.first().ok_or(ModelError::EmptyVoice)?;
        for voice in &voices[1..] {
            if voice.metadata != first.metadata {
                return Err(ModelError::MetadataError);
            }
            if voice.stream_models.len() != first.stream_models.len() {
                return Err(ModelError::MetadataError);
            }
            if !voice
                .stream_models
                .iter()
                .zip(&first.stream_models)
                .all(|(a, b)| a.metadata == b.metadata)
            {
                return Err(ModelError::MetadataError);
            }
        }

        Ok(Self(voices))
    }

    /// Get the first voice in this [`VoiceSet`].
    #[inline]
    fn first(&self) -> &Voice {
        // ensured to have at least one element
        self.0.first().unwrap()
    }
    /// Get the number of voices in this [`VoiceSet`].
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    /// Get whether the [`VoiceSet`] is empty.
    /// 
    /// This function always return `true`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get global metadata.
    #[inline]
    pub fn global_metadata(&self) -> &GlobalModelMetadata {
        &self.first().metadata
    }
    /// Get stream metadata for the specified stream.
    #[inline]
    pub fn stream_metadata(&self, stream_index: usize) -> &StreamModelMetadata {
        &self.first().stream_models[stream_index].metadata
    }
    /// Get the windows for the specified stream.
    #[inline]
    pub fn stream_windows(&self, stream_index: usize) -> &Windows {
        &self.first().stream_models[stream_index].windows
    }

    /// Call the given function with each [`Voice`], apply weights to its result, and return the weighted parameter.
    pub fn weighted<F: Fn(&Arc<Voice>) -> &ModelParameter>(
        &self,
        weights: &Weights,
        param: F,
    ) -> ModelParameter {
        let mut params_iter = self.iter().map(param);
        let mut weights_iter = weights.iter();
        let first_voice = params_iter.next().unwrap();
        let first_weight = weights_iter.next().unwrap();

        let mut result = first_voice.mul(*first_weight);
        for (param, weight) in params_iter.zip(weights_iter) {
            result.mul_add_assign(*weight, param);
        }
        result
    }
}

impl Deref for VoiceSet {
    type Target = [Arc<Voice>];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
