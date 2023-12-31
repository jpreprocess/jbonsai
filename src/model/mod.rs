use std::{fmt::Display, path::Path};

use self::stream::{Model, ModelParameter, Pattern, StreamModels};

pub mod interporation_weight;
pub mod stream;

#[cfg(feature = "htsvoice")]
mod parser;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModelErrorKind {
    Io,
    NomError,
    MetadataError,
}

impl ModelErrorKind {
    pub fn with_error<E>(self, source: E) -> ModelError
    where
        anyhow::Error: From<E>,
    {
        ModelError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ModelError(kind={kind:?}, source={source})")]
pub struct ModelError {
    pub kind: ModelErrorKind,
    source: anyhow::Error,
}

pub struct ModelSet {
    metadata: GlobalModelMetadata,
    voices: Vec<Voice>,
}

impl Display for ModelSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.metadata)?;
        for (i, voice) in self.voices.iter().enumerate() {
            writeln!(f, "Voice #{}:\n{}", i, voice)?;
        }
        Ok(())
    }
}

impl ModelSet {
    #[cfg(feature = "htsvoice")]
    pub fn load_htsvoice_files<P: AsRef<Path>>(paths: &[P]) -> Result<Self, ModelError> {
        let mut metadata = None;
        let mut voices = Vec::with_capacity(paths.len());
        for p in paths {
            let f = std::fs::read(p).map_err(|err| ModelErrorKind::Io.with_error(err))?;

            let (_, (new_metadata, voice)) =
                parser::parse_htsvoice::<nom::error::VerboseError<&[u8]>>(&f).map_err(|err| {
                    ModelErrorKind::NomError
                        .with_error(anyhow::anyhow!("Parser returned error:\n{}", err))
                })?;

            if let Some(ref metadata) = metadata {
                if *metadata != new_metadata {
                    return Err(ModelErrorKind::MetadataError
                        .with_error(anyhow::anyhow!("The global metadata does not match.")));
                }
            } else {
                metadata = Some(new_metadata);
            }
            voices.push(voice);
        }
        Ok(Self {
            metadata: metadata.unwrap(),
            voices,
        })
    }

    /// Get sampling frequency of HTS voices
    pub fn get_sampling_frequency(&self) -> usize {
        self.metadata.sampling_frequency
    }
    /// Get frame period of HTS voices
    pub fn get_fperiod(&self) -> usize {
        self.metadata.frame_period
    }
    /// Get stream option
    pub fn get_option(&self, stream_index: usize) -> &[String] {
        // TODO: option
        &self.voices[0].stream_models[stream_index].metadata.option
    }
    /// Get GV flag
    pub fn get_gv_flag(&self, string: &str) -> bool {
        if self.metadata.gv_off_context.is_empty() {
            true
        } else {
            !self
                .metadata
                .gv_off_context
                .iter()
                .any(|p| p.is_match(string))
        }
    }
    /// Get number of state
    pub fn get_nstate(&self) -> usize {
        self.metadata.num_states
    }
    /// Get full-context label format
    pub fn get_fullcontext_label_format(&self) -> &str {
        &self.metadata.fullcontext_format
    }
    /// Get full-context label version
    pub fn get_fullcontext_label_version(&self) -> &str {
        &self.metadata.fullcontext_version
    }
    /// Get number of stream
    pub fn get_nstream(&self) -> usize {
        self.metadata.num_streams
    }
    /// Get number of voices
    pub fn get_nvoices(&self) -> usize {
        // self.metadata.num_voices
        self.voices.len()
    }

    /// Get vector length
    pub fn get_vector_length(&self, stream_index: usize) -> usize {
        self.voices[0].stream_models[stream_index]
            .metadata
            .vector_length
    }
    /// Get MSD flag
    pub fn is_msd(&self, stream_index: usize) -> bool {
        self.voices[0].stream_models[stream_index].metadata.is_msd
    }

