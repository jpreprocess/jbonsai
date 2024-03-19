use std::{fmt::Display, path::Path, sync::Arc};

use self::{
    interporation_weight::InterporationWeight,
    stream::{Model, ModelParameter, StreamModels},
    window::Windows,
};
use jlabel::Label;

pub mod interporation_weight;
pub mod question;
pub mod stream;
pub mod window;

#[cfg(feature = "htsvoice")]
mod parser;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("No HTS voice was given.")]
    EmptyVoice,
    #[error("The global metadata does not match.")]
    MetadataError,
    #[error("Io failed: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "htsvoice")]
    #[error("Parser returned error:{0}")]
    ParserError(#[from] parser::ModelParseError),
}

pub type StreamParameter = Vec<(Vec<(f64, f64)>, f64)>;
pub type GvParameter = (Vec<(f64, f64)>, Vec<bool>);

pub struct Models<'a> {
    labels: Vec<Label>,

    /// Assumptions:
    /// - Has at least one element
    /// - Consistent with metadata
    /// - Has identical stream metadata
    voices: &'a VoiceSet,
    weights: &'a InterporationWeight,
}

impl<'a> Models<'a> {
    pub fn new(labels: Vec<Label>, voices: &'a VoiceSet, weights: &'a InterporationWeight) -> Self {
        Self {
            labels,
            voices,
            weights,
        }
    }

    pub fn nstream(&self) -> usize {
        self.voices.first().metadata.num_streams
    }
    pub fn nstate(&self) -> usize {
        self.voices.first().metadata.num_states
    }
    pub fn vector_length(&self, stream_index: usize) -> usize {
        let metadata = &self.voices.first().stream_models[stream_index].metadata;
        metadata.vector_length
    }

    pub fn duration(&self) -> Vec<(f64, f64)> {
        let metadata = &self.voices.first().metadata;
        let weight = self.weights.get_duration().get_weights();
        self.labels
            .iter()
            .flat_map(|label| {
                let mut params = ModelParameter::new(metadata.num_states, false);
                for (voice, weight) in self.voices.iter().zip(weight) {
                    let curr_params = voice.duration_model.get_parameter(2, label);
                    params.add_assign(*weight, curr_params);
                }
                params.parameters
            })
            .collect()
    }
    /// FIXME: label/state -> window -> vector
    pub fn stream(&self, stream_index: usize) -> StreamParameter {
        let global_metadata = &self.voices.first().metadata;
        let stream_metadata = &self.voices.first().stream_models[stream_index].metadata;
        let weight = self.weights.get_parameter(stream_index).get_weights();
        self.labels
            .iter()
            .flat_map(|label| {
                (2..2 + global_metadata.num_states).map(|state_index| {
                    let mut params = ModelParameter::new(
                        stream_metadata.vector_length * stream_metadata.num_windows,
                        stream_metadata.is_msd,
                    );
                    for (voice, weight) in self.voices.iter().zip(weight) {
                        let curr_params = voice.stream_models[stream_index]
                            .stream_model
                            .get_parameter(state_index, label);
                        params.add_assign(*weight, curr_params);
                    }
                    let ModelParameter { parameters, msd } = params;
                    // FIXME: Split parameter
                    (parameters, msd.unwrap_or(f64::MAX))
                })
            })
            .collect()
    }
    pub fn gv(&self, stream_index: usize) -> Option<GvParameter> {
        let global_metadata = &self.voices.first().metadata;
        let stream_metadata = &self.voices.first().stream_models[stream_index].metadata;
        if !stream_metadata.use_gv {
            return None;
        }

        let weight = self.weights.get_gv(stream_index).get_weights();

        let mut params = ModelParameter::new(stream_metadata.vector_length, false);
        for (voice, weight) in self.voices.iter().zip(weight) {
            let curr_params = voice.stream_models[stream_index]
                .gv_model
                .as_ref()
                .unwrap()
                .get_parameter(2, self.labels.first()?);
            params.add_assign(*weight, curr_params);
        }

        let gv_switch = self
            .labels
            .iter()
            .flat_map(|label| {
                let switch = !global_metadata.gv_off_context.test(label);
                [switch].repeat(global_metadata.num_states)
            })
            .collect();

        Some((params.parameters, gv_switch))
    }
    pub fn windows(&self, stream_index: usize) -> &Windows {
        &self.voices.first().stream_models[stream_index].windows
    }
}

pub fn apply_additional_half_tone(params: &mut StreamParameter, additional_half_tone: f64) {
    use crate::constants::{HALF_TONE, MAX_LF0, MIN_LF0};
    if additional_half_tone == 0.0 {
        return;
    }
    params.iter_mut().for_each(|(p, _)| {
        let f = &mut p[0].0;
        *f += additional_half_tone * HALF_TONE;
        *f = f.max(MIN_LF0).min(MAX_LF0);
    });
}

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

    pub fn first(&self) -> &Voice {
        // ensured to have at least one element
        self.0.first().unwrap()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<Voice>> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalModelMetadata {
    pub hts_voice_version: String,
    pub sampling_frequency: usize,
    pub frame_period: usize,
    pub num_states: usize,
    pub num_streams: usize,
    pub stream_type: Vec<String>,
    pub fullcontext_format: String,
    pub fullcontext_version: String,
    pub gv_off_context: question::Question,
}

impl Display for GlobalModelMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HTS Voice Version: {}", self.hts_voice_version)?;
        writeln!(f, "Sampling Frequency: {}", self.sampling_frequency)?;
        writeln!(f, "Frame Period: {}", self.frame_period)?;
        writeln!(f, "Number of States: {}", self.num_states)?;
        writeln!(f, "Number of Streams: {}", self.num_streams)?;
        writeln!(f, "Streams: {}", self.stream_type.join(", "))?;
        writeln!(
            f,
            "Fullcontext: {}@{}",
            self.fullcontext_format, self.fullcontext_version
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Voice {
    pub metadata: GlobalModelMetadata,
    pub duration_model: Model,
    pub stream_models: Vec<StreamModels>,
}

impl Voice {
    #[cfg(feature = "htsvoice")]
    pub fn load_htsvoice_file<P: AsRef<Path>>(path: &P) -> Result<Self, ModelError> {
        let f = std::fs::read(path)?;
        Ok(parser::parse_htsvoice(&f)?)
    }
}

impl Display for Voice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duration Model: {}", self.duration_model)?;
        writeln!(f, "Stream Models:")?;
        for (i, model) in self.stream_models.iter().enumerate() {
            write!(f, "#{}:\n{}", i, model)?;
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "htsvoice"))]
mod tests {
    use std::sync::Arc;

