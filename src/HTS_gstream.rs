#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use crate::util::*;

extern "C" {
    fn HTS_free(p: *mut libc::c_void);
    fn HTS_error(error: libc::c_int, message: *const libc::c_char, _: ...);
    fn HTS_Audio_flush(audio: *mut HTS_Audio);
    fn HTS_calloc(num: size_t, size: size_t) -> *mut libc::c_void;
    fn HTS_PStreamSet_get_nstream(pss: *mut HTS_PStreamSet) -> size_t;
    fn HTS_PStreamSet_get_vector_length(
        pss: *mut HTS_PStreamSet,
        stream_index: size_t,
    ) -> size_t;
    fn HTS_PStreamSet_get_total_frame(pss: *mut HTS_PStreamSet) -> size_t;
    fn HTS_PStreamSet_get_parameter(
        pss: *mut HTS_PStreamSet,
        stream_index: size_t,
        frame_index: size_t,
        vector_index: size_t,
    ) -> libc::c_double;
    fn HTS_PStreamSet_get_msd_flag(
        pss: *mut HTS_PStreamSet,
        stream_index: size_t,
        frame_index: size_t,
    ) -> HTS_Boolean;
    fn HTS_PStreamSet_is_msd(
        pss: *mut HTS_PStreamSet,
        stream_index: size_t,
    ) -> HTS_Boolean;
    fn HTS_Vocoder_initialize(
        v: *mut HTS_Vocoder,
        m: size_t,
        stage: size_t,
        use_log_gain: HTS_Boolean,
        rate: size_t,
        fperiod: size_t,
    );
    fn HTS_Vocoder_synthesize(
        v: *mut HTS_Vocoder,
        m: size_t,
        lf0: libc::c_double,
        spectrum: *mut libc::c_double,
        nlpf: size_t,
        lpf: *mut libc::c_double,
        alpha: libc::c_double,
        beta: libc::c_double,
        volume: libc::c_double,
        rawdata: *mut libc::c_double,
        audio: *mut HTS_Audio,
    );
    fn HTS_Vocoder_clear(v: *mut HTS_Vocoder);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_initialize(mut gss: *mut HTS_GStreamSet) {
    (*gss).nstream = 0 as libc::c_int as size_t;
    (*gss).total_frame = 0 as libc::c_int as size_t;
    (*gss).total_nsample = 0 as libc::c_int as size_t;
    (*gss).gstream = 0 as *mut HTS_GStream;
    (*gss).gspeech = 0 as *mut libc::c_double;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_create(
    mut gss: *mut HTS_GStreamSet,
    mut pss: *mut HTS_PStreamSet,
    mut stage: size_t,
    mut use_log_gain: HTS_Boolean,
    mut sampling_rate: size_t,
    mut fperiod: size_t,
    mut alpha: libc::c_double,
    mut beta: libc::c_double,
    mut stop: *mut HTS_Boolean,
    mut volume: libc::c_double,
    mut audio: *mut HTS_Audio,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut msd_frame: size_t = 0;
    let mut v: HTS_Vocoder = HTS_Vocoder {
        is_first: 0,
        stage: 0,
        gamma: 0.,
        use_log_gain: 0,
        fprd: 0,
        next: 0,
        gauss: 0,
        rate: 0.,
        pitch_of_curr_point: 0.,
        pitch_counter: 0.,
        pitch_inc_per_point: 0.,
        excite_ring_buff: 0 as *mut libc::c_double,
        excite_buff_size: 0,
        excite_buff_index: 0,
        sw: 0,
        x: 0,
        freqt_buff: 0 as *mut libc::c_double,
        freqt_size: 0,
        spectrum2en_buff: 0 as *mut libc::c_double,
        spectrum2en_size: 0,
        r1: 0.,
        r2: 0.,
        s: 0.,
        postfilter_buff: 0 as *mut libc::c_double,
        postfilter_size: 0,
        c: 0 as *mut libc::c_double,
        cc: 0 as *mut libc::c_double,
        cinc: 0 as *mut libc::c_double,
        d1: 0 as *mut libc::c_double,
        lsp2lpc_buff: 0 as *mut libc::c_double,
        lsp2lpc_size: 0,
        gc2gc_buff: 0 as *mut libc::c_double,
        gc2gc_size: 0,
    };
    let mut nlpf: size_t = 0 as libc::c_int as size_t;
    let mut lpf: *mut libc::c_double = 0 as *mut libc::c_double;
    if !((*gss).gstream).is_null() || !((*gss).gspeech).is_null() {
        HTS_error(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: HTS_GStreamSet is not initialized.\n\0" as *const u8
                as *const libc::c_char,
        );
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*gss).nstream = HTS_PStreamSet_get_nstream(pss);
    (*gss).total_frame = HTS_PStreamSet_get_total_frame(pss);
    (*gss).total_nsample = fperiod * (*gss).total_frame;
    (*gss)
        .gstream = HTS_calloc(
        (*gss).nstream,
        ::core::mem::size_of::<HTS_GStream>() as libc::c_ulong,
    ) as *mut HTS_GStream;
    i = 0 as libc::c_int as size_t;
    while i < (*gss).nstream {
        (*((*gss).gstream).offset(i as isize))
            .vector_length = HTS_PStreamSet_get_vector_length(pss, i);
        let ref mut fresh0 = (*((*gss).gstream).offset(i as isize)).par;
        *fresh0 = HTS_calloc(
            (*gss).total_frame,
            ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < (*gss).total_frame {
            let ref mut fresh1 = *((*((*gss).gstream).offset(i as isize)).par)
                .offset(j as isize);
            *fresh1 = HTS_calloc(
                (*((*gss).gstream).offset(i as isize)).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            j = j.wrapping_add(1);
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
    (*gss)
        .gspeech = HTS_calloc(
        (*gss).total_nsample,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < (*gss).nstream {
        if HTS_PStreamSet_is_msd(pss, i) != 0 {
            j = 0 as libc::c_int as size_t;
            msd_frame = 0 as libc::c_int as size_t;
            while j < (*gss).total_frame {
                if HTS_PStreamSet_get_msd_flag(pss, i, j) != 0 {
                    k = 0 as libc::c_int as size_t;
                    while k < (*((*gss).gstream).offset(i as isize)).vector_length {
                        *(*((*((*gss).gstream).offset(i as isize)).par)
                            .offset(j as isize))
                            .offset(
                                k as isize,
                            ) = HTS_PStreamSet_get_parameter(pss, i, msd_frame, k);
                        k = k.wrapping_add(1);
                        k;
                    }
                    msd_frame = msd_frame.wrapping_add(1);
                    msd_frame;
                } else {
                    k = 0 as libc::c_int as size_t;
                    while k < (*((*gss).gstream).offset(i as isize)).vector_length {
                        *(*((*((*gss).gstream).offset(i as isize)).par)
                            .offset(j as isize))
                            .offset(k as isize) = -1.0e+10f64;
                        k = k.wrapping_add(1);
                        k;
                    }
                }
                j = j.wrapping_add(1);
                j;
            }
        } else {
            j = 0 as libc::c_int as size_t;
            while j < (*gss).total_frame {
                k = 0 as libc::c_int as size_t;
                while k < (*((*gss).gstream).offset(i as isize)).vector_length {
                    *(*((*((*gss).gstream).offset(i as isize)).par).offset(j as isize))
                        .offset(k as isize) = HTS_PStreamSet_get_parameter(pss, i, j, k);
                    k = k.wrapping_add(1);
                    k;
                }
                j = j.wrapping_add(1);
                j;
            }
        }
        i = i.wrapping_add(1);
        i;
    }
    if (*gss).nstream != 2 as libc::c_int as size_t
        && (*gss).nstream != 3 as libc::c_int as size_t
    {
        HTS_error(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: The number of streams should be 2 or 3.\n\0"
                as *const u8 as *const libc::c_char,
        );
        HTS_GStreamSet_clear(gss);
        return 0 as libc::c_int as HTS_Boolean;
    }
    if HTS_PStreamSet_get_vector_length(pss, 1 as libc::c_int as size_t)
        != 1 as libc::c_int as size_t
    {
        HTS_error(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: The size of lf0 static vector should be 1.\n\0"
                as *const u8 as *const libc::c_char,
        );
        HTS_GStreamSet_clear(gss);
        return 0 as libc::c_int as HTS_Boolean;
    }
    if (*gss).nstream >= 3 as libc::c_int as size_t
        && (*((*gss).gstream).offset(2 as libc::c_int as isize)).vector_length
            % 2 as libc::c_int as size_t == 0 as libc::c_int as size_t
    {
        HTS_error(
            1 as libc::c_int,
            b"HTS_GStreamSet_create: The number of low-pass filter coefficient should be odd numbers.\0"
                as *const u8 as *const libc::c_char,
        );
        HTS_GStreamSet_clear(gss);
        return 0 as libc::c_int as HTS_Boolean;
    }
    HTS_Vocoder_initialize(
        &mut v,
        ((*((*gss).gstream).offset(0 as libc::c_int as isize)).vector_length)
            .wrapping_sub(1 as libc::c_int as size_t),
        stage,
        use_log_gain,
        sampling_rate,
        fperiod,
    );
    if (*gss).nstream >= 3 as libc::c_int as size_t {
        nlpf = (*((*gss).gstream).offset(2 as libc::c_int as isize)).vector_length;
    }
    i = 0 as libc::c_int as size_t;
    while i < (*gss).total_frame && *stop as libc::c_int == 0 as libc::c_int {
        j = i * fperiod;
        if (*gss).nstream >= 3 as libc::c_int as size_t {
            lpf = &mut *(*((*((*gss).gstream).offset(2 as libc::c_int as isize)).par)
                .offset(i as isize))
                .offset(0 as libc::c_int as isize) as *mut libc::c_double;
        }
        HTS_Vocoder_synthesize(
            &mut v,
            ((*((*gss).gstream).offset(0 as libc::c_int as isize)).vector_length)
                .wrapping_sub(1 as libc::c_int as size_t),
            *(*((*((*gss).gstream).offset(1 as libc::c_int as isize)).par)
                .offset(i as isize))
                .offset(0 as libc::c_int as isize),
            &mut *(*((*((*gss).gstream).offset(0 as libc::c_int as isize)).par)
                .offset(i as isize))
                .offset(0 as libc::c_int as isize),
            nlpf,
            lpf,
            alpha,
            beta,
            volume,
            &mut *((*gss).gspeech).offset(j as isize),
            audio,
        );
        i = i.wrapping_add(1);
        i;
    }
    HTS_Vocoder_clear(&mut v);
    if !audio.is_null() {
        HTS_Audio_flush(audio);
    }
    return 1 as libc::c_int as HTS_Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_get_total_nsamples(
    mut gss: *mut HTS_GStreamSet,
) -> size_t {
    return (*gss).total_nsample;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_get_total_frame(
    mut gss: *mut HTS_GStreamSet,
) -> size_t {
    return (*gss).total_frame;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_get_vector_length(
    mut gss: *mut HTS_GStreamSet,
    mut stream_index: size_t,
) -> size_t {
    return (*((*gss).gstream).offset(stream_index as isize)).vector_length;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_get_speech(
    mut gss: *mut HTS_GStreamSet,
    mut sample_index: size_t,
) -> libc::c_double {
    return *((*gss).gspeech).offset(sample_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_get_parameter(
    mut gss: *mut HTS_GStreamSet,
    mut stream_index: size_t,
    mut frame_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    return *(*((*((*gss).gstream).offset(stream_index as isize)).par)
        .offset(frame_index as isize))
        .offset(vector_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_GStreamSet_clear(mut gss: *mut HTS_GStreamSet) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    if !((*gss).gstream).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*gss).nstream {
            if !((*((*gss).gstream).offset(i as isize)).par).is_null() {
                j = 0 as libc::c_int as size_t;
                while j < (*gss).total_frame {
                    HTS_free(
                        *((*((*gss).gstream).offset(i as isize)).par).offset(j as isize)
                            as *mut libc::c_void,
                    );
                    j = j.wrapping_add(1);
                    j;
                }
                HTS_free(
                    (*((*gss).gstream).offset(i as isize)).par as *mut libc::c_void,
                );
            }
            i = i.wrapping_add(1);
            i;
        }
        HTS_free((*gss).gstream as *mut libc::c_void);
    }
    if !((*gss).gspeech).is_null() {
        HTS_free((*gss).gspeech as *mut libc::c_void);
    }
    HTS_GStreamSet_initialize(gss);
}
