use std::{ops::Deref, sync::Arc};

use super::{
    interporation_weight::Weights, voice::model::ModelParameter, GlobalModelMetadata, ModelError,
    StreamModelMetadata, Voice, Windows,
};

/// Assumptions:
/// - Has at least one element
/// - Consistent with metadata
/// - Has identical stream metadata
#[derive(Debug, Clone)]
pub struct VoiceSet(Vec<Arc<Voice>>);
impl VoiceSet {
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

    #[inline]
    fn first(&self) -> &Voice {
        // ensured to have at least one element
        self.0.first().unwrap()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn global_metadata(&self) -> &GlobalModelMetadata {
        &self.first().metadata
    }
    #[inline]
    pub fn stream_metadata(&self, stream_index: usize) -> &StreamModelMetadata {
        &self.first().stream_models[stream_index].metadata
    }
    #[inline]
    pub fn stream_windows(&self, stream_index: usize) -> &Windows {
        &self.first().stream_models[stream_index].windows
    }

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
