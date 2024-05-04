use std::{borrow::Cow, sync::Arc};

use self::voice::model::ModelParameter;

pub use self::{
    interporation_weight::InterporationWeight,
    voice::{window::Windows, GlobalModelMetadata, StreamModelMetadata, Voice},
};

use jlabel::Label;

pub mod interporation_weight;
pub mod voice;

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

    voices: Cow<'a, VoiceSet>,
    weights: Cow<'a, InterporationWeight>,
}

impl<'a> Models<'a> {
    pub fn new(labels: Vec<Label>, voices: &'a VoiceSet, weights: &'a InterporationWeight) -> Self {
        Self {
            labels,
            voices: Cow::Borrowed(voices),
            weights: Cow::Borrowed(weights),
        }
    }

    pub fn nstate(&self) -> usize {
        self.voices.first().metadata.num_states
    }
    pub fn vector_length(&self, stream_index: usize) -> usize {
        let metadata = &self.voices.first().stream_models[stream_index].metadata;
        metadata.vector_length
    }

    pub fn duration(&self) -> Vec<(f64, f64)> {
        let metadata = &self.voices.global_metadata();
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
        let global_metadata = &self.voices.global_metadata();
        let stream_metadata = &self.voices.stream_metadata(stream_index);
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
        let global_metadata = &self.voices.global_metadata();
        let stream_metadata = &self.voices.stream_metadata(stream_index);
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
        *f = f.clamp(MIN_LF0, MAX_LF0);
    });
}

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
    pub fn first(&self) -> &Voice {
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
    pub fn iter(&self) -> impl Iterator<Item = &Arc<Voice>> {
        self.0.iter()
    }
}

#[cfg(feature = "htsvoice")]
pub fn load_htsvoice_file<P: AsRef<std::path::Path>>(path: &P) -> Result<Voice, ModelError> {
    let f = std::fs::read(path)?;
    Ok(parser::parse_htsvoice(&f)?)
}

#[cfg(all(test, feature = "htsvoice"))]
pub mod tests {
    use std::{borrow::Cow, sync::Arc};

    use crate::{
        model::{
            interporation_weight::{InterporationWeight, Weights},
            voice::window::Window,
        },
        tests::{
            MODEL_NITECH_ATR503, MODEL_TOHOKU_F01_HAPPY, MODEL_TOHOKU_F01_NORMAL, SAMPLE_SENTENCE_1,
        },
    };

    use super::{load_htsvoice_file, Models, Voice, VoiceSet};

    fn load_voice() -> Voice {
        load_htsvoice_file(&MODEL_NITECH_ATR503).unwrap()
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

    pub fn load_models() -> Models<'static> {
        let labels = SAMPLE_SENTENCE_1
            .iter()
            .map(|line| line.parse().unwrap())
            .collect();

        let voiceset = VoiceSet::new(vec![Arc::new(load_voice())]).unwrap();
        let iw = InterporationWeight::new(1, 3);

        Models {
            labels,
            voices: Cow::Owned(voiceset),
            weights: Cow::Owned(iw),
        }
    }