    /// Get dynamic window size
    pub fn get_window_size(&self, stream_index: usize) -> usize {
        // TODO: check implementation
        self.voices.last().unwrap().stream_models[stream_index]
            .windows
            .len()
    }
    /// Get left width of dynamic window
    pub fn get_window_left_width(&self, stream_index: usize, window_index: usize) -> isize {
        // TODO: check implementation
        let fsize = self.voices.last().unwrap().stream_models[stream_index].windows[window_index]
            .len() as isize;
        -fsize / 2
    }
    /// Get right width of dynamic window
    pub fn get_window_right_width(&self, stream_index: usize, window_index: usize) -> isize {
        // TODO: check implementation
        let fsize = self.voices.last().unwrap().stream_models[stream_index].windows[window_index]
            .len() as isize;
        if fsize % 2 == 0 {
            fsize / 2 - 1
        } else {
            fsize / 2
        }
    }
    /// Get coefficient of dynamic window
    pub fn get_window_coefficient(
        &self,
        stream_index: usize,
        window_index: usize,
        coefficient_index: isize,
    ) -> f64 {
        // TODO: check implementation
        let row = &self.voices.last().unwrap().stream_models[stream_index].windows[window_index];
        row[((row.len() / 2) as isize + coefficient_index) as usize]
    }
    /// Get max width of dynamic window
    pub fn get_window_max_width(&self, stream_index: usize) -> usize {
        // TODO: check implementation; important
        let max_width = self.voices.last().unwrap().stream_models[stream_index]
            .windows
            .iter()
            .map(Vec::len)
            .max()
            .unwrap();
        max_width / 2
    }

    /// Get GV flag
    pub fn use_gv(&self, stream_index: usize) -> bool {
        // TODO: check implementation
        self.voices[0].stream_models[stream_index]
            .gv_model
            .is_some()
    }

