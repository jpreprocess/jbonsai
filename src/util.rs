pub type size_t = libc::c_ulong;
pub type HTS_Boolean = libc::c_char;
pub type uint32_t = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Audio {
    pub sampling_frequency: size_t,
    pub max_buff_size: size_t,
    pub buff: *mut libc::c_short,
    pub buff_size: size_t,
    pub audio_interface: *mut libc::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Window {
    pub size: size_t,
    pub l_width: *mut libc::c_int,
    pub r_width: *mut libc::c_int,
    pub coefficient: *mut *mut libc::c_double,
    pub max_width: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Pattern {
    pub string: *mut libc::c_char,
    pub next: *mut HTS_Pattern,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Question {
    pub string: *mut libc::c_char,
    pub head: *mut HTS_Pattern,
    pub next: *mut HTS_Question,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Node {
    pub index: libc::c_int,
    pub pdf: size_t,
    pub yes: *mut HTS_Node,
    pub no: *mut HTS_Node,
    pub next: *mut HTS_Node,
    pub quest: *mut HTS_Question,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Tree {
    pub head: *mut HTS_Pattern,
    pub next: *mut HTS_Tree,
    pub root: *mut HTS_Node,
    pub state: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Model {
    pub vector_length: size_t,
    pub num_windows: size_t,
    pub is_msd: HTS_Boolean,
    pub ntree: size_t,
    pub npdf: *mut size_t,
    pub pdf: *mut *mut *mut libc::c_float,
    pub tree: *mut HTS_Tree,
    pub question: *mut HTS_Question,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_ModelSet {
    pub hts_voice_version: *mut libc::c_char,
    pub sampling_frequency: size_t,
    pub frame_period: size_t,
    pub num_voices: size_t,
    pub num_states: size_t,
    pub num_streams: size_t,
    pub stream_type: *mut libc::c_char,
    pub fullcontext_format: *mut libc::c_char,
    pub fullcontext_version: *mut libc::c_char,
    pub gv_off_context: *mut HTS_Question,
    pub option: *mut *mut libc::c_char,
    pub duration: *mut HTS_Model,
    pub window: *mut HTS_Window,
    pub stream: *mut *mut HTS_Model,
    pub gv: *mut *mut HTS_Model,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_LabelString {
    pub next: *mut HTS_LabelString,
    pub name: *mut libc::c_char,
    pub start: libc::c_double,
    pub end: libc::c_double,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Label {
    pub head: *mut HTS_LabelString,
    pub size: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_SStream {
    pub vector_length: size_t,
    pub mean: *mut *mut libc::c_double,
    pub vari: *mut *mut libc::c_double,
    pub msd: *mut libc::c_double,
    pub win_size: size_t,
    pub win_l_width: *mut libc::c_int,
    pub win_r_width: *mut libc::c_int,
    pub win_coefficient: *mut *mut libc::c_double,
    pub win_max_width: size_t,
    pub gv_mean: *mut libc::c_double,
    pub gv_vari: *mut libc::c_double,
    pub gv_switch: *mut HTS_Boolean,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_SStreamSet {
    pub sstream: *mut HTS_SStream,
    pub nstream: size_t,
    pub nstate: size_t,
    pub duration: *mut size_t,
    pub total_state: size_t,
    pub total_frame: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_SMatrices {
    pub mean: *mut *mut libc::c_double,
    pub ivar: *mut *mut libc::c_double,
    pub g: *mut libc::c_double,
    pub wuw: *mut *mut libc::c_double,
    pub wum: *mut libc::c_double,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_PStream {
    pub vector_length: size_t,
    pub length: size_t,
    pub width: size_t,
    pub par: *mut *mut libc::c_double,
    pub sm: HTS_SMatrices,
    pub win_size: size_t,
    pub win_l_width: *mut libc::c_int,
    pub win_r_width: *mut libc::c_int,
    pub win_coefficient: *mut *mut libc::c_double,
    pub msd_flag: *mut HTS_Boolean,
    pub gv_mean: *mut libc::c_double,
    pub gv_vari: *mut libc::c_double,
    pub gv_switch: *mut HTS_Boolean,
    pub gv_length: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_PStreamSet {
    pub pstream: *mut HTS_PStream,
    pub nstream: size_t,
    pub total_frame: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_GStream {
    pub vector_length: size_t,
    pub par: *mut *mut libc::c_double,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_GStreamSet {
    pub total_nsample: size_t,
    pub total_frame: size_t,
    pub nstream: size_t,
    pub gstream: *mut HTS_GStream,
    pub gspeech: *mut libc::c_double,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Condition {
    pub sampling_frequency: size_t,
    pub fperiod: size_t,
    pub audio_buff_size: size_t,
    pub stop: HTS_Boolean,
    pub volume: libc::c_double,
    pub msd_threshold: *mut libc::c_double,
    pub gv_weight: *mut libc::c_double,
    pub phoneme_alignment_flag: HTS_Boolean,
    pub speed: libc::c_double,
    pub stage: size_t,
    pub use_log_gain: HTS_Boolean,
    pub alpha: libc::c_double,
    pub beta: libc::c_double,
    pub additional_half_tone: libc::c_double,
    pub duration_iw: *mut libc::c_double,
    pub parameter_iw: *mut *mut libc::c_double,
    pub gv_iw: *mut *mut libc::c_double,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Engine {
    pub condition: HTS_Condition,
    pub audio: HTS_Audio,
    pub ms: HTS_ModelSet,
    pub label: HTS_Label,
    pub sss: HTS_SStreamSet,
    pub pss: HTS_PStreamSet,
    pub gss: HTS_GStreamSet,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_Vocoder {
    pub is_first: HTS_Boolean,
    pub stage: size_t,
    pub gamma: libc::c_double,
    pub use_log_gain: HTS_Boolean,
    pub fprd: size_t,
    pub next: libc::c_ulong,
    pub gauss: HTS_Boolean,
    pub rate: libc::c_double,
    pub pitch_of_curr_point: libc::c_double,
    pub pitch_counter: libc::c_double,
    pub pitch_inc_per_point: libc::c_double,
    pub excite_ring_buff: *mut libc::c_double,
    pub excite_buff_size: size_t,
    pub excite_buff_index: size_t,
    pub sw: libc::c_uchar,
    pub x: libc::c_int,
    pub freqt_buff: *mut libc::c_double,
    pub freqt_size: size_t,
    pub spectrum2en_buff: *mut libc::c_double,
    pub spectrum2en_size: size_t,
    pub r1: libc::c_double,
    pub r2: libc::c_double,
    pub s: libc::c_double,
    pub postfilter_buff: *mut libc::c_double,
    pub postfilter_size: size_t,
    pub c: *mut libc::c_double,
    pub cc: *mut libc::c_double,
    pub cinc: *mut libc::c_double,
    pub d1: *mut libc::c_double,
    pub lsp2lpc_buff: *mut libc::c_double,
    pub lsp2lpc_size: size_t,
    pub gc2gc_buff: *mut libc::c_double,
    pub gc2gc_size: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HTS_File {
    pub type_0: libc::c_uchar,
    pub pointer: *mut libc::c_void,
}

#[macro_export]
macro_rules! HTS_error {
    ($error:expr,$message:expr,$args:expr,) => {};
    ($error:expr,$message:expr,$args:expr) => {};
    ($error:expr,$message:expr) => {};
    ($error:expr,$message:expr,) => {};
}