    #[test]
    fn get_parameters() {
        let models = load_models();

        let duration = models.duration();
        assert_eq!(duration.len(), 40);
        assert_eq!(
            duration[..15],
            [
                (7.939206123352051, 145.76211547851563),
                (16.867250442504883, 353.91778564453125),
                (13.902158737182617, 178.05068969726563),
                (24.711565017700195, 395.954833984375),
                (15.016390800476074, 62.81060791015625),
                (2.9893455505371094, 3.7195587158203125),
                (3.650455951690674, 7.21462869644165),
                (2.317136287689209, 2.8865654468536377),
                (2.3675591945648193, 2.918273448944092),
                (2.4925434589385986, 2.9260120391845703),
                (2.1477856636047363, 2.4373505115509033),
                (3.2821402549743652, 4.192541599273682),
                (2.679042100906372, 3.923785924911499),
                (3.378859281539917, 3.866243362426758),
                (2.7264480590820313, 3.725647211074829),
            ]
        );

        let stream = models.stream(1);
        assert_eq!(stream.len(), 40);
        assert_eq!(
            stream[..15],
            [
                (
                    vec![
                        (4.708907127380371, 0.027746843174099922),
                        (0.010573429986834526, 0.0006717125070281327),
                        (-0.019542237743735313, 0.002855533268302679)
                    ],
                    0.05000000074505806
                ),
                (
                    vec![
                        (4.714630603790283, 0.03322882577776909),
                        (-0.009544742293655872, 0.000757755886297673),
                        (0.011145883239805698, 0.0031274918001145124)
                    ],
                    0.05000000074505806
                ),
                (
                    vec![
                        (4.704207420349121, 0.040450580418109894),
                        (0.004150974098592997, 0.0008980912389233708),
                        (0.010611549019813538, 0.0024848130997270346)
                    ],
                    0.05000000074505806
                ),
                (
                    vec![(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)],
                    0.05000000074505806
                ),
                (
                    vec![
                        (4.768340110778809, 0.01530302595347166),
                        (0.02272343635559082, 3.5269540603621863e-6),
                        (-0.047215938568115234, 1.3166980352252722e-5)
                    ],
                    0.05000000074505806
                ),
                (
                    vec![
                        (4.747085094451904, 0.009076375514268875),
                        (-0.010534754022955894, 0.002568872645497322),
                        (-0.016766104847192764, 0.014940978959202766)
                    ],
                    0.23628035187721252
                ),
                (
                    vec![
                        (4.736148357391357, 0.009678148664534092),
                        (0.00046353874495252967, 0.002193617168813944),
                        (-0.01878436654806137, 0.013272966258227825)
                    ],
                    0.3182770907878876
                ),
                (
                    vec![
                        (4.739607334136963, 0.0061369095928967),
                        (0.014216499403119087, 0.001773378811776638),
                        (0.014568353071808815, 0.008928200230002403)
                    ],
                    0.24298794567584991
                ),
                (
                    vec![
                        (4.785215377807617, 0.0035884405951946974),
                        (-0.0017961699049919844, 0.0011838842183351517),
                        (-0.03521687909960747, 0.009459378197789192)
                    ],
                    0.47957301139831543
                ),
                (
                    vec![
                        (4.727545261383057, 0.006344881374388933),
                        (-0.0061436910182237625, 0.0008336332393810153),
                        (0.012339762412011623, 0.0043235644698143005)
                    ],
                    0.9500000476837158
                ),
                (
                    vec![
                        (4.806920528411865, 0.005436264909803867),
                        (0.005690717604011297, 8.830774459056556e-5),
                        (-0.00019663637795019895, 0.00024312522145919502)
                    ],
                    0.949999988079071
                ),
                (
                    vec![
                        (4.726495742797852, 0.009544309228658676),
                        (0.004016753751784563, 6.134989234851673e-5),
                        (0.0006506261415779591, 0.00020928174490109086)
                    ],
                    0.949999988079071
                ),
                (
                    vec![
                        (4.89390230178833, 0.0047211721539497375),
                        (0.010379847139120102, 2.7608957680058666e-5),
                        (0.00029396452009677887, 8.474134665448219e-5)
                    ],
                    0.949999988079071
                ),
                (
                    vec![
                        (4.889120578765869, 0.002151205437257886),
                        (0.0037524907384067774, 3.744014975382015e-5),
                        (-0.0010508624836802483, 7.232622738229111e-5)
                    ],
                    0.949999988079071
                ),
                (
                    vec![
                        (4.946272373199463, 0.008521423675119877),
                        (0.001904668752104044, 5.143996168044396e-5),
                        (-0.0012227826518937945, 7.035945600364357e-5)
                    ],
                    0.949999988079071
                )
            ]
        );

        assert_eq!(
            models.gv(1),
            Some((
                vec![(0.03621548041701317, 0.00010934889724012464)],
                vec![
                    false, false, false, false, false, true, true, true, true, true, true, true,
                    true, true, true, true, true, true, true, true, true, true, true, true, true,
                    true, true, true, true, true, true, true, true, true, true, false, false,
                    false, false, false
                ]
            ))
        );
    }

    #[test]
    fn window() {
        let models = load_models();

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
        let normal = load_htsvoice_file(&MODEL_TOHOKU_F01_NORMAL).unwrap();
        let happy = load_htsvoice_file(&MODEL_TOHOKU_F01_HAPPY).unwrap();
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
