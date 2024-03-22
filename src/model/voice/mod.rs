use std::fmt::Display;

use self::{parameter::Model, window::Windows};

pub mod parameter;
pub mod question;
pub mod tree;
pub mod window;

#[derive(Debug, Clone, PartialEq)]
pub struct Voice {
    pub metadata: GlobalModelMetadata,
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
pub struct StreamModels {
    pub metadata: StreamModelMetadata,

    pub stream_model: Model,
    pub gv_model: Option<Model>,
    pub windows: Windows,
}

impl Display for StreamModels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  Model: {}", self.stream_model)?;
        if let Some(ref gv_model) = self.gv_model {
            write!(f, "  GV Model: {}", gv_model)?;
        }
        writeln!(
            f,
            "  Window Width: {}",
            self.windows.iter().fold(String::new(), |acc, curr| {
                if acc.is_empty() {
                    format!("{}", curr.width())
                } else {
                    format!("{}, {}", acc, curr.width())
                }
            })
        )?;
        Ok(())
    }
}

impl StreamModels {
    pub fn new(
        metadata: StreamModelMetadata,
        stream_model: Model,
        gv_model: Option<Model>,
        windows: Windows,
    ) -> Self {
        StreamModels {
            metadata,
            stream_model,
            gv_model,
            windows,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamModelMetadata {
    pub vector_length: usize,
    pub num_windows: usize,
    pub is_msd: bool,
    pub use_gv: bool,
    pub option: Vec<String>,
}
