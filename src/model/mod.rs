use self::model::{Model, ModelParameter, Pattern, StreamModels};

mod model;
mod parser;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModelErrorKind {
    TreeNode,
    TreeIndex,
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
    manifest: GlobalModelManifest,
    voices: Vec<Voice>,
}

    // hts_voice_version: String,
    // sampling_frequency: usize,
    // frame_period: usize,
    // num_voices: usize,
    // num_states: usize,
    // num_streams: usize,
    // stream_type: Vec<String>,
    // fullcontext_format: String,
    // fullcontext_version: String,
    // gv_off_context: Vec<String>,

#[derive(Debug, Clone, Default)]
pub struct GlobalModelManifest {
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

// /// HTS_ModelSet_get_sampling_frequency: get sampling frequency of HTS voices
// size_t HTS_ModelSet_get_sampling_frequency(HTS_ModelSet * ms)

// /// HTS_ModelSet_get_fperiod: get frame period of HTS voices
// size_t HTS_ModelSet_get_fperiod(HTS_ModelSet * ms)

// /// HTS_ModelSet_get_fperiod: get stream option
// const char *HTS_ModelSet_get_option(HTS_ModelSet * ms, size_t stream_index)

// /// HTS_ModelSet_get_gv_flag: get GV flag
// HTS_Boolean HTS_ModelSet_get_gv_flag(HTS_ModelSet * ms, const char *string)

// /// HTS_ModelSet_get_nstate: get number of state
// size_t HTS_ModelSet_get_nstate(HTS_ModelSet * ms)

// const char *HTS_ModelSet_get_fullcontext_label_format(HTS_ModelSet * ms)
// const char *HTS_ModelSet_get_fullcontext_label_version(HTS_ModelSet * ms)
// /// HTS_ModelSet_get_nstream: get number of stream
// size_t HTS_ModelSet_get_nstream(HTS_ModelSet * ms)

// /// HTS_ModelSet_get_nvoices: get number of stream
// size_t HTS_ModelSet_get_nvoices(HTS_ModelSet * ms)

// /// HTS_ModelSet_get_vector_length: get vector length
// size_t HTS_ModelSet_get_vector_length(HTS_ModelSet * ms, size_t stream_index)

// /// HTS_ModelSet_is_msd: get MSD flag
// HTS_Boolean HTS_ModelSet_is_msd(HTS_ModelSet * ms, size_t stream_index)

// /// HTS_ModelSet_get_window_size: get dynamic window size
// size_t HTS_ModelSet_get_window_size(HTS_ModelSet * ms, size_t stream_index)

// /// HTS_ModelSet_get_window_left_width: get left width of dynamic window
// int HTS_ModelSet_get_window_left_width(HTS_ModelSet * ms, size_t stream_index, size_t window_index)

// /// HTS_ModelSet_get_window_right_width: get right width of dynamic window
// int HTS_ModelSet_get_window_right_width(HTS_ModelSet * ms, size_t stream_index, size_t window_index)

// /// HTS_ModelSet_get_window_coefficient: get coefficient of dynamic window
// double HTS_ModelSet_get_window_coefficient(HTS_ModelSet * ms, size_t stream_index, size_t window_index, size_t coefficient_index)

// /// HTS_ModelSet_get_window_max_width: get max width of dynamic window
// size_t HTS_ModelSet_get_window_max_width(HTS_ModelSet * ms, size_t stream_index)

// /// HTS_ModelSet_use_gv: get GV flag
// HTS_Boolean HTS_ModelSet_use_gv(HTS_ModelSet * ms, size_t stream_index)

// /// HTS_ModelSet_get_duration_index: get duration PDF & tree index
// void HTS_ModelSet_get_duration_index(HTS_ModelSet * ms, size_t voice_index, const char *string, size_t * tree_index, size_t * pdf_index)

// /// HTS_ModelSet_get_duration: get duration using interpolation weight
// void HTS_ModelSet_get_duration(HTS_ModelSet * ms, const char *string, const double *iw, double *mean, double *vari)

// /// HTS_ModelSet_get_parameter_index: get paramter PDF & tree index
// void HTS_ModelSet_get_parameter_index(HTS_ModelSet * ms, size_t voice_index, size_t stream_index, size_t state_index, const char *string, size_t * tree_index, size_t * pdf_index)

// /// HTS_ModelSet_get_parameter: get parameter using interpolation weight
// void HTS_ModelSet_get_parameter(HTS_ModelSet * ms, size_t stream_index, size_t state_index, const char *string, const double *const *iw, double *mean, double *vari, double *msd)

// /// HTS_ModelSet_get_gv_index: get gv PDF & tree index
// void HTS_ModelSet_get_gv_index(HTS_ModelSet * ms, size_t voice_index, size_t stream_index, const char *string, size_t * tree_index, size_t * pdf_index)

// /// HTS_ModelSet_get_gv: get GV using interpolation weight
// void HTS_ModelSet_get_gv(HTS_ModelSet * ms, size_t stream_index, const char *string, const double *const *iw, double *mean, double *vari)