    /// Get duration PDF & tree index
    pub fn get_duration_index(
        &self,
        voice_index: usize,
        string: &str,
    ) -> (Option<usize>, Option<usize>) {
        self.voices[voice_index].duration_model.get_index(2, string)
    }
    /// Get duration using interpolation weight
    pub fn get_duration(&self, string: &str, iw: &[f64]) -> ModelParameter {
        let mut params = ModelParameter::new(self.get_nstate(), false);
        for (voice, weight) in self.voices.iter().zip(iw) {
            let curr_params = voice.duration_model.get_parameter(2, string);
            params.add_assign(*weight, curr_params);
        }
        params
    }
    /// Get paramter PDF & tree index
    pub fn get_parameter_index(
        &self,
        voice_index: usize,
        stream_index: usize,
        state_index: usize,
        string: &str,
    ) -> (Option<usize>, Option<usize>) {
        self.voices[voice_index].stream_models[stream_index]
            .stream_model
            .get_index(state_index, string)
    }
    /// Get parameter using interpolation weight
    pub fn get_parameter(
        &self,
        stream_index: usize,
        state_index: usize,
        string: &str,
        iw: &[f64],
    ) -> ModelParameter {
        let mut params = ModelParameter::new(
            self.get_vector_length(stream_index) * self.get_window_size(stream_index),
            self.is_msd(stream_index),
        );
        for (voice, weight) in self.voices.iter().zip(iw) {
            let curr_params = voice.stream_models[stream_index]
                .stream_model
                .get_parameter(state_index, string);
            params.add_assign(*weight, curr_params);
        }
        params
    }
    /// Get gv PDF & tree index
    pub fn get_gv_index(
        &self,
        voice_index: usize,
        stream_index: usize,
        string: &str,
    ) -> (Option<usize>, Option<usize>) {
        self.voices[voice_index].stream_models[stream_index]
            .gv_model
            .as_ref()
            .unwrap()
            .get_index(2, string)
    }
    /// Get GV using interpolation weight
    pub fn get_gv(&self, stream_index: usize, string: &str, iw: &[f64]) -> ModelParameter {
        let mut params = ModelParameter::new(self.get_vector_length(stream_index), false);
        for (voice, weight) in self.voices.iter().zip(iw) {
            let curr_params = voice.stream_models[stream_index]
                .gv_model
                .as_ref()
                .unwrap()
                .get_parameter(2, string);
            params.add_assign(*weight, curr_params);
        }
        params
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobalModelMetadata {
    pub hts_voice_version: String,
    pub sampling_frequency: usize,
    pub frame_period: usize,
    pub num_voices: usize,
    pub num_states: usize,
    pub num_streams: usize,
    pub stream_type: Vec<String>,
    pub fullcontext_format: String,
    pub fullcontext_version: String,
    pub gv_off_context: Vec<Pattern>,
}

impl Display for GlobalModelMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HTS Voice Version: {}", self.hts_voice_version)?;
        writeln!(f, "Sampling Frequency: {}", self.sampling_frequency)?;
        writeln!(f, "Frame Period: {}", self.frame_period)?;
        writeln!(f, "Number of Voices: {}", self.num_voices)?;
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

pub struct Voice {
    pub duration_model: Model,
    pub stream_models: Vec<StreamModels>,
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
    use crate::{
        model::{stream::ModelParameter, ModelSet},
        tests::{
            MODEL_NITECH_ATR503, MODEL_TOHOKU_F01_HAPPY, MODEL_TOHOKU_F01_NORMAL, SAMPLE_SENTENCE_1,
        },
    };

    fn load_models() -> ModelSet {
        ModelSet::load_htsvoice_files(&[MODEL_NITECH_ATR503]).unwrap()
    }

    #[test]
    fn get_metadata() {
        let jsyn = load_models();
        assert_eq!(jsyn.get_sampling_frequency(), 48000);
        assert_eq!(jsyn.get_fperiod(), 240);
        assert_eq!(jsyn.get_nstate(), 5);
    }

    #[test]
    fn get_duration() {
        let jsyn = load_models();

        let (jsyn_tree_index, jsyn_pdf_index) = jsyn.get_duration_index(0, SAMPLE_SENTENCE_1[2]);
        assert_eq!(jsyn_tree_index.unwrap(), 2);
        assert_eq!(jsyn_pdf_index.unwrap(), 144);

        let jsyn_param = jsyn.get_duration(SAMPLE_SENTENCE_1[2], &[1.0]);
        assert_eq!(
            jsyn_param,
            ModelParameter {
                parameters: vec![
                    (2.1477856636047363, 2.4373505115509033),
                    (3.2821402549743652, 4.192541599273682),
                    (2.679042100906372, 3.923785924911499),
                    (3.378859281539917, 3.866243362426758),
                    (2.7264480590820313, 3.725647211074829)
                ],
                msd: None
            }
        );
    }

    #[test]
    fn get_parameter() {
        let jsyn = load_models();

        let (jsyn_tree_index, jsyn_pdf_index) =
            jsyn.get_parameter_index(0, 1, 2, SAMPLE_SENTENCE_1[2]);
        assert_eq!(jsyn_tree_index, Some(2));
        assert_eq!(jsyn_pdf_index, Some(234));

        let jsyn_param = jsyn.get_parameter(1, 2, SAMPLE_SENTENCE_1[2], &[1.0, 1.0]);
        assert_eq!(
            jsyn_param,
            ModelParameter {
                parameters: vec![
                    (4.806920528411865, 0.005436264909803867),
                    (0.005690717604011297, 8.830774459056556e-5),
                    (-0.00019663637795019895, 0.00024312522145919502)
                ],
                msd: Some(0.949999988079071),
            }
        );
    }

    #[test]
    fn get_gv() {
        let jsyn = load_models();

        assert!(jsyn.use_gv(1));

        let (jsyn_tree_index, jsyn_pdf_index) = jsyn.get_gv_index(0, 1, SAMPLE_SENTENCE_1[2]);
        assert_eq!(jsyn_tree_index, Some(2));
        assert_eq!(jsyn_pdf_index, Some(3));

        let jsyn_param = jsyn.get_gv(1, SAMPLE_SENTENCE_1[2], &[1.0, 1.0]);
        assert_eq!(
            jsyn_param,
            ModelParameter {
                parameters: vec![(0.03621548041701317, 0.00010934889724012464)],
                msd: None,
            }
        );
    }

    #[test]
    fn window() {
        let jsyn = load_models();
        assert_eq!(jsyn.get_window_size(0), 3);
        assert_eq!(jsyn.get_window_left_width(0, 1), -1);
        assert_eq!(jsyn.get_window_right_width(0, 1), 1);
        assert_eq!(jsyn.get_window_coefficient(0, 1, 0), 0.0);
        assert_eq!(jsyn.get_window_coefficient(0, 1, 1), 0.5);
        assert_eq!(jsyn.get_window_max_width(0), 1);
    }

    #[test]
    fn multiple_models() {
        let modelset =
            ModelSet::load_htsvoice_files(&[MODEL_TOHOKU_F01_NORMAL, MODEL_TOHOKU_F01_HAPPY])
                .unwrap();
        assert_eq!(
            modelset.get_duration(SAMPLE_SENTENCE_1[2], &[0.7, 0.3]),
            ModelParameter {
                parameters: vec![
                    (3.345043873786926, 6.943870377540589),
                    (9.866290760040282, 59.23959312438964),
                    (5.616884994506836, 16.154539680480955),
                    (1.7678393721580503, 0.9487730085849762),
                    (1.3566675186157227, 1.2509666562080382)
                ],
                msd: None
            }
        );
        assert_eq!(
            modelset.get_parameter(1, 2, SAMPLE_SENTENCE_1[2], &[0.7, 0.3]),
            ModelParameter {
                parameters: vec![
                    (5.354794883728027, 0.00590993594378233),
                    (-0.004957371624186635, 0.00017984867736231536),
                    (0.010301648452877997, 0.00044686400215141473)
                ],
                msd: Some(0.9955164790153503)
            }
        );
    }

    #[test]
    fn display() {
        let jsyn = load_models();
        println!("{}", jsyn);
    }
}
