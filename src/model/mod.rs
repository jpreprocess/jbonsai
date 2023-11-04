use std::path::Path;

use self::model::{Model, ModelParameter, Pattern, StreamModels};

mod model;

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

impl ModelSet {
    #[cfg(feature = "htsvoice")]
    pub fn load_htsvoice_files<P: AsRef<Path>>(paths: &[P]) -> Result<Self, ModelError> {
        let mut metadata = None;
        let mut voices = Vec::with_capacity(paths.len());
        for p in paths.as_ref() {
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
    pub fn get_sampling_fperiod(&self) -> usize {
        self.metadata.sampling_frequency
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
        } else if self
            .metadata
            .gv_off_context
            .iter()
            .any(|p| p.is_match(string))
        {
            false
        } else {
            true
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
        self.metadata.num_voices
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
    pub fn get_window_left_width(&self, stream_index: usize, window_index: usize) -> i32 {
        // TODO: check implementation
        let fsize = self.voices.last().unwrap().stream_models[stream_index].windows[window_index]
            .len() as i32;
        -fsize / 2
    }
    /// Get right width of dynamic window
    pub fn get_window_right_width(&self, stream_index: usize, window_index: usize) -> i32 {
        // TODO: check implementation
        let fsize = self.voices.last().unwrap().stream_models[stream_index].windows[window_index]
            .len() as i32;
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
        coefficient_index: usize,
    ) -> f32 {
        // TODO: check implementation
        let row = &self.voices.last().unwrap().stream_models[stream_index].windows[window_index];
        row[row.len() / 2 + coefficient_index]
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
    pub fn get_duration(&self, string: &str, iw: &[f32]) -> ModelParameter {
        self.voices
            .iter()
            .enumerate()
            .fold(None::<ModelParameter>, |mut acc, (i, curr)| {
                let params = curr.duration_model.get_parameter(2, string);
                if let Some(ref mut acc) = acc {
                    acc.add_assign(iw[i], params);
                } else {
                    acc = Some(params.clone());
                }
                acc
            })
            .unwrap()
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
        iw: &[f32],
    ) -> ModelParameter {
        self.voices
            .iter()
            .enumerate()
            .fold(None::<ModelParameter>, |mut acc, (i, curr)| {
                let params = curr.stream_models[stream_index]
                    .stream_model
                    .get_parameter(state_index, string);
                if let Some(ref mut acc) = acc {
                    acc.add_assign(iw[i], params);
                } else {
                    acc = Some(params.clone());
                }
                acc
            })
            .unwrap()
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
    pub fn get_gv(&self, stream_index: usize, string: &str, iw: &[f32]) -> ModelParameter {
        self.voices
            .iter()
            .enumerate()
            .fold(None::<ModelParameter>, |mut acc, (i, curr)| {
                let params = curr.stream_models[stream_index]
                    .gv_model
                    .as_ref()
                    .unwrap()
                    .get_parameter(2, string);
                if let Some(ref mut acc) = acc {
                    acc.add_assign(iw[i], params);
                } else {
                    acc = Some(params.clone());
                }
                acc
            })
            .unwrap()
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

pub struct Voice {
    pub duration_model: Model,
    pub stream_models: Vec<StreamModels>,
}
