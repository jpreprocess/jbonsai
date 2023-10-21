use libc::FILE;

use crate::util::*;

extern "C" {
    fn atof(__nptr: *const libc::c_char) -> libc::c_double;
    fn atoi(__nptr: *const libc::c_char) -> libc::c_int;
    fn strstr(_: *const libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn exp(_: libc::c_double) -> libc::c_double;
    fn log(_: libc::c_double) -> libc::c_double;
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

#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_initialize(engine: *mut HTS_Engine) {
    (*engine).condition.sampling_frequency = 0 as libc::c_int as size_t;
    (*engine).condition.fperiod = 0 as libc::c_int as size_t;
    (*engine).condition.audio_buff_size = 0 as libc::c_int as size_t;
    (*engine).condition.stop = 0 as libc::c_int as HTS_Boolean;
    (*engine).condition.volume = 1.0f64;
    (*engine).condition.msd_threshold = std::ptr::null_mut::<libc::c_double>();
    (*engine).condition.gv_weight = std::ptr::null_mut::<libc::c_double>();
    (*engine).condition.speed = 1.0f64;
    (*engine).condition.phoneme_alignment_flag = 0 as libc::c_int as HTS_Boolean;
    (*engine).condition.stage = 0 as libc::c_int as size_t;
    (*engine).condition.use_log_gain = 0 as libc::c_int as HTS_Boolean;
    (*engine).condition.alpha = 0.0f64;
    (*engine).condition.beta = 0.0f64;
    (*engine).condition.additional_half_tone = 0.0f64;
    (*engine).condition.duration_iw = std::ptr::null_mut::<libc::c_double>();
    (*engine).condition.parameter_iw = std::ptr::null_mut::<*mut libc::c_double>();
    (*engine).condition.gv_iw = std::ptr::null_mut::<*mut libc::c_double>();
    // HTS_Audio_initialize(&mut (*engine).audio);
    HTS_ModelSet_initialize(&mut (*engine).ms);
    HTS_Label_initialize(&mut (*engine).label);
    HTS_SStreamSet_initialize(&mut (*engine).sss);
    HTS_PStreamSet_initialize(&mut (*engine).pss);
    HTS_GStreamSet_initialize(&mut (*engine).gss);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_load(
    engine: *mut HTS_Engine,
    voices: *mut *mut libc::c_char,
    num_voices: size_t,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut nstream: size_t = 0;
    let mut average_weight: libc::c_double = 0.;
    let mut option: *const libc::c_char = std::ptr::null::<libc::c_char>();
    let mut find: *const libc::c_char = std::ptr::null::<libc::c_char>();
    HTS_Engine_clear(engine);
    if HTS_ModelSet_load(&mut (*engine).ms, voices, num_voices) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_clear(engine);
        return 0 as libc::c_int as HTS_Boolean;
    }
    nstream = HTS_ModelSet_get_nstream(&mut (*engine).ms);
    average_weight = 1.0f64 / num_voices as libc::c_double;
    (*engine).condition.sampling_frequency = HTS_ModelSet_get_sampling_frequency(&mut (*engine).ms);
    (*engine).condition.fperiod = HTS_ModelSet_get_fperiod(&mut (*engine).ms);
    (*engine).condition.msd_threshold = HTS_calloc(
        nstream,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < nstream {
        *((*engine).condition.msd_threshold).offset(i as isize) = 0.5f64;
        i = i.wrapping_add(1);
        i;
    }
    (*engine).condition.gv_weight = HTS_calloc(
        nstream,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < nstream {
        *((*engine).condition.gv_weight).offset(i as isize) = 1.0f64;
        i = i.wrapping_add(1);
        i;
    }
    option = HTS_ModelSet_get_option(&mut (*engine).ms, 0 as libc::c_int as size_t);
    find = strstr(option, b"GAMMA=\0" as *const u8 as *const libc::c_char);
    if !find.is_null() {
        (*engine).condition.stage = atoi(&*find.offset((strlen
            as unsafe extern "C" fn(*const libc::c_char) -> libc::c_ulong)(
            b"GAMMA=\0" as *const u8 as *const libc::c_char,
        ) as isize)) as size_t;
    }
    find = strstr(option, b"LN_GAIN=\0" as *const u8 as *const libc::c_char);
    if !find.is_null() {
        (*engine).condition.use_log_gain = (if atoi(&*find.offset((strlen
            as unsafe extern "C" fn(*const libc::c_char) -> libc::c_ulong)(
            b"LN_GAIN=\0" as *const u8 as *const libc::c_char,
        ) as isize))
            == 1 as libc::c_int
        {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        }) as HTS_Boolean;
    }
    find = strstr(option, b"ALPHA=\0" as *const u8 as *const libc::c_char);
    if !find.is_null() {
        (*engine).condition.alpha = atof(&*find.offset((strlen
            as unsafe extern "C" fn(*const libc::c_char) -> libc::c_ulong)(
            b"ALPHA=\0" as *const u8 as *const libc::c_char,
        ) as isize));
    }
    (*engine).condition.duration_iw = HTS_calloc(
        num_voices,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < num_voices {
        *((*engine).condition.duration_iw).offset(i as isize) = average_weight;
        i = i.wrapping_add(1);
        i;
    }
    (*engine).condition.parameter_iw = HTS_calloc(
        num_voices,
        ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
    ) as *mut *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < num_voices {
        let fresh0 = &mut (*((*engine).condition.parameter_iw).offset(i as isize));
        *fresh0 = HTS_calloc(
            nstream,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < nstream {
            *(*((*engine).condition.parameter_iw).offset(i as isize)).offset(j as isize) =
                average_weight;
            j = j.wrapping_add(1);
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
    (*engine).condition.gv_iw = HTS_calloc(
        num_voices,
        ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
    ) as *mut *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < num_voices {
        let fresh1 = &mut (*((*engine).condition.gv_iw).offset(i as isize));
        *fresh1 = HTS_calloc(
            nstream,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < nstream {
            *(*((*engine).condition.gv_iw).offset(i as isize)).offset(j as isize) = average_weight;
            j = j.wrapping_add(1);
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_sampling_frequency(
    engine: *mut HTS_Engine,
    mut i: size_t,
) {
    if i < 1 as libc::c_int as size_t {
        i = 1 as libc::c_int as size_t;
    }
    (*engine).condition.sampling_frequency = i;
    // HTS_Audio_set_parameter(
    //     &mut (*engine).audio,
    //     (*engine).condition.sampling_frequency,
    //     (*engine).condition.audio_buff_size,
    // );
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_sampling_frequency(engine: *mut HTS_Engine) -> size_t {
    (*engine).condition.sampling_frequency
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_fperiod(engine: *mut HTS_Engine, mut i: size_t) {
    if i < 1 as libc::c_int as size_t {
        i = 1 as libc::c_int as size_t;
    }
    (*engine).condition.fperiod = i;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_fperiod(engine: *mut HTS_Engine) -> size_t {
    (*engine).condition.fperiod
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_audio_buff_size(
    engine: *mut HTS_Engine,
    i: size_t,
) {
    (*engine).condition.audio_buff_size = i;
    // HTS_Audio_set_parameter(
    //     &mut (*engine).audio,
    //     (*engine).condition.sampling_frequency,
    //     (*engine).condition.audio_buff_size,
    // );
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_audio_buff_size(engine: *mut HTS_Engine) -> size_t {
    (*engine).condition.audio_buff_size
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_stop_flag(engine: *mut HTS_Engine, b: HTS_Boolean) {
    (*engine).condition.stop = b;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_stop_flag(engine: *mut HTS_Engine) -> HTS_Boolean {
    (*engine).condition.stop
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_volume(engine: *mut HTS_Engine, f: libc::c_double) {
    (*engine).condition.volume = exp(f * DB);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_volume(engine: *mut HTS_Engine) -> libc::c_double {
    log((*engine).condition.volume) / DB
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_msd_threshold(
    engine: *mut HTS_Engine,
    stream_index: size_t,
    mut f: libc::c_double,
) {
    if f < 0.0f64 {
        f = 0.0f64;
    }
    if f > 1.0f64 {
        f = 1.0f64;
    }
    *((*engine).condition.msd_threshold).offset(stream_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_msd_threshold(
    engine: *mut HTS_Engine,
    stream_index: size_t,
) -> libc::c_double {
    *((*engine).condition.msd_threshold).offset(stream_index as isize)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_gv_weight(
    engine: *mut HTS_Engine,
    stream_index: size_t,
    mut f: libc::c_double,
) {
    if f < 0.0f64 {
        f = 0.0f64;
    }
    *((*engine).condition.gv_weight).offset(stream_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_gv_weight(
    engine: *mut HTS_Engine,
    stream_index: size_t,
) -> libc::c_double {
    *((*engine).condition.gv_weight).offset(stream_index as isize)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_speed(engine: *mut HTS_Engine, mut f: libc::c_double) {
    if f < 1.0E-06f64 {
        f = 1.0E-06f64;
    }
    (*engine).condition.speed = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_phoneme_alignment_flag(
    engine: *mut HTS_Engine,
    b: HTS_Boolean,
) {
    (*engine).condition.phoneme_alignment_flag = b;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_alpha(engine: *mut HTS_Engine, mut f: libc::c_double) {
    if f < 0.0f64 {
        f = 0.0f64;
    }
    if f > 1.0f64 {
        f = 1.0f64;
    }
    (*engine).condition.alpha = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_alpha(engine: *mut HTS_Engine) -> libc::c_double {
    (*engine).condition.alpha
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_beta(engine: *mut HTS_Engine, mut f: libc::c_double) {
    if f < 0.0f64 {
        f = 0.0f64;
    }
    if f > 1.0f64 {
        f = 1.0f64;
    }
    (*engine).condition.beta = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_beta(engine: *mut HTS_Engine) -> libc::c_double {
    (*engine).condition.beta
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_add_half_tone(
    engine: *mut HTS_Engine,
    f: libc::c_double,
) {
    (*engine).condition.additional_half_tone = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_duration_interpolation_weight(
    engine: *mut HTS_Engine,
    voice_index: size_t,
    f: libc::c_double,
) {
    *((*engine).condition.duration_iw).offset(voice_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_duration_interpolation_weight(
    engine: *mut HTS_Engine,
    voice_index: size_t,
) -> libc::c_double {
    *((*engine).condition.duration_iw).offset(voice_index as isize)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_parameter_interpolation_weight(
    engine: *mut HTS_Engine,
    voice_index: size_t,
    stream_index: size_t,
    f: libc::c_double,
) {
    *(*((*engine).condition.parameter_iw).offset(voice_index as isize))
        .offset(stream_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_parameter_interpolation_weight(
    engine: *mut HTS_Engine,
    voice_index: size_t,
    stream_index: size_t,
) -> libc::c_double {
    *(*((*engine).condition.parameter_iw).offset(voice_index as isize))
        .offset(stream_index as isize)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_gv_interpolation_weight(
    engine: *mut HTS_Engine,
    voice_index: size_t,
    stream_index: size_t,
    f: libc::c_double,
) {
    *(*((*engine).condition.gv_iw).offset(voice_index as isize)).offset(stream_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_gv_interpolation_weight(
    engine: *mut HTS_Engine,
    voice_index: size_t,
    stream_index: size_t,
) -> libc::c_double {
    *(*((*engine).condition.gv_iw).offset(voice_index as isize))
        .offset(stream_index as isize)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_total_state(engine: *mut HTS_Engine) -> size_t {
    HTS_SStreamSet_get_total_state(&mut (*engine).sss)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_set_state_mean(
    engine: *mut HTS_Engine,
    stream_index: size_t,
    state_index: size_t,
    vector_index: size_t,
    f: libc::c_double,
) {
    HTS_SStreamSet_set_mean(
        &mut (*engine).sss,
        stream_index,
        state_index,
        vector_index,
        f,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_state_mean(
    engine: *mut HTS_Engine,
    stream_index: size_t,
    state_index: size_t,
    vector_index: size_t,
) -> libc::c_double {
    HTS_SStreamSet_get_mean(&mut (*engine).sss, stream_index, state_index, vector_index)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_state_duration(
    engine: *mut HTS_Engine,
    state_index: size_t,
) -> size_t {
    HTS_SStreamSet_get_duration(&mut (*engine).sss, state_index)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_nvoices(engine: *mut HTS_Engine) -> size_t {
    HTS_ModelSet_get_nvoices(&mut (*engine).ms)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_nstream(engine: *mut HTS_Engine) -> size_t {
    HTS_ModelSet_get_nstream(&mut (*engine).ms)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_nstate(engine: *mut HTS_Engine) -> size_t {
    HTS_ModelSet_get_nstate(&mut (*engine).ms)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_fullcontext_label_format(
    engine: *mut HTS_Engine,
) -> *const libc::c_char {
    HTS_ModelSet_get_fullcontext_label_format(&mut (*engine).ms)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_fullcontext_label_version(
    engine: *mut HTS_Engine,
) -> *const libc::c_char {
    HTS_ModelSet_get_fullcontext_label_version(&mut (*engine).ms)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_total_frame(engine: *mut HTS_Engine) -> size_t {
    HTS_GStreamSet_get_total_frame(&mut (*engine).gss)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_nsamples(engine: *mut HTS_Engine) -> size_t {
    HTS_GStreamSet_get_total_nsamples(&mut (*engine).gss)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_generated_parameter(
    engine: *mut HTS_Engine,
    stream_index: size_t,
    frame_index: size_t,
    vector_index: size_t,
) -> libc::c_double {
    HTS_GStreamSet_get_parameter(
        &mut (*engine).gss,
        stream_index,
        frame_index,
        vector_index,
    )
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_get_generated_speech(
    engine: *mut HTS_Engine,
    index: size_t,
) -> libc::c_double {
    HTS_GStreamSet_get_speech(&mut (*engine).gss, index)
}
unsafe extern "C" fn HTS_Engine_generate_state_sequence(
    engine: *mut HTS_Engine,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut state_index: size_t = 0;
    let mut model_index: size_t = 0;
    let mut f: libc::c_double = 0.;
    if HTS_SStreamSet_create(
        &mut (*engine).sss,
        &mut (*engine).ms,
        &mut (*engine).label,
        (*engine).condition.phoneme_alignment_flag,
        (*engine).condition.speed,
        (*engine).condition.duration_iw,
        (*engine).condition.parameter_iw,
        (*engine).condition.gv_iw,
    ) as libc::c_int
        != 1 as libc::c_int
    {
        HTS_Engine_refresh(engine);
        return 0 as libc::c_int as HTS_Boolean;
    }
    if (*engine).condition.additional_half_tone != 0.0f64 {
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
            f += (*engine).condition.additional_half_tone * HALF_TONE;
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
            state_index;
            if state_index >= HTS_Engine_get_nstate(engine) {
                state_index = 0 as libc::c_int as size_t;
                model_index = model_index.wrapping_add(1);
                model_index;
            }
            i = i.wrapping_add(1);
            i;
        }
    }
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_generate_state_sequence_from_fn(
    engine: *mut HTS_Engine,
    fn_0: *const libc::c_char,
) -> HTS_Boolean {
    HTS_Engine_refresh(engine);
    HTS_Label_load_from_fn(
        &mut (*engine).label,
        (*engine).condition.sampling_frequency,
        (*engine).condition.fperiod,
        fn_0,
    );
    HTS_Engine_generate_state_sequence(engine)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_generate_state_sequence_from_strings(
    engine: *mut HTS_Engine,
    lines: *mut *mut libc::c_char,
    num_lines: size_t,
) -> HTS_Boolean {
    HTS_Engine_refresh(engine);
    HTS_Label_load_from_strings(
        &mut (*engine).label,
        (*engine).condition.sampling_frequency,
        (*engine).condition.fperiod,
        lines,
        num_lines,
    );
    HTS_Engine_generate_state_sequence(engine)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_generate_parameter_sequence(
    engine: *mut HTS_Engine,
) -> HTS_Boolean {
    HTS_PStreamSet_create(
        &mut (*engine).pss,
        &mut (*engine).sss,
        (*engine).condition.msd_threshold,
        (*engine).condition.gv_weight,
    )
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_generate_sample_sequence(
    engine: *mut HTS_Engine,
) -> HTS_Boolean {
    HTS_GStreamSet_create(
        &mut (*engine).gss,
        &mut (*engine).pss,
        (*engine).condition.stage,
        (*engine).condition.use_log_gain,
        (*engine).condition.sampling_frequency,
        (*engine).condition.fperiod,
        (*engine).condition.alpha,
        (*engine).condition.beta,
        &mut (*engine).condition.stop,
        (*engine).condition.volume,
    )
}
unsafe extern "C" fn HTS_Engine_synthesize(engine: *mut HTS_Engine) -> HTS_Boolean {
    if HTS_Engine_generate_state_sequence(engine) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_refresh(engine);
        return 0 as libc::c_int as HTS_Boolean;
    }
    if HTS_Engine_generate_parameter_sequence(engine) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_refresh(engine);
        return 0 as libc::c_int as HTS_Boolean;
    }
    if HTS_Engine_generate_sample_sequence(engine) as libc::c_int != 1 as libc::c_int {
        HTS_Engine_refresh(engine);
        return 0 as libc::c_int as HTS_Boolean;
    }
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_synthesize_from_fn(
    engine: *mut HTS_Engine,
    fn_0: *const libc::c_char,
) -> HTS_Boolean {
    HTS_Engine_refresh(engine);
    HTS_Label_load_from_fn(
        &mut (*engine).label,
        (*engine).condition.sampling_frequency,
        (*engine).condition.fperiod,
        fn_0,
    );
    HTS_Engine_synthesize(engine)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_synthesize_from_strings(
    engine: *mut HTS_Engine,
    lines: *mut *mut libc::c_char,
    num_lines: size_t,
) -> HTS_Boolean {
    HTS_Engine_refresh(engine);
    HTS_Label_load_from_strings(
        &mut (*engine).label,
        (*engine).condition.sampling_frequency,
        (*engine).condition.fperiod,
        lines,
        num_lines,
    );
    HTS_Engine_synthesize(engine)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_save_information(
    engine: *mut HTS_Engine,
    fp: *mut FILE,
) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut l: size_t = 0;
    let mut m: size_t = 0;
    let mut n: size_t = 0;
    let mut temp: libc::c_double = 0.;
    let condition: *mut HTS_Condition = &mut (*engine).condition;
    let ms: *mut HTS_ModelSet = &mut (*engine).ms;
    let label: *mut HTS_Label = &mut (*engine).label;
    let sss: *mut HTS_SStreamSet = &mut (*engine).sss;
    let pss: *mut HTS_PStreamSet = &mut (*engine).pss;
    fprintf(
        fp,
        b"[Global parameter]\n\0" as *const u8 as *const libc::c_char,
    );
    fprintf(
        fp,
        b"Sampring frequency                     -> %8lu(Hz)\n\0" as *const u8
            as *const libc::c_char,
        (*condition).sampling_frequency,
    );
    fprintf(
        fp,
        b"Frame period                           -> %8lu(point)\n\0" as *const u8
            as *const libc::c_char,
        (*condition).fperiod,
    );
    fprintf(
        fp,
        b"                                          %8.5f(msec)\n\0" as *const u8
            as *const libc::c_char,
        1e+3f64 * (*condition).fperiod as libc::c_double
            / (*condition).sampling_frequency as libc::c_double,
    );
    fprintf(
        fp,
        b"All-pass constant                      -> %8.5f\n\0" as *const u8 as *const libc::c_char,
        (*condition).alpha as libc::c_float as libc::c_double,
    );
    fprintf(
        fp,
        b"Gamma                                  -> %8.5f\n\0" as *const u8 as *const libc::c_char,
        (if (*condition).stage == 0 as libc::c_int as size_t {
            0.0f64
        } else {
            -1.0f64 / (*condition).stage as libc::c_double
        }) as libc::c_float as libc::c_double,
    );
    if (*condition).stage != 0 as libc::c_int as size_t {
        if (*condition).use_log_gain as libc::c_int == 1 as libc::c_int {
            fprintf(
                fp,
                b"Log gain flag                          ->     TRUE\n\0" as *const u8
                    as *const libc::c_char,
            );
        } else {
            fprintf(
                fp,
                b"Log gain flag                          ->    FALSE\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
    }
    fprintf(
        fp,
        b"Postfiltering coefficient              -> %8.5f\n\0" as *const u8 as *const libc::c_char,
        (*condition).beta as libc::c_float as libc::c_double,
    );
    fprintf(
        fp,
        b"Audio buffer size                      -> %8lu(sample)\n\0" as *const u8
            as *const libc::c_char,
        (*condition).audio_buff_size,
    );
    fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
    fprintf(
        fp,
        b"[Duration parameter]\n\0" as *const u8 as *const libc::c_char,
    );
    fprintf(
        fp,
        b"Number of states                       -> %8lu\n\0" as *const u8 as *const libc::c_char,
        HTS_ModelSet_get_nstate(ms),
    );
    fprintf(
        fp,
        b"         Interpolation size            -> %8lu\n\0" as *const u8 as *const libc::c_char,
        HTS_ModelSet_get_nvoices(ms),
    );
    i = 0 as libc::c_int as size_t;
    temp = 0.0f64;
    while i < HTS_ModelSet_get_nvoices(ms) {
        temp += *((*condition).duration_iw).offset(i as isize);
        i = i.wrapping_add(1);
        i;
    }
    i = 0 as libc::c_int as size_t;
    while i < HTS_ModelSet_get_nvoices(ms) {
        if *((*condition).duration_iw).offset(i as isize) != 0.0f64 {
            *((*condition).duration_iw).offset(i as isize) /= temp;
        }
        i = i.wrapping_add(1);
        i;
    }
    i = 0 as libc::c_int as size_t;
    while i < HTS_ModelSet_get_nvoices(ms) {
        fprintf(
            fp,
            b"         Interpolation weight[%2lu]      -> %8.0f(%%)\n\0" as *const u8
                as *const libc::c_char,
            i,
            (100 as libc::c_int as libc::c_double * *((*condition).duration_iw).offset(i as isize))
                as libc::c_float as libc::c_double,
        );
        i = i.wrapping_add(1);
        i;
    }
    fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
    fprintf(
        fp,
        b"[Stream parameter]\n\0" as *const u8 as *const libc::c_char,
    );
    i = 0 as libc::c_int as size_t;
    while i < HTS_ModelSet_get_nstream(ms) {
        fprintf(
            fp,
            b"Stream[%2lu] vector length               -> %8lu\n\0" as *const u8
                as *const libc::c_char,
            i,
            HTS_ModelSet_get_vector_length(ms, i),
        );
        fprintf(
            fp,
            b"           Dynamic window size         -> %8lu\n\0" as *const u8
                as *const libc::c_char,
            HTS_ModelSet_get_window_size(ms, i),
        );
        fprintf(
            fp,
            b"           Interpolation size          -> %8lu\n\0" as *const u8
                as *const libc::c_char,
            HTS_ModelSet_get_nvoices(ms),
        );
        j = 0 as libc::c_int as size_t;
        temp = 0.0f64;
        while j < HTS_ModelSet_get_nvoices(ms) {
            temp += *(*((*condition).parameter_iw).offset(j as isize)).offset(i as isize);
            j = j.wrapping_add(1);
            j;
        }
        j = 0 as libc::c_int as size_t;
        while j < HTS_ModelSet_get_nvoices(ms) {
            if *(*((*condition).parameter_iw).offset(j as isize)).offset(i as isize) != 0.0f64 {
                *(*((*condition).parameter_iw).offset(j as isize)).offset(i as isize) /= temp;
            }
            j = j.wrapping_add(1);
            j;
        }
        j = 0 as libc::c_int as size_t;
        while j < HTS_ModelSet_get_nvoices(ms) {
            fprintf(
                fp,
                b"           Interpolation weight[%2lu]    -> %8.0f(%%)\n\0" as *const u8
                    as *const libc::c_char,
                j,
                (100 as libc::c_int as libc::c_double
                    * *(*((*condition).parameter_iw).offset(j as isize)).offset(i as isize))
                    as libc::c_float as libc::c_double,
            );
            j = j.wrapping_add(1);
            j;
        }
        if HTS_ModelSet_is_msd(ms, i) != 0 {
            fprintf(
                fp,
                b"           MSD flag                    ->     TRUE\n\0" as *const u8
                    as *const libc::c_char,
            );
            fprintf(
                fp,
                b"           MSD threshold               -> %8.5f\n\0" as *const u8
                    as *const libc::c_char,
                *((*condition).msd_threshold).offset(i as isize),
            );
        } else {
            fprintf(
                fp,
                b"           MSD flag                    ->    FALSE\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            fprintf(
                fp,
                b"           GV flag                     ->     TRUE\n\0" as *const u8
                    as *const libc::c_char,
            );
            fprintf(
                fp,
                b"           GV weight                   -> %8.0f(%%)\n\0" as *const u8
                    as *const libc::c_char,
                (100 as libc::c_int as libc::c_double
                    * *((*condition).gv_weight).offset(i as isize)) as libc::c_float
                    as libc::c_double,
            );
            fprintf(
                fp,
                b"           GV interpolation size       -> %8lu\n\0" as *const u8
                    as *const libc::c_char,
                HTS_ModelSet_get_nvoices(ms),
            );
            j = 0 as libc::c_int as size_t;
            temp = 0.0f64;
            while j < HTS_ModelSet_get_nvoices(ms) {
                temp += *(*((*condition).gv_iw).offset(j as isize)).offset(i as isize);
                j = j.wrapping_add(1);
                j;
            }
            j = 0 as libc::c_int as size_t;
            while j < HTS_ModelSet_get_nvoices(ms) {
                if *(*((*condition).gv_iw).offset(j as isize)).offset(i as isize) != 0.0f64 {
                    *(*((*condition).gv_iw).offset(j as isize)).offset(i as isize) /= temp;
                }
                j = j.wrapping_add(1);
                j;
            }
            j = 0 as libc::c_int as size_t;
            while j < HTS_ModelSet_get_nvoices(ms) {
                fprintf(
                    fp,
                    b"           GV interpolation weight[%2lu] -> %8.0f(%%)\n\0" as *const u8
                        as *const libc::c_char,
                    j,
                    (100 as libc::c_int as libc::c_double
                        * *(*((*condition).gv_iw).offset(j as isize)).offset(i as isize))
                        as libc::c_float as libc::c_double,
                );
                j = j.wrapping_add(1);
                j;
            }
        } else {
            fprintf(
                fp,
                b"           GV flag                     ->    FALSE\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
        i = i.wrapping_add(1);
        i;
    }
    fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
    fprintf(
        fp,
        b"[Generated sequence]\n\0" as *const u8 as *const libc::c_char,
    );
    fprintf(
        fp,
        b"Number of HMMs                         -> %8lu\n\0" as *const u8 as *const libc::c_char,
        HTS_Label_get_size(label),
    );
    fprintf(
        fp,
        b"Number of stats                        -> %8lu\n\0" as *const u8 as *const libc::c_char,
        (HTS_Label_get_size(label)).wrapping_mul(HTS_ModelSet_get_nstate(ms)),
    );
    fprintf(
        fp,
        b"Length of this speech                  -> %8.3f(sec)\n\0" as *const u8
            as *const libc::c_char,
        (HTS_PStreamSet_get_total_frame(pss) as libc::c_double
            * (*condition).fperiod as libc::c_double
            / (*condition).sampling_frequency as libc::c_double) as libc::c_float
            as libc::c_double,
    );
    fprintf(
        fp,
        b"                                       -> %8lu(frames)\n\0" as *const u8
            as *const libc::c_char,
        (HTS_PStreamSet_get_total_frame(pss)).wrapping_mul((*condition).fperiod),
    );
    i = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        fprintf(fp, b"HMM[%2lu]\n\0" as *const u8 as *const libc::c_char, i);
        fprintf(
            fp,
            b"  Name                                 -> %s\n\0" as *const u8 as *const libc::c_char,
            HTS_Label_get_string(label, i),
        );
        fprintf(fp, b"  Duration\n\0" as *const u8 as *const libc::c_char);
        j = 0 as libc::c_int as size_t;
        while j < HTS_ModelSet_get_nvoices(ms) {
            fprintf(
                fp,
                b"    Interpolation[%2lu]\n\0" as *const u8 as *const libc::c_char,
                j,
            );
            HTS_ModelSet_get_duration_index(ms, j, HTS_Label_get_string(label, i), &mut k, &mut l);
            fprintf(
                fp,
                b"      Tree index                       -> %8lu\n\0" as *const u8
                    as *const libc::c_char,
                k,
            );
            fprintf(
                fp,
                b"      PDF index                        -> %8lu\n\0" as *const u8
                    as *const libc::c_char,
                l,
            );
            j = j.wrapping_add(1);
            j;
        }
        j = 0 as libc::c_int as size_t;
        while j < HTS_ModelSet_get_nstate(ms) {
            fprintf(
                fp,
                b"  State[%2lu]\n\0" as *const u8 as *const libc::c_char,
                j.wrapping_add(2 as libc::c_int as libc::c_ulong),
            );
            fprintf(
                fp,
                b"    Length                             -> %8lu(frames)\n\0" as *const u8
                    as *const libc::c_char,
                HTS_SStreamSet_get_duration(sss, (i * HTS_ModelSet_get_nstate(ms)).wrapping_add(j)),
            );
            k = 0 as libc::c_int as size_t;
            while k < HTS_ModelSet_get_nstream(ms) {
                fprintf(
                    fp,
                    b"    Stream[%2lu]\n\0" as *const u8 as *const libc::c_char,
                    k,
                );
                if HTS_ModelSet_is_msd(ms, k) != 0 {
                    if HTS_SStreamSet_get_msd(
                        sss,
                        k,
                        (i * HTS_ModelSet_get_nstate(ms)).wrapping_add(j),
                    ) > *((*condition).msd_threshold).offset(k as isize)
                    {
                        fprintf(
                            fp,
                            b"      MSD flag                         ->     TRUE\n\0" as *const u8
                                as *const libc::c_char,
                        );
                    } else {
                        fprintf(
                            fp,
                            b"      MSD flag                         ->    FALSE\n\0" as *const u8
                                as *const libc::c_char,
                        );
                    }
                }
                l = 0 as libc::c_int as size_t;
                while l < HTS_ModelSet_get_nvoices(ms) {
                    fprintf(
                        fp,
                        b"      Interpolation[%2lu]\n\0" as *const u8 as *const libc::c_char,
                        l,
                    );
                    HTS_ModelSet_get_parameter_index(
                        ms,
                        l,
                        k,
                        j.wrapping_add(2 as libc::c_int as size_t),
                        HTS_Label_get_string(label, i),
                        &mut m,
                        &mut n,
                    );
                    fprintf(
                        fp,
                        b"        Tree index                     -> %8lu\n\0" as *const u8
                            as *const libc::c_char,
                        m,
                    );
                    fprintf(
                        fp,
                        b"        PDF index                      -> %8lu\n\0" as *const u8
                            as *const libc::c_char,
                        n,
                    );
                    l = l.wrapping_add(1);
                    l;
                }
                k = k.wrapping_add(1);
                k;
            }
            j = j.wrapping_add(1);
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_save_label(engine: *mut HTS_Engine, fp: *mut FILE) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut frame: size_t = 0;
    let mut state: size_t = 0;
    let mut duration: size_t = 0;
    let label: *mut HTS_Label = &mut (*engine).label;
    let sss: *mut HTS_SStreamSet = &mut (*engine).sss;
    let nstate: size_t = HTS_ModelSet_get_nstate(&mut (*engine).ms);
    let rate: libc::c_double = (*engine).condition.fperiod as libc::c_double * 1.0e+07f64
        / (*engine).condition.sampling_frequency as libc::c_double;
    i = 0 as libc::c_int as size_t;
    state = 0 as libc::c_int as size_t;
    frame = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        j = 0 as libc::c_int as size_t;
        duration = 0 as libc::c_int as size_t;
        while j < nstate {
            let fresh2 = state;
            state = state.wrapping_add(1);
            duration = duration.wrapping_add(HTS_SStreamSet_get_duration(sss, fresh2));
            j = j.wrapping_add(1);
            j;
        }
        fprintf(
            fp,
            b"%lu %lu %s\n\0" as *const u8 as *const libc::c_char,
            (frame as libc::c_double * rate) as libc::c_ulong,
            (frame.wrapping_add(duration) as libc::c_double * rate) as libc::c_ulong,
            HTS_Label_get_string(label, i),
        );
        frame = frame.wrapping_add(duration);
        i = i.wrapping_add(1);
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_save_generated_parameter(
    engine: *mut HTS_Engine,
    stream_index: size_t,
    fp: *mut FILE,
) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut temp: libc::c_float = 0.;
    let gss: *mut HTS_GStreamSet = &mut (*engine).gss;
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
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_save_generated_speech(
    engine: *mut HTS_Engine,
    fp: *mut FILE,
) {
    let mut i: size_t = 0;
    let mut x: libc::c_double = 0.;
    let mut temp: libc::c_short = 0;
    let gss: *mut HTS_GStreamSet = &mut (*engine).gss;
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
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_save_riff(engine: *mut HTS_Engine, fp: *mut FILE) {
    let mut i: size_t = 0;
    let mut x: libc::c_double = 0.;
    let mut temp: libc::c_short = 0;
    let gss: *mut HTS_GStreamSet = &mut (*engine).gss;
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
    let mut data_25_28: libc::c_int = (*engine).condition.sampling_frequency as libc::c_int;
    let mut data_29_32: libc::c_int = ((*engine).condition.sampling_frequency)
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
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_refresh(engine: *mut HTS_Engine) {
    HTS_GStreamSet_clear(&mut (*engine).gss);
    HTS_PStreamSet_clear(&mut (*engine).pss);
    HTS_SStreamSet_clear(&mut (*engine).sss);
    HTS_Label_clear(&mut (*engine).label);
    (*engine).condition.stop = 0 as libc::c_int as HTS_Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Engine_clear(engine: *mut HTS_Engine) {
    let mut i: size_t = 0;
    if !((*engine).condition.msd_threshold).is_null() {
        HTS_free((*engine).condition.msd_threshold as *mut libc::c_void);
    }
    if !((*engine).condition.duration_iw).is_null() {
        HTS_free((*engine).condition.duration_iw as *mut libc::c_void);
    }
    if !((*engine).condition.gv_weight).is_null() {
        HTS_free((*engine).condition.gv_weight as *mut libc::c_void);
    }
    if !((*engine).condition.parameter_iw).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < HTS_ModelSet_get_nvoices(&mut (*engine).ms) {
            HTS_free(*((*engine).condition.parameter_iw).offset(i as isize) as *mut libc::c_void);
            i = i.wrapping_add(1);
            i;
        }
        HTS_free((*engine).condition.parameter_iw as *mut libc::c_void);
    }
    if !((*engine).condition.gv_iw).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < HTS_ModelSet_get_nvoices(&mut (*engine).ms) {
            HTS_free(*((*engine).condition.gv_iw).offset(i as isize) as *mut libc::c_void);
            i = i.wrapping_add(1);
            i;
        }
        HTS_free((*engine).condition.gv_iw as *mut libc::c_void);
    }
    HTS_ModelSet_clear(&mut (*engine).ms);
    // HTS_Audio_clear(&mut (*engine).audio);
    HTS_Engine_initialize(engine);
}