    use crate::{
        model::{
            interporation_weight::{InterporationWeight, Weights},
            window::Window,
        },
        tests::{
            MODEL_NITECH_ATR503, MODEL_TOHOKU_F01_HAPPY, MODEL_TOHOKU_F01_NORMAL, SAMPLE_SENTENCE_1,
        },
    };

    use super::{Models, Voice, VoiceSet};

    fn load_voice() -> Voice {
        Voice::load_htsvoice_file(&MODEL_NITECH_ATR503).unwrap()
    }

    #[test]
    fn metadata() {
        let voice = load_voice();
        assert_eq!(voice.metadata.sampling_frequency, 48000);
        assert_eq!(voice.metadata.frame_period, 240);
        assert_eq!(voice.metadata.num_states, 5);
    }

    #[test]
    fn tree_index() {
        let voice = load_voice();
        let label = SAMPLE_SENTENCE_1[2].parse().unwrap();

        assert_eq!(
            voice.duration_model.get_index(2, &label),
            (Some(2), Some(144))
        );
        assert_eq!(
            voice.stream_models[1].stream_model.get_index(2, &label),
            (Some(2), Some(234))
        );
        assert_eq!(
            voice.stream_models[1]
                .gv_model
                .as_ref()
                .unwrap()
                .get_index(2, &label),
            (Some(2), Some(3))
        );
    }

    #[test]
    fn get_parameters() {
        let voiceset = VoiceSet::new(vec![Arc::new(load_voice())]).unwrap();
        let iw = InterporationWeight::new(1, 3);
        let models = Models::new(vec![SAMPLE_SENTENCE_1[2].parse().unwrap()], &voiceset, &iw);

        assert_eq!(
            models.duration(),
            vec![
                (2.1477856636047363, 2.4373505115509033),
                (3.2821402549743652, 4.192541599273682),
                (2.679042100906372, 3.923785924911499),
                (3.378859281539917, 3.866243362426758),
                (2.7264480590820313, 3.725647211074829)
            ]
        );
        assert_eq!(
            models.stream(1)[0],
            (
                vec![
                    (4.806920528411865, 0.005436264909803867),
                    (0.005690717604011297, 0.00008830774459056556),
                    (-0.00019663637795019895, 0.00024312522145919502),
                ],
                0.949999988079071
            )
        );
        assert_eq!(
            models.gv(1),
            Some((
                vec![(0.03621548041701317, 0.00010934889724012464)],
                vec![true, true, true, true, true]
            ))
        );
    }

    #[test]
    fn window() {
        let voiceset = VoiceSet::new(vec![Arc::new(load_voice())]).unwrap();
        let iw = InterporationWeight::new(1, 3);
        let models = Models::new(vec![], &voiceset, &iw);

        let windows = models.windows(0);

        assert_eq!(windows.size(), 3);
        assert_eq!(windows.max_width(), 1);

        let window = windows.iter().nth(1).unwrap();

        assert_eq!(window.left_width(), 1);
        assert_eq!(window.right_width(), 1);

        assert_eq!(window, &Window::new(vec![-0.5, 0.0, 0.5]));
    }

    #[test]
    fn multiple_models() {
        let normal = Voice::load_htsvoice_file(&MODEL_TOHOKU_F01_NORMAL).unwrap();
        let happy = Voice::load_htsvoice_file(&MODEL_TOHOKU_F01_HAPPY).unwrap();
        let voiceset = VoiceSet::new(vec![Arc::new(normal), Arc::new(happy)]).unwrap();

        let mut iw = InterporationWeight::new(2, 3);
        iw.set_duration(Weights::new(&[0.7, 0.3]).unwrap()).unwrap();
        iw.set_parameter(1, Weights::new(&[0.7, 0.3]).unwrap())
            .unwrap();

        let models = Models::new(vec![SAMPLE_SENTENCE_1[2].parse().unwrap()], &voiceset, &iw);

        assert_eq!(
            models.duration(),
            vec![
                (3.345043873786926, 6.943870377540589),
                (9.866290760040282, 59.23959312438964),
                (5.616884994506836, 16.154539680480955),
                (1.7678393721580503, 0.9487730085849762),
                (1.3566675186157227, 1.2509666562080382)
            ]
        );
        assert_eq!(
            models.stream(1)[0],
            (
                vec![
                    (5.354794883728027, 0.00590993594378233),
                    (-0.004957371624186635, 0.00017984867736231536),
                    (0.010301648452877997, 0.00044686400215141473)
                ],
                0.9955164790153503
            )
        );
    }
}
