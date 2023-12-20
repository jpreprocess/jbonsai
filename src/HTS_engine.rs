use std::rc::Rc;

use libc::FILE;

use crate::label::Label;
use crate::model::ModelSet;
use crate::sstream::SStreamSet;
use crate::{util::*, HTS_GStreamSet, HTS_Label, HTS_ModelSet, HTS_PStreamSet, HTS_SStreamSet};

extern "C" {
    fn atof(__nptr: *const libc::c_char) -> f64;
    fn atoi(__nptr: *const libc::c_char) -> libc::c_int;
    fn strstr(_: *const libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn exp(_: f64) -> f64;
    fn log(_: f64) -> f64;
    fn fprintf(_: *mut FILE, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn fwrite(
        _: *const libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
}

use crate::{
    HTS_GStreamSet_clear, HTS_GStreamSet_create, HTS_GStreamSet_get_parameter,
    HTS_GStreamSet_get_speech, HTS_GStreamSet_get_total_frame, HTS_GStreamSet_get_total_nsamples,
    HTS_GStreamSet_get_vector_length, HTS_GStreamSet_initialize, HTS_Label_clear,
    HTS_Label_get_size, HTS_Label_get_string, HTS_Label_initialize, HTS_Label_load_from_fn,
    HTS_Label_load_from_strings, HTS_ModelSet_clear, HTS_ModelSet_get_duration_index,
    HTS_ModelSet_get_fperiod, HTS_ModelSet_get_fullcontext_label_format,
    HTS_ModelSet_get_fullcontext_label_version, HTS_ModelSet_get_nstate, HTS_ModelSet_get_nstream,
    HTS_ModelSet_get_nvoices, HTS_ModelSet_get_option, HTS_ModelSet_get_parameter_index,
    HTS_ModelSet_get_sampling_frequency, HTS_ModelSet_get_vector_length,
    HTS_ModelSet_get_window_size, HTS_ModelSet_initialize, HTS_ModelSet_is_msd, HTS_ModelSet_load,
    HTS_ModelSet_use_gv, HTS_PStreamSet_clear, HTS_PStreamSet_create,
    HTS_PStreamSet_get_total_frame, HTS_PStreamSet_initialize, HTS_SStreamSet_clear,
    HTS_SStreamSet_create, HTS_SStreamSet_get_duration, HTS_SStreamSet_get_mean,
    HTS_SStreamSet_get_msd, HTS_SStreamSet_get_total_state, HTS_SStreamSet_initialize,
    HTS_SStreamSet_set_mean, HTS_calloc, HTS_free, HTS_fwrite_little_endian,
};

#[derive(Clone)]
pub struct HTS_Condition {
    pub sampling_frequency: usize,
    pub fperiod: usize,
    pub stop: bool,
    pub volume: f64,
    pub msd_threshold: Vec<f64>,
    pub gv_weight: Vec<f64>,
    pub phoneme_alignment_flag: bool,
    pub speed: f64,
    pub stage: usize,
    pub use_log_gain: bool,
    pub alpha: f64,
    pub beta: f64,
    pub additional_half_tone: f64,
    pub duration_iw: Vec<f64>,
    pub parameter_iw: Vec<Vec<f64>>,
    pub gv_iw: Vec<Vec<f64>>,
}

// #[derive(Clone)]
pub struct HTS_Engine {
    pub condition: HTS_Condition,
    pub ms: Rc<ModelSet>,
    pub label: Option<Label>,
    pub sss: Option<SStreamSet>,
    pub pss: HTS_PStreamSet,
    pub gss: HTS_GStreamSet,
}

pub fn HTS_Engine_load(voices: &Vec<String>) -> HTS_Engine {
    let mut condition = HTS_Condition {
        sampling_frequency: 0,
        fperiod: 0,
        stop: false,
        volume: 1.0f64,
        msd_threshold: Vec::new(),
        gv_weight: Vec::new(),
        speed: 1.0f64,
        phoneme_alignment_flag: false,
        stage: 0,
        use_log_gain: false,
        alpha: 0.0f64,
        beta: 0.0f64,
        additional_half_tone: 0.0f64,
        duration_iw: Vec::new(),
        parameter_iw: Vec::new(),
        gv_iw: Vec::new(),
    };

    /* load voices */
    let ms = ModelSet::load_htsvoice_files(voices).unwrap();
    let nstream = ms.get_nstream();
    let average_weight = 1.0f64 / voices.len() as f64;

    /* global */
    condition.sampling_frequency = ms.get_sampling_frequency() as usize;
    condition.fperiod = ms.get_fperiod();
    condition.msd_threshold = (0..nstream).into_iter().map(|_| 0.5).collect();
    condition.gv_weight = (0..nstream).into_iter().map(|_| 1.0).collect();

    /* spectrum */
    for option in ms.get_option(0) {
        let Some((key, value)) = option.split_once('=') else {
            eprintln!("Skipped unrecognized option {}.", option);
            continue;
        };
        match key {
            "GAMMA" => condition.stage = value.parse().unwrap(),
            "LN_GAIN" => condition.use_log_gain = value == "1",
            "ALPHA" => condition.alpha = value.parse().unwrap(),
            _ => eprintln!("Skipped unrecognized option {}.", option),
        }
    }

    /* interpolation weights */
    condition.duration_iw = (0..voices.len())
        .into_iter()
        .map(|_| average_weight)
        .collect();
    condition.parameter_iw = (0..voices.len())
        .into_iter()
        .map(|_| (0..nstream).into_iter().map(|_| average_weight).collect())
        .collect();
    condition.gv_iw = (0..voices.len())
        .into_iter()
        .map(|_| (0..nstream).into_iter().map(|_| average_weight).collect())
        .collect();

    HTS_Engine {
        condition,
        ms: Rc::new(ms),
        label: None,
        sss: None,
        pss: HTS_PStreamSet_initialize(),
        gss: HTS_GStreamSet_initialize(),
    }
}

pub fn HTS_Engine_set_sampling_frequency(engine: &mut HTS_Engine, mut i: usize) {
    if i < 1 {
        i = 1;
    }
    engine.condition.sampling_frequency = i;
}

pub fn HTS_Engine_get_sampling_frequency(engine: &mut HTS_Engine) -> usize {
    engine.condition.sampling_frequency
}

pub fn HTS_Engine_set_fperiod(engine: &mut HTS_Engine, mut i: usize) {
    if i < 1 {
        i = 1;
    }
    engine.condition.fperiod = i;
}

pub fn HTS_Engine_get_fperiod(engine: &mut HTS_Engine) -> usize {
    engine.condition.fperiod
}

pub fn HTS_Engine_set_stop_flag(engine: &mut HTS_Engine, b: bool) {
    engine.condition.stop = b;
}

pub fn HTS_Engine_get_stop_flag(engine: &mut HTS_Engine) -> bool {
    engine.condition.stop
}

pub fn HTS_Engine_set_volume(engine: &mut HTS_Engine, f: f64) {
    engine.condition.volume = (f * DB).exp();
}

pub fn HTS_Engine_get_volume(engine: &mut HTS_Engine) -> f64 {
    engine.condition.volume.ln() / DB
}

pub fn HTS_Engine_set_msd_threshold(engine: &mut HTS_Engine, stream_index: usize, mut f: f64) {
    if f < 0.0 {
        f = 0.0;
    }
    if f > 1.0 {
        f = 1.0;
    }
    engine.condition.msd_threshold[stream_index] = f;
}

pub fn HTS_Engine_get_msd_threshold(engine: &mut HTS_Engine, stream_index: usize) -> f64 {
    engine.condition.msd_threshold[stream_index]
}

pub fn HTS_Engine_set_gv_weight(engine: &mut HTS_Engine, stream_index: usize, mut f: f64) {
    if f < 0.0 {
        f = 0.0;
    }
    engine.condition.gv_weight[stream_index] = f;
}

pub fn HTS_Engine_get_gv_weight(engine: &mut HTS_Engine, stream_index: usize) -> f64 {
    engine.condition.gv_weight[stream_index]
}

pub fn HTS_Engine_set_speed(engine: &mut HTS_Engine, mut f: f64) {
    if f < 1.0E-06f64 {
        f = 1.0E-06f64;
    }
    engine.condition.speed = f;
}

pub fn HTS_Engine_set_phoneme_alignment_flag(engine: &mut HTS_Engine, b: bool) {
    engine.condition.phoneme_alignment_flag = b;
}

pub fn HTS_Engine_set_alpha(engine: &mut HTS_Engine, mut f: f64) {
    if f < 0.0f64 {
        f = 0.0f64;
    }
    if f > 1.0f64 {
        f = 1.0f64;
    }
    engine.condition.alpha = f;
}

pub fn HTS_Engine_get_alpha(engine: &mut HTS_Engine) -> f64 {
    engine.condition.alpha
}

pub fn HTS_Engine_set_beta(engine: &mut HTS_Engine, mut f: f64) {
    if f < 0.0f64 {
        f = 0.0f64;
    }
    if f > 1.0f64 {
        f = 1.0f64;
    }
    engine.condition.beta = f;
}

pub fn HTS_Engine_get_beta(engine: &mut HTS_Engine) -> f64 {
    engine.condition.beta
}

pub fn HTS_Engine_add_half_tone(engine: &mut HTS_Engine, f: f64) {
    engine.condition.additional_half_tone = f;
}

pub fn HTS_Engine_set_duration_interpolation_weight(
    engine: &mut HTS_Engine,
    voice_index: usize,
    f: f64,
) {
    engine.condition.duration_iw[voice_index] = f;
}

pub fn HTS_Engine_get_duration_interpolation_weight(
    engine: &mut HTS_Engine,
    voice_index: usize,
) -> f64 {
    engine.condition.duration_iw[voice_index]
}

pub fn HTS_Engine_set_parameter_interpolation_weight(
    engine: &mut HTS_Engine,
    voice_index: usize,
    stream_index: usize,
    f: f64,
) {
    engine.condition.parameter_iw[voice_index][stream_index] = f;
}

pub fn HTS_Engine_get_parameter_interpolation_weight(
    engine: &mut HTS_Engine,
    voice_index: usize,
    stream_index: usize,
) -> f64 {
    engine.condition.parameter_iw[voice_index][stream_index]
}

pub fn HTS_Engine_set_gv_interpolation_weight(
    engine: &mut HTS_Engine,
    voice_index: usize,
    stream_index: usize,
    f: f64,
) {
    engine.condition.gv_iw[voice_index][stream_index] = f;
}

pub fn HTS_Engine_get_gv_interpolation_weight(
    engine: &mut HTS_Engine,
    voice_index: usize,
    stream_index: usize,
) -> f64 {
    engine.condition.gv_iw[voice_index][stream_index]
}

pub fn HTS_Engine_get_total_state(engine: &mut HTS_Engine) -> size_t {
    engine.sss.as_ref().unwrap().get_total_state() as u64
}

pub fn HTS_Engine_set_state_mean(
    engine: &mut HTS_Engine,
    stream_index: size_t,
    state_index: size_t,
    vector_index: size_t,
    f: f64,
) {
    engine
        .sss
        .as_mut()
        .unwrap()
        .set_mean(stream_index as usize, state_index as usize, vector_index as usize, f);
}

pub fn HTS_Engine_get_state_mean(
    engine: &mut HTS_Engine,
    stream_index: size_t,
    state_index: size_t,
    vector_index: size_t,
) -> f64 {
    engine
        .sss
        .as_ref()
        .unwrap()
        .get_mean(stream_index as usize, state_index as usize, vector_index as usize)
}

pub fn HTS_Engine_get_state_duration(
    engine: &mut HTS_Engine,
    state_index: size_t,
) -> size_t {
    engine.sss.as_ref().unwrap().get_duration(state_index as usize) as u64
}

pub fn HTS_Engine_get_nvoices(engine: &mut HTS_Engine) -> usize {
    engine.ms.get_nvoices()
}

pub fn HTS_Engine_get_nstream(engine: &mut HTS_Engine) -> usize {
    engine.ms.get_nstream()
}

pub fn HTS_Engine_get_nstate(engine: &mut HTS_Engine) -> usize {
    engine.ms.get_nstate()
}

pub fn HTS_Engine_get_fullcontext_label_format(engine: &mut HTS_Engine) -> &str {
    engine.ms.get_fullcontext_label_format()
}

pub fn HTS_Engine_get_fullcontext_label_version(engine: &mut HTS_Engine) -> &str {
    engine.ms.get_fullcontext_label_version()
}

pub unsafe fn HTS_Engine_get_total_frame(engine: &mut HTS_Engine) -> size_t {
    HTS_GStreamSet_get_total_frame(&mut engine.gss)
}

pub unsafe fn HTS_Engine_get_nsamples(engine: &mut HTS_Engine) -> size_t {
    HTS_GStreamSet_get_total_nsamples(&mut engine.gss)
}

pub unsafe fn HTS_Engine_get_generated_parameter(
    engine: &mut HTS_Engine,
    stream_index: size_t,
    frame_index: size_t,
    vector_index: size_t,
) -> f64 {
    HTS_GStreamSet_get_parameter(&mut engine.gss, stream_index, frame_index, vector_index)
}

pub unsafe fn HTS_Engine_get_generated_speech(engine: &mut HTS_Engine, index: size_t) -> f64 {
    HTS_GStreamSet_get_speech(&mut engine.gss, index)
}
fn HTS_Engine_generate_state_sequence(engine: &mut HTS_Engine) -> bool {
    let mut i: size_t = 0;
    let mut state_index = 0;
    let mut model_index: size_t = 0;
    let mut f: f64 = 0.;
    engine.sss = SStreamSet::create(
        engine.ms.clone(),
        engine.label.as_ref().unwrap(),
        engine.condition.phoneme_alignment_flag,
        engine.condition.speed,
        &mut engine.condition.duration_iw,
        &mut engine.condition.parameter_iw,
        &mut engine.condition.gv_iw,
    );
    if engine.condition.additional_half_tone != 0.0f64 {
        state_index = 0 as libc::c_int as size_t;
        model_index = 0 as libc::c_int as size_t;
        i = 0 as libc::c_int as size_t;
        while i < HTS_Engine_get_total_state(engine) {
            f = HTS_Engine_get_state_mean(
                engine,
                1 as libc::c_int as size_t,
                i,
                0 as libc::c_int as size_t,
            );
            f += engine.condition.additional_half_tone * HALF_TONE;
            if f < MIN_LF0 {
                f = MIN_LF0;
            } else if f > MAX_LF0 {
                f = MAX_LF0;
            }
            HTS_Engine_set_state_mean(
                engine,
                1 as libc::c_int as size_t,
                i,
                0 as libc::c_int as size_t,
                f,
            );
            state_index = state_index.wrapping_add(1);
            if state_index as usize >= HTS_Engine_get_nstate(engine) {
                state_index = 0 as libc::c_int as size_t;
                model_index = model_index.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
    true
}

// pub unsafe fn HTS_Engine_generate_state_sequence_from_fn(
//     engine: &mut HTS_Engine,
//     fn_0: *const libc::c_char,
// ) -> bool {
//     HTS_Engine_refresh(engine);
//     HTS_Label_load_from_fn(
//         &mut engine.label,
//         engine.condition.sampling_frequency as u64,
//         engine.condition.fperiod as u64,
//         fn_0,
//     );
//     HTS_Engine_generate_state_sequence(engine)
// }

pub unsafe fn HTS_Engine_generate_state_sequence_from_strings(
    engine: &mut HTS_Engine,
    lines: &[String],
) -> bool {
    HTS_Engine_refresh(engine);
    engine.label = Some(Label::load_from_strings(
        engine.condition.sampling_frequency,
        engine.condition.fperiod,
        lines,
    ));
    HTS_Engine_generate_state_sequence(engine)
}

pub unsafe fn HTS_Engine_generate_parameter_sequence(engine: &mut HTS_Engine) -> bool {
    HTS_PStreamSet_create(
        &mut engine.pss,
        engine.sss.as_ref().unwrap(),
        engine.condition.msd_threshold.as_mut_ptr(),
        engine.condition.gv_weight.as_mut_ptr(),
    )
}

pub unsafe fn HTS_Engine_generate_sample_sequence(engine: &mut HTS_Engine) -> bool {
    HTS_GStreamSet_create(
        &mut engine.gss,
        &mut engine.pss,
        engine.condition.stage as u64,
        engine.condition.use_log_gain,
        engine.condition.sampling_frequency as u64,
        engine.condition.fperiod as u64,
        engine.condition.alpha,
        engine.condition.beta,
        engine.condition.stop,
        engine.condition.volume,
    )
}
unsafe fn HTS_Engine_synthesize(engine: &mut HTS_Engine) -> bool {
    if HTS_Engine_generate_state_sequence(engine) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_refresh(engine);
        return false;
    }
    if HTS_Engine_generate_parameter_sequence(engine) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_refresh(engine);
        return false;
    }
    if HTS_Engine_generate_sample_sequence(engine) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_refresh(engine);
        return false;
    }
    true
}

// pub unsafe fn HTS_Engine_synthesize_from_fn(
//     engine: &mut HTS_Engine,
//     fn_0: *const libc::c_char,
// ) -> bool {
//     HTS_Engine_refresh(engine);
//     HTS_Label_load_from_fn(
//         &mut engine.label,
//         engine.condition.sampling_frequency as u64,
//         engine.condition.fperiod as u64,
//         fn_0,
//     );
//     HTS_Engine_synthesize(engine)
// }

pub unsafe fn HTS_Engine_synthesize_from_strings(
    engine: &mut HTS_Engine,
    lines: &[String],
) -> bool {
    HTS_Engine_refresh(engine);
    engine.label = Some(Label::load_from_strings(
        engine.condition.sampling_frequency,
        engine.condition.fperiod,
        lines,
    ));
    HTS_Engine_synthesize(engine)
}

pub unsafe fn HTS_Engine_save_information(engine: &mut HTS_Engine, fp: *mut FILE) {
    // let mut i: size_t = 0;
    // let mut j: size_t = 0;
    // let mut k: size_t = 0;
    // let mut l: size_t = 0;
    // let mut m: size_t = 0;
    // let mut n: size_t = 0;
    // let mut temp: f64 = 0.;
    // let condition: &mut HTS_Condition = &mut engine.condition;
    // let ms: &mut HTS_ModelSet = &mut engine.ms;
    // let label: &mut HTS_Label = &mut engine.label;
    // let sss = &mut engine.sss;
    // let pss = &mut engine.pss;
    // fprintf(
    //     fp,
    //     b"[Global parameter]\n\0" as *const u8 as *const libc::c_char,
    // );
    // fprintf(
    //     fp,
    //     b"Sampring frequency                     -> %8lu(Hz)\n\0" as *const u8
    //         as *const libc::c_char,
    //     condition.sampling_frequency,
    // );
    // fprintf(
    //     fp,
    //     b"Frame period                           -> %8lu(point)\n\0" as *const u8
    //         as *const libc::c_char,
    //     condition.fperiod,
    // );
    // fprintf(
    //     fp,
    //     b"                                          %8.5f(msec)\n\0" as *const u8
    //         as *const libc::c_char,
    //     1e+3f64 * condition.fperiod as f64
    //         / condition.sampling_frequency as f64,
    // );
    // fprintf(
    //     fp,
    //     b"All-pass constant                      -> %8.5f\n\0" as *const u8 as *const libc::c_char,
    //     condition.alpha as libc::c_float as f64,
    // );
    // fprintf(
    //     fp,
    //     b"Gamma                                  -> %8.5f\n\0" as *const u8 as *const libc::c_char,
    //     (if condition.stage == 0 as libc::c_int as size_t {
    //         0.0f64
    //     } else {
    //         -1.0f64 / condition.stage as f64
    //     }) as libc::c_float as f64,
    // );
    // if condition.stage != 0 as libc::c_int as size_t {
    //     if condition.use_log_gain as libc::c_int == 1 as libc::c_int {
    //         fprintf(
    //             fp,
    //             b"Log gain flag                          ->     TRUE\n\0" as *const u8
    //                 as *const libc::c_char,
    //         );
    //     } else {
    //         fprintf(
    //             fp,
    //             b"Log gain flag                          ->    FALSE\n\0" as *const u8
    //                 as *const libc::c_char,
    //         );
    //     }
    // }
    // fprintf(
    //     fp,
    //     b"Postfiltering coefficient              -> %8.5f\n\0" as *const u8 as *const libc::c_char,
    //     condition.beta as libc::c_float as f64,
    // );
    // fprintf(
    //     fp,
    //     b"Audio buffer size                      -> %8lu(sample)\n\0" as *const u8
    //         as *const libc::c_char,
    //     condition.audio_buff_size,
    // );
    // fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
    // fprintf(
    //     fp,
    //     b"[Duration parameter]\n\0" as *const u8 as *const libc::c_char,
    // );
    // fprintf(
    //     fp,
    //     b"Number of states                       -> %8lu\n\0" as *const u8 as *const libc::c_char,
    //     HTS_ModelSet_get_nstate(ms),
    // );
    // fprintf(
    //     fp,
    //     b"         Interpolation size            -> %8lu\n\0" as *const u8 as *const libc::c_char,
    //     HTS_ModelSet_get_nvoices(ms),
    // );
    // i = 0 as libc::c_int as size_t;
    // temp = 0.0f64;
    // while i < HTS_ModelSet_get_nvoices(ms) {
    //     temp += *(condition.duration_iw).offset(i as isize);
    //     i = i.wrapping_add(1);
    // }
    // i = 0 as libc::c_int as size_t;
    // while i < HTS_ModelSet_get_nvoices(ms) {
    //     if *(condition.duration_iw).offset(i as isize) != 0.0f64 {
    //         *(condition.duration_iw).offset(i as isize) /= temp;
    //     }
    //     i = i.wrapping_add(1);
    // }
    // i = 0 as libc::c_int as size_t;
    // while i < HTS_ModelSet_get_nvoices(ms) {
    //     fprintf(
    //         fp,
    //         b"         Interpolation weight[%2lu]      -> %8.0f(%%)\n\0" as *const u8
    //             as *const libc::c_char,
    //         i,
    //         (100 as libc::c_int as f64 * *(condition.duration_iw).offset(i as isize))
    //             as libc::c_float as f64,
    //     );
    //     i = i.wrapping_add(1);
    // }
    // fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
    // fprintf(
    //     fp,
    //     b"[Stream parameter]\n\0" as *const u8 as *const libc::c_char,
    // );
    // i = 0 as libc::c_int as size_t;
    // while i < HTS_ModelSet_get_nstream(ms) {
    //     fprintf(
    //         fp,
    //         b"Stream[%2lu] vector length               -> %8lu\n\0" as *const u8
    //             as *const libc::c_char,
    //         i,
    //         HTS_ModelSet_get_vector_length(ms, i),
    //     );
    //     fprintf(
    //         fp,
    //         b"           Dynamic window size         -> %8lu\n\0" as *const u8
    //             as *const libc::c_char,
    //         HTS_ModelSet_get_window_size(ms, i),
    //     );
    //     fprintf(
    //         fp,
    //         b"           Interpolation size          -> %8lu\n\0" as *const u8
    //             as *const libc::c_char,
    //         HTS_ModelSet_get_nvoices(ms),
    //     );
    //     j = 0 as libc::c_int as size_t;
    //     temp = 0.0f64;
    //     while j < HTS_ModelSet_get_nvoices(ms) {
    //         temp += *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize);
    //         j = j.wrapping_add(1);
    //     }
    //     j = 0 as libc::c_int as size_t;
    //     while j < HTS_ModelSet_get_nvoices(ms) {
    //         if *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize) != 0.0f64 {
    //             *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize) /= temp;
    //         }
    //         j = j.wrapping_add(1);
    //     }
    //     j = 0 as libc::c_int as size_t;
    //     while j < HTS_ModelSet_get_nvoices(ms) {
    //         fprintf(
    //             fp,
    //             b"           Interpolation weight[%2lu]    -> %8.0f(%%)\n\0" as *const u8
    //                 as *const libc::c_char,
    //             j,
    //             (100 as libc::c_int as f64
    //                 * *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize))
    //                 as libc::c_float as f64,
    //         );
    //         j = j.wrapping_add(1);
    //     }
    //     if HTS_ModelSet_is_msd(ms, i) != 0 {
    //         fprintf(
    //             fp,
    //             b"           MSD flag                    ->     TRUE\n\0" as *const u8
    //                 as *const libc::c_char,
    //         );
    //         fprintf(
    //             fp,
    //             b"           MSD threshold               -> %8.5f\n\0" as *const u8
    //                 as *const libc::c_char,
    //             *(condition.msd_threshold).offset(i as isize),
    //         );
    //     } else {
    //         fprintf(
    //             fp,
    //             b"           MSD flag                    ->    FALSE\n\0" as *const u8
    //                 as *const libc::c_char,
    //         );
    //     }
    //     if HTS_ModelSet_use_gv(ms, i) != 0 {
    //         fprintf(
    //             fp,
    //             b"           GV flag                     ->     TRUE\n\0" as *const u8
    //                 as *const libc::c_char,
    //         );
    //         fprintf(
    //             fp,
    //             b"           GV weight                   -> %8.0f(%%)\n\0" as *const u8
    //                 as *const libc::c_char,
    //             (100 as libc::c_int as f64 * *(condition.gv_weight).offset(i as isize))
    //                 as libc::c_float as f64,
    //         );
    //         fprintf(
    //             fp,
    //             b"           GV interpolation size       -> %8lu\n\0" as *const u8
    //                 as *const libc::c_char,
    //             HTS_ModelSet_get_nvoices(ms),
    //         );
    //         j = 0 as libc::c_int as size_t;
    //         temp = 0.0f64;
    //         while j < HTS_ModelSet_get_nvoices(ms) {
    //             temp += *(*(condition.gv_iw).offset(j as isize)).offset(i as isize);
    //             j = j.wrapping_add(1);
    //         }
    //         j = 0 as libc::c_int as size_t;
    //         while j < HTS_ModelSet_get_nvoices(ms) {
    //             if *(*(condition.gv_iw).offset(j as isize)).offset(i as isize) != 0.0f64 {
    //                 *(*(condition.gv_iw).offset(j as isize)).offset(i as isize) /= temp;
    //             }
    //             j = j.wrapping_add(1);
    //         }
    //         j = 0 as libc::c_int as size_t;
    //         while j < HTS_ModelSet_get_nvoices(ms) {
    //             fprintf(
    //                 fp,
    //                 b"           GV interpolation weight[%2lu] -> %8.0f(%%)\n\0" as *const u8
    //                     as *const libc::c_char,
    //                 j,
    //                 (100 as libc::c_int as f64
    //                     * *(*(condition.gv_iw).offset(j as isize)).offset(i as isize))
    //                     as libc::c_float as f64,
    //             );
    //             j = j.wrapping_add(1);
    //         }
    //     } else {
    //         fprintf(
    //             fp,
    //             b"           GV flag                     ->    FALSE\n\0" as *const u8
    //                 as *const libc::c_char,
    //         );
    //     }
    //     i = i.wrapping_add(1);
    // }
    // fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
    // fprintf(
    //     fp,
    //     b"[Generated sequence]\n\0" as *const u8 as *const libc::c_char,
    // );
    // fprintf(
    //     fp,
    //     b"Number of HMMs                         -> %8lu\n\0" as *const u8 as *const libc::c_char,
    //     HTS_Label_get_size(label),
    // );
    // fprintf(
    //     fp,
    //     b"Number of stats                        -> %8lu\n\0" as *const u8 as *const libc::c_char,
    //     (HTS_Label_get_size(label)).wrapping_mul(HTS_ModelSet_get_nstate(ms)),
    // );
    // fprintf(
    //     fp,
    //     b"Length of this speech                  -> %8.3f(sec)\n\0" as *const u8
    //         as *const libc::c_char,
    //     (HTS_PStreamSet_get_total_frame(pss) as f64
    //         * condition.fperiod as f64
    //         / condition.sampling_frequency as f64) as libc::c_float
    //         as f64,
    // );
    // fprintf(
    //     fp,
    //     b"                                       -> %8lu(frames)\n\0" as *const u8
    //         as *const libc::c_char,
    //     (HTS_PStreamSet_get_total_frame(pss)).wrapping_mul(condition.fperiod),
    // );
    // i = 0 as libc::c_int as size_t;
    // while i < HTS_Label_get_size(label) {
    //     fprintf(fp, b"HMM[%2lu]\n\0" as *const u8 as *const libc::c_char, i);
    //     fprintf(
    //         fp,
    //         b"  Name                                 -> %s\n\0" as *const u8 as *const libc::c_char,
    //         HTS_Label_get_string(label, i),
    //     );
    //     fprintf(fp, b"  Duration\n\0" as *const u8 as *const libc::c_char);
    //     j = 0 as libc::c_int as size_t;
    //     while j < HTS_ModelSet_get_nvoices(ms) {
    //         fprintf(
    //             fp,
    //             b"    Interpolation[%2lu]\n\0" as *const u8 as *const libc::c_char,
    //             j,
    //         );
    //         HTS_ModelSet_get_duration_index(ms, j, HTS_Label_get_string(label, i), &mut k, &mut l);
    //         fprintf(
    //             fp,
    //             b"      Tree index                       -> %8lu\n\0" as *const u8
    //                 as *const libc::c_char,
    //             k,
    //         );
    //         fprintf(
    //             fp,
    //             b"      PDF index                        -> %8lu\n\0" as *const u8
    //                 as *const libc::c_char,
    //             l,
    //         );
    //         j = j.wrapping_add(1);
    //     }
    //     j = 0 as libc::c_int as size_t;
    //     while j < HTS_ModelSet_get_nstate(ms) {
    //         fprintf(
    //             fp,
    //             b"  State[%2lu]\n\0" as *const u8 as *const libc::c_char,
    //             j.wrapping_add(2 as libc::c_int as libc::c_ulong),
    //         );
    //         fprintf(
    //             fp,
    //             b"    Length                             -> %8lu(frames)\n\0" as *const u8
    //                 as *const libc::c_char,
    //             HTS_SStreamSet_get_duration(sss, (i * HTS_ModelSet_get_nstate(ms)).wrapping_add(j)),
    //         );
    //         k = 0 as libc::c_int as size_t;
    //         while k < HTS_ModelSet_get_nstream(ms) {
    //             fprintf(
    //                 fp,
    //                 b"    Stream[%2lu]\n\0" as *const u8 as *const libc::c_char,
    //                 k,
    //             );
    //             if HTS_ModelSet_is_msd(ms, k) != 0 {
    //                 if HTS_SStreamSet_get_msd(
    //                     sss,
    //                     k,
    //                     (i * HTS_ModelSet_get_nstate(ms)).wrapping_add(j),
    //                 ) > *(condition.msd_threshold).offset(k as isize)
    //                 {
    //                     fprintf(
    //                         fp,
    //                         b"      MSD flag                         ->     TRUE\n\0" as *const u8
    //                             as *const libc::c_char,
    //                     );
    //                 } else {
    //                     fprintf(
    //                         fp,
    //                         b"      MSD flag                         ->    FALSE\n\0" as *const u8
    //                             as *const libc::c_char,
    //                     );
    //                 }
    //             }
    //             l = 0 as libc::c_int as size_t;
    //             while l < HTS_ModelSet_get_nvoices(ms) {
    //                 fprintf(
    //                     fp,
    //                     b"      Interpolation[%2lu]\n\0" as *const u8 as *const libc::c_char,
    //                     l,
    //                 );
    //                 HTS_ModelSet_get_parameter_index(
    //                     ms,
    //                     l,
    //                     k,
    //                     j.wrapping_add(2 as libc::c_int as size_t),
    //                     HTS_Label_get_string(label, i),
    //                     &mut m,
    //                     &mut n,
    //                 );
    //                 fprintf(
    //                     fp,
    //                     b"        Tree index                     -> %8lu\n\0" as *const u8
    //                         as *const libc::c_char,
    //                     m,
    //                 );
    //                 fprintf(
    //                     fp,
    //                     b"        PDF index                      -> %8lu\n\0" as *const u8
    //                         as *const libc::c_char,
    //                     n,
    //                 );
    //                 l = l.wrapping_add(1);
    //             }
    //             k = k.wrapping_add(1);
    //         }
    //         j = j.wrapping_add(1);
    //     }
    //     i = i.wrapping_add(1);
    // }
}

pub unsafe fn HTS_Engine_save_label(engine: &mut HTS_Engine, fp: *mut FILE) {
    let mut i = 0;
    let mut j = 0;
    let mut frame = 0;
    let mut state = 0;
    let mut duration = 0;
    let label = engine.label.as_ref().unwrap();
    let sss = engine.sss.as_ref().unwrap();
    let nstate = engine.ms.get_nstate();
    let rate: f64 =
        engine.condition.fperiod as f64 * 1.0e+07f64 / engine.condition.sampling_frequency as f64;
    i = 0;
    state = 0;
    frame = 0;
    while i < label.get_size() {
        j = 0;
        duration = 0;
        while j < nstate {
            let fresh2 = state;
            state += 1;
            duration += sss.get_duration(fresh2);
            j = j.wrapping_add(1);
        }
        fprintf(
            fp,
            b"%lu %lu %s\n\0" as *const u8 as *const libc::c_char,
            (frame as f64 * rate) as libc::c_ulong,
            ((frame as f64 + duration as f64) * rate) as libc::c_ulong,
            label.get_string(i),
        );
        frame += duration;
        i = i.wrapping_add(1);
    }
}

pub unsafe fn HTS_Engine_save_generated_parameter(
    engine: &mut HTS_Engine,
    stream_index: size_t,
    fp: *mut FILE,
) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut temp: libc::c_float = 0.;
    let gss: &mut HTS_GStreamSet = &mut engine.gss;
    i = 0 as libc::c_int as size_t;
    while i < HTS_GStreamSet_get_total_frame(gss) {
        j = 0 as libc::c_int as size_t;
        while j < HTS_GStreamSet_get_vector_length(gss, stream_index) {
            temp = HTS_GStreamSet_get_parameter(gss, stream_index, i, j) as libc::c_float;
            fwrite(
                &mut temp as *mut libc::c_float as *const libc::c_void,
                ::core::mem::size_of::<libc::c_float>() as libc::c_ulong,
                1 as libc::c_int as libc::c_ulong,
                fp,
            );
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}

pub unsafe fn HTS_Engine_save_generated_speech(engine: &mut HTS_Engine, fp: *mut FILE) {
    let mut i: size_t = 0;
    let mut x: f64 = 0.;
    let mut temp: libc::c_short = 0;
    let gss: &mut HTS_GStreamSet = &mut engine.gss;
    i = 0 as libc::c_int as size_t;
    while i < HTS_GStreamSet_get_total_nsamples(gss) {
        x = HTS_GStreamSet_get_speech(gss, i);
        if x > 32767.0f64 {
            temp = 32767 as libc::c_int as libc::c_short;
        } else if x < -32768.0f64 {
            temp = -(32768 as libc::c_int) as libc::c_short;
        } else {
            temp = x as libc::c_short;
        }
        fwrite(
            &mut temp as *mut libc::c_short as *const libc::c_void,
            ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
            1 as libc::c_int as libc::c_ulong,
            fp,
        );
        i = i.wrapping_add(1);
    }
}

pub unsafe fn HTS_Engine_save_riff(engine: &mut HTS_Engine, fp: *mut FILE) {
    let mut i: size_t = 0;
    let mut x: f64 = 0.;
    let mut temp: libc::c_short = 0;
    let gss: &mut HTS_GStreamSet = &mut engine.gss;
    let mut data_01_04: [libc::c_char; 4] = [
        'R' as i32 as libc::c_char,
        'I' as i32 as libc::c_char,
        'F' as i32 as libc::c_char,
        'F' as i32 as libc::c_char,
    ];
    let mut data_05_08: libc::c_int = (HTS_GStreamSet_get_total_nsamples(gss))
        .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
        .wrapping_add(36 as libc::c_int as libc::c_ulong)
        as libc::c_int;
    let mut data_09_12: [libc::c_char; 4] = [
        'W' as i32 as libc::c_char,
        'A' as i32 as libc::c_char,
        'V' as i32 as libc::c_char,
        'E' as i32 as libc::c_char,
    ];
    let mut data_13_16: [libc::c_char; 4] = [
        'f' as i32 as libc::c_char,
        'm' as i32 as libc::c_char,
        't' as i32 as libc::c_char,
        ' ' as i32 as libc::c_char,
    ];
    let mut data_17_20: libc::c_int = 16 as libc::c_int;
    let mut data_21_22: libc::c_short = 1 as libc::c_int as libc::c_short;
    let mut data_23_24: libc::c_short = 1 as libc::c_int as libc::c_short;
    let mut data_25_28: libc::c_int = engine.condition.sampling_frequency as libc::c_int;
    let mut data_29_32: libc::c_int = (engine.condition.sampling_frequency as u64)
        .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
        as libc::c_int;
    let mut data_33_34: libc::c_short =
        ::core::mem::size_of::<libc::c_short>() as libc::c_ulong as libc::c_short;
    let mut data_35_36: libc::c_short = (::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
        .wrapping_mul(8 as libc::c_int as libc::c_ulong)
        as libc::c_short;
    let mut data_37_40: [libc::c_char; 4] = [
        'd' as i32 as libc::c_char,
        'a' as i32 as libc::c_char,
        't' as i32 as libc::c_char,
        'a' as i32 as libc::c_char,
    ];
    let mut data_41_44: libc::c_int = (HTS_GStreamSet_get_total_nsamples(gss))
        .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
        as libc::c_int;
    HTS_fwrite_little_endian(
        data_01_04.as_mut_ptr() as *const libc::c_void,
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
        4 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_05_08 as *mut libc::c_int as *const libc::c_void,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        data_09_12.as_mut_ptr() as *const libc::c_void,
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
        4 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        data_13_16.as_mut_ptr() as *const libc::c_void,
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
        4 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_17_20 as *mut libc::c_int as *const libc::c_void,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_21_22 as *mut libc::c_short as *const libc::c_void,
        ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_23_24 as *mut libc::c_short as *const libc::c_void,
        ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_25_28 as *mut libc::c_int as *const libc::c_void,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_29_32 as *mut libc::c_int as *const libc::c_void,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_33_34 as *mut libc::c_short as *const libc::c_void,
        ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_35_36 as *mut libc::c_short as *const libc::c_void,
        ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        data_37_40.as_mut_ptr() as *const libc::c_void,
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
        4 as libc::c_int as size_t,
        fp,
    );
    HTS_fwrite_little_endian(
        &mut data_41_44 as *mut libc::c_int as *const libc::c_void,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        1 as libc::c_int as size_t,
        fp,
    );
    i = 0 as libc::c_int as size_t;
    while i < HTS_GStreamSet_get_total_nsamples(gss) {
        x = HTS_GStreamSet_get_speech(gss, i);
        if x > 32767.0f64 {
            temp = 32767 as libc::c_int as libc::c_short;
        } else if x < -32768.0f64 {
            temp = -(32768 as libc::c_int) as libc::c_short;
        } else {
            temp = x as libc::c_short;
        }
        HTS_fwrite_little_endian(
            &mut temp as *mut libc::c_short as *const libc::c_void,
            ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
            1 as libc::c_int as size_t,
            fp,
        );
        i = i.wrapping_add(1);
    }
}

pub unsafe fn HTS_Engine_refresh(engine: &mut HTS_Engine) {
    HTS_GStreamSet_clear(&mut engine.gss);
    HTS_PStreamSet_clear(&mut engine.pss);
    engine.sss = None;
    engine.label = None;
    engine.condition.stop = false;
}
