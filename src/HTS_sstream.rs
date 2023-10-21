#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use crate::{util::*, HTS_Label, HTS_ModelSet, HTS_error};
extern "C" {
    fn fabs(_: libc::c_double) -> libc::c_double;
}

use crate::{
    HTS_Label_get_end_frame, HTS_Label_get_size, HTS_Label_get_string, HTS_ModelSet_get_duration,
    HTS_ModelSet_get_gv, HTS_ModelSet_get_gv_flag, HTS_ModelSet_get_nstate,
    HTS_ModelSet_get_nstream, HTS_ModelSet_get_nvoices, HTS_ModelSet_get_parameter,
    HTS_ModelSet_get_vector_length, HTS_ModelSet_get_window_coefficient,
    HTS_ModelSet_get_window_left_width, HTS_ModelSet_get_window_max_width,
    HTS_ModelSet_get_window_right_width, HTS_ModelSet_get_window_size, HTS_ModelSet_is_msd,
    HTS_ModelSet_use_gv, HTS_calloc, HTS_free,
};

#[derive(Clone)]
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

#[derive(Clone)]
pub struct HTS_SStreamSet {
    pub sstream: *mut HTS_SStream,
    pub nstream: size_t,
    pub nstate: size_t,
    pub duration: *mut size_t,
    pub total_state: size_t,
    pub total_frame: size_t,
}

#[derive(Clone)]
pub struct HTS_SMatrices {
    pub mean: *mut *mut libc::c_double,
    pub ivar: *mut *mut libc::c_double,
    pub g: *mut libc::c_double,
    pub wuw: *mut *mut libc::c_double,
    pub wum: *mut libc::c_double,
}

unsafe fn HTS_set_default_duration(
    mut duration: *mut size_t,
    mut mean: *mut libc::c_double,
    mut _vari: *mut libc::c_double,
    mut size: size_t,
) -> libc::c_double {
    let mut i: size_t = 0;
    let mut temp: libc::c_double = 0.;
    let mut sum: size_t = 0 as libc::c_int as size_t;
    i = 0 as libc::c_int as size_t;
    while i < size {
        temp = *mean.offset(i as isize) + 0.5f64;
        if temp < 1.0f64 {
            *duration.offset(i as isize) = 1 as libc::c_int as size_t;
        } else {
            *duration.offset(i as isize) = temp as size_t;
        }
        sum = sum.wrapping_add(*duration.offset(i as isize));
        i = i.wrapping_add(1);
    }
    sum as libc::c_double
}
unsafe fn HTS_set_specified_duration(
    mut duration: *mut size_t,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
    mut size: size_t,
    mut frame_length: libc::c_double,
) -> libc::c_double {
    let mut i: size_t = 0;
    let mut j: libc::c_int = 0;
    let mut temp1: libc::c_double = 0.;
    let mut temp2: libc::c_double = 0.;
    let mut rho: libc::c_double = 0.0f64;
    let mut sum: size_t = 0 as libc::c_int as size_t;
    let mut target_length: size_t = 0;
    if frame_length + 0.5f64 < 1.0f64 {
        target_length = 1 as libc::c_int as size_t;
    } else {
        target_length = (frame_length + 0.5f64) as size_t;
    }
    if target_length <= size {
        if target_length < size {
            HTS_error!(
                -(1 as libc::c_int),
                b"HTS_set_specified_duration: Specified frame length is too short.\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
        i = 0 as libc::c_int as size_t;
        while i < size {
            *duration.offset(i as isize) = 1 as libc::c_int as size_t;
            i = i.wrapping_add(1);
        }
        return size as libc::c_double;
    }
    temp1 = 0.0f64;
    temp2 = 0.0f64;
    i = 0 as libc::c_int as size_t;
    while i < size {
        temp1 += *mean.offset(i as isize);
        temp2 += *vari.offset(i as isize);
        i = i.wrapping_add(1);
    }
    rho = (target_length as libc::c_double - temp1) / temp2;
    i = 0 as libc::c_int as size_t;
    while i < size {
        temp1 = *mean.offset(i as isize) + rho * *vari.offset(i as isize) + 0.5f64;
        if temp1 < 1.0f64 {
            *duration.offset(i as isize) = 1 as libc::c_int as size_t;
        } else {
            *duration.offset(i as isize) = temp1 as size_t;
        }
        sum = sum.wrapping_add(*duration.offset(i as isize));
        i = i.wrapping_add(1);
    }
    while target_length != sum {
        if target_length > sum {
            j = -(1 as libc::c_int);
            i = 0 as libc::c_int as size_t;
            while i < size {
                temp2 = fabs(
                    rho - (*duration.offset(i as isize) as libc::c_double
                        + 1 as libc::c_int as libc::c_double
                        - *mean.offset(i as isize))
                        / *vari.offset(i as isize),
                );
                if j < 0 as libc::c_int || temp1 > temp2 {
                    j = i as libc::c_int;
                    temp1 = temp2;
                }
                i = i.wrapping_add(1);
            }
            sum = sum.wrapping_add(1);
            let fresh0 = &mut (*duration.offset(j as isize));
            *fresh0 = (*fresh0).wrapping_add(1);
        } else {
            j = -(1 as libc::c_int);
            i = 0 as libc::c_int as size_t;
            while i < size {
                if *duration.offset(i as isize) > 1 as libc::c_int as size_t {
                    temp2 = fabs(
                        rho - (*duration.offset(i as isize) as libc::c_double
                            - 1 as libc::c_int as libc::c_double
                            - *mean.offset(i as isize))
                            / *vari.offset(i as isize),
                    );
                    if j < 0 as libc::c_int || temp1 > temp2 {
                        j = i as libc::c_int;
                        temp1 = temp2;
                    }
                }
                i = i.wrapping_add(1);
            }
            sum = sum.wrapping_sub(1);
            let fresh1 = &mut (*duration.offset(j as isize));
            *fresh1 = (*fresh1).wrapping_sub(1);
        }
    }
    target_length as libc::c_double
}

pub fn HTS_SStreamSet_initialize() -> HTS_SStreamSet {
    HTS_SStreamSet {
        nstream: 0 as libc::c_int as size_t,
        nstate: 0 as libc::c_int as size_t,
        sstream: std::ptr::null_mut::<HTS_SStream>(),
        duration: std::ptr::null_mut::<size_t>(),
        total_state: 0 as libc::c_int as size_t,
        total_frame: 0 as libc::c_int as size_t,
    }
}

pub unsafe fn HTS_SStreamSet_create(
    sss: &mut HTS_SStreamSet,
    mut ms: &mut HTS_ModelSet,
    mut label: &mut HTS_Label,
    mut phoneme_alignment_flag: HTS_Boolean,
    mut speed: libc::c_double,
    mut duration_iw: &mut Vec<f64>,
    mut parameter_iw: &mut Vec<Vec<f64>>,
    mut gv_iw: &mut Vec<Vec<f64>>,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut temp: libc::c_double = 0.;
    let mut shift: libc::c_int = 0;
    let mut state: size_t = 0;
    let mut sst: *mut HTS_SStream = std::ptr::null_mut::<HTS_SStream>();
    let mut duration_mean: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut duration_vari: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut frame_length: libc::c_double = 0.;
    let mut next_time: size_t = 0;
    let mut next_state: size_t = 0;
    i = 0 as libc::c_int as size_t;
    temp = 0.0f64;
    while i < HTS_ModelSet_get_nvoices(ms) {
        temp += duration_iw[i as usize];
        i = i.wrapping_add(1);
    }
    if temp == 0.0f64 {
        return 0 as libc::c_int as HTS_Boolean;
    } else if temp != 1.0f64 {
        i = 0 as libc::c_int as size_t;
        while i < HTS_ModelSet_get_nvoices(ms) {
            if duration_iw[i as usize] != 0.0f64 {
                duration_iw[i as usize] /= temp;
            }
            i = i.wrapping_add(1);
        }
    }
    i = 0 as libc::c_int as size_t;
    while i < HTS_ModelSet_get_nstream(ms) {
        j = 0 as libc::c_int as size_t;
        temp = 0.0f64;
        while j < HTS_ModelSet_get_nvoices(ms) {
            temp += parameter_iw[j as usize][i as usize];
            j = j.wrapping_add(1);
        }
        if temp == 0.0f64 {
            return 0 as libc::c_int as HTS_Boolean;
        } else if temp != 1.0f64 {
            j = 0 as libc::c_int as size_t;
            while j < HTS_ModelSet_get_nvoices(ms) {
                if parameter_iw[j as usize][i as usize] != 0.0f64 {
                    parameter_iw[j as usize][i as usize] /= temp;
                }
                j = j.wrapping_add(1);
            }
        }
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            j = 0 as libc::c_int as size_t;
            temp = 0.0f64;
            while j < HTS_ModelSet_get_nvoices(ms) {
                temp += gv_iw[j as usize][i as usize];
                j = j.wrapping_add(1);
            }
            if temp == 0.0f64 {
                return 0 as libc::c_int as HTS_Boolean;
            } else if temp != 1.0f64 {
                j = 0 as libc::c_int as size_t;
                while j < HTS_ModelSet_get_nvoices(ms) {
                    if gv_iw[j as usize][i as usize] != 0.0f64 {
                        gv_iw[j as usize][i as usize] /= temp;
                    }
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    sss.nstate = HTS_ModelSet_get_nstate(ms);
    sss.nstream = HTS_ModelSet_get_nstream(ms);
    sss.total_frame = 0 as libc::c_int as size_t;
    sss.total_state = HTS_Label_get_size(label) * sss.nstate;
    sss.duration = HTS_calloc(
        sss.total_state,
        ::core::mem::size_of::<size_t>() as libc::c_ulong,
    ) as *mut size_t;
    sss.sstream = HTS_calloc(
        sss.nstream,
        ::core::mem::size_of::<HTS_SStream>() as libc::c_ulong,
    ) as *mut HTS_SStream;
    i = 0 as libc::c_int as size_t;
    while i < sss.nstream {
        sst = &mut *(sss.sstream).offset(i as isize) as *mut HTS_SStream;
        (*sst).vector_length = HTS_ModelSet_get_vector_length(ms, i);
        (*sst).mean = HTS_calloc(
            sss.total_state,
            ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        (*sst).vari = HTS_calloc(
            sss.total_state,
            ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        if HTS_ModelSet_is_msd(ms, i) != 0 {
            (*sst).msd = HTS_calloc(
                sss.total_state,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
        } else {
            (*sst).msd = std::ptr::null_mut::<libc::c_double>();
        }
        j = 0 as libc::c_int as size_t;
        while j < sss.total_state {
            let fresh2 = &mut (*((*sst).mean).offset(j as isize));
            *fresh2 = HTS_calloc(
                (*sst).vector_length * HTS_ModelSet_get_window_size(ms, i),
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            let fresh3 = &mut (*((*sst).vari).offset(j as isize));
            *fresh3 = HTS_calloc(
                (*sst).vector_length * HTS_ModelSet_get_window_size(ms, i),
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            j = j.wrapping_add(1);
        }
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            (*sst).gv_switch = HTS_calloc(
                sss.total_state,
                ::core::mem::size_of::<HTS_Boolean>() as libc::c_ulong,
            ) as *mut HTS_Boolean;
            j = 0 as libc::c_int as size_t;
            while j < sss.total_state {
                *((*sst).gv_switch).offset(j as isize) = 1 as libc::c_int as HTS_Boolean;
                j = j.wrapping_add(1);
            }
        } else {
            (*sst).gv_switch = std::ptr::null_mut::<HTS_Boolean>();
        }
        i = i.wrapping_add(1);
    }
    duration_mean = HTS_calloc(
        sss.total_state,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    duration_vari = HTS_calloc(
        sss.total_state,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        HTS_ModelSet_get_duration(
            ms,
            HTS_Label_get_string(label, i),
            duration_iw,
            &mut *duration_mean.offset((i * sss.nstate) as isize),
            &mut *duration_vari.offset((i * sss.nstate) as isize),
        );
        i = i.wrapping_add(1);
    }
    if phoneme_alignment_flag as libc::c_int == 1 as libc::c_int {
        next_time = 0 as libc::c_int as size_t;
        next_state = 0 as libc::c_int as size_t;
        state = 0 as libc::c_int as size_t;
        i = 0 as libc::c_int as size_t;
        while i < HTS_Label_get_size(label) {
            temp = HTS_Label_get_end_frame(label, i);
            if temp >= 0 as libc::c_int as libc::c_double {
                next_time = next_time.wrapping_add(HTS_set_specified_duration(
                    &mut *(sss.duration).offset(next_state as isize),
                    &mut *duration_mean.offset(next_state as isize),
                    &mut *duration_vari.offset(next_state as isize),
                    state.wrapping_add(sss.nstate).wrapping_sub(next_state),
                    temp - next_time as libc::c_double,
                ) as size_t);
                next_state = state.wrapping_add(sss.nstate);
            } else if i.wrapping_add(1 as libc::c_int as size_t) == HTS_Label_get_size(label) {
                HTS_error!(
                    -(1 as libc::c_int),
                    b"HTS_SStreamSet_create: The time of final label is not specified.\n\0"
                        as *const u8 as *const libc::c_char,
                );
                HTS_set_default_duration(
                    &mut *(sss.duration).offset(next_state as isize),
                    &mut *duration_mean.offset(next_state as isize),
                    &mut *duration_vari.offset(next_state as isize),
                    state.wrapping_add(sss.nstate).wrapping_sub(next_state),
                );
            }
            state = state.wrapping_add(sss.nstate);
            i = i.wrapping_add(1);
        }
    } else if speed != 1.0f64 {
        temp = 0.0f64;
        i = 0 as libc::c_int as size_t;
        while i < sss.total_state {
            temp += *duration_mean.offset(i as isize);
            i = i.wrapping_add(1);
        }
        frame_length = temp / speed;
        HTS_set_specified_duration(
            sss.duration,
            duration_mean,
            duration_vari,
            sss.total_state,
            frame_length,
        );
    } else {
        HTS_set_default_duration(sss.duration, duration_mean, duration_vari, sss.total_state);
    }
    HTS_free(duration_mean as *mut libc::c_void);
    HTS_free(duration_vari as *mut libc::c_void);
    i = 0 as libc::c_int as size_t;
    state = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        j = 2 as libc::c_int as size_t;
        while j <= (sss.nstate).wrapping_add(1 as libc::c_int as size_t) {
            sss.total_frame =
                (sss.total_frame).wrapping_add(*(sss.duration).offset(state as isize));
            k = 0 as libc::c_int as size_t;
            while k < sss.nstream {
                sst = &mut *(sss.sstream).offset(k as isize);
                if !((*sst).msd).is_null() {
                    HTS_ModelSet_get_parameter(
                        ms,
                        k,
                        j,
                        HTS_Label_get_string(label, i),
                        &mut parameter_iw,
                        *((*sst).mean).offset(state as isize),
                        *((*sst).vari).offset(state as isize),
                        &mut *((*sst).msd).offset(state as isize),
                    );
                } else {
                    HTS_ModelSet_get_parameter(
                        ms,
                        k,
                        j,
                        HTS_Label_get_string(label, i),
                        &mut parameter_iw,
                        *((*sst).mean).offset(state as isize),
                        *((*sst).vari).offset(state as isize),
                        std::ptr::null_mut::<libc::c_double>(),
                    );
                }
                k = k.wrapping_add(1);
            }
            state = state.wrapping_add(1);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as libc::c_int as size_t;
    while i < sss.nstream {
        sst = &mut *(sss.sstream).offset(i as isize);
        (*sst).win_size = HTS_ModelSet_get_window_size(ms, i);
        (*sst).win_max_width = HTS_ModelSet_get_window_max_width(ms, i);
        (*sst).win_l_width = HTS_calloc(
            (*sst).win_size,
            ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ) as *mut libc::c_int;
        (*sst).win_r_width = HTS_calloc(
            (*sst).win_size,
            ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ) as *mut libc::c_int;
        (*sst).win_coefficient = HTS_calloc(
            (*sst).win_size,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < (*sst).win_size {
            *((*sst).win_l_width).offset(j as isize) = HTS_ModelSet_get_window_left_width(ms, i, j);
            *((*sst).win_r_width).offset(j as isize) =
                HTS_ModelSet_get_window_right_width(ms, i, j);
            if *((*sst).win_l_width).offset(j as isize) + *((*sst).win_r_width).offset(j as isize)
                == 0 as libc::c_int
            {
                let fresh4 = &mut (*((*sst).win_coefficient).offset(j as isize));
                *fresh4 = HTS_calloc(
                    (-(2 as libc::c_int) * *((*sst).win_l_width).offset(j as isize)
                        + 1 as libc::c_int) as size_t,
                    ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
                ) as *mut libc::c_double;
            } else {
                let fresh5 = &mut (*((*sst).win_coefficient).offset(j as isize));
                *fresh5 = HTS_calloc(
                    (-(2 as libc::c_int) * *((*sst).win_l_width).offset(j as isize)) as size_t,
                    ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
                ) as *mut libc::c_double;
            }
            let fresh6 = &mut (*((*sst).win_coefficient).offset(j as isize));
            *fresh6 = (*fresh6).offset(-(*((*sst).win_l_width).offset(j as isize) as isize));
            shift = *((*sst).win_l_width).offset(j as isize);
            while shift <= *((*sst).win_r_width).offset(j as isize) {
                *(*((*sst).win_coefficient).offset(j as isize)).offset(shift as isize) =
                    HTS_ModelSet_get_window_coefficient(ms, i, j, shift as size_t);
                shift += 1;
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as libc::c_int as size_t;
    while i < sss.nstream {
        sst = &mut *(sss.sstream).offset(i as isize) as *mut HTS_SStream;
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            (*sst).gv_mean = HTS_calloc(
                (*sst).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            (*sst).gv_vari = HTS_calloc(
                (*sst).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            HTS_ModelSet_get_gv(
                ms,
                i,
                HTS_Label_get_string(label, 0 as libc::c_int as size_t),
                &gv_iw,
                (*sst).gv_mean,
                (*sst).gv_vari,
            );
        } else {
            (*sst).gv_mean = std::ptr::null_mut::<libc::c_double>();
            (*sst).gv_vari = std::ptr::null_mut::<libc::c_double>();
        }
        i = i.wrapping_add(1);
    }
    i = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        if HTS_ModelSet_get_gv_flag(ms, HTS_Label_get_string(label, i)) as libc::c_int
            == 0 as libc::c_int
        {
            j = 0 as libc::c_int as size_t;
            while j < sss.nstream {
                if HTS_ModelSet_use_gv(ms, j) as libc::c_int == 1 as libc::c_int {
                    k = 0 as libc::c_int as size_t;
                    while k < sss.nstate {
                        *((*(sss.sstream).offset(j as isize)).gv_switch)
                            .offset((i * sss.nstate).wrapping_add(k) as isize) =
                            0 as libc::c_int as HTS_Boolean;
                        k = k.wrapping_add(1);
                    }
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    1 as libc::c_int as HTS_Boolean
}

pub unsafe fn HTS_SStreamSet_get_nstream(sss: &mut HTS_SStreamSet) -> size_t {
    sss.nstream
}

pub unsafe fn HTS_SStreamSet_get_vector_length(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> size_t {
    (*(sss.sstream).offset(stream_index as isize)).vector_length
}

pub unsafe fn HTS_SStreamSet_is_msd(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    (if !((*(sss.sstream).offset(stream_index as isize)).msd).is_null() {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    }) as HTS_Boolean
}

pub unsafe fn HTS_SStreamSet_get_total_state(sss: &mut HTS_SStreamSet) -> size_t {
    sss.total_state
}

pub unsafe fn HTS_SStreamSet_get_total_frame(sss: &mut HTS_SStreamSet) -> size_t {
    sss.total_frame
}

pub unsafe fn HTS_SStreamSet_get_msd(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
) -> libc::c_double {
    *((*(sss.sstream).offset(stream_index as isize)).msd).offset(state_index as isize)
}

pub unsafe fn HTS_SStreamSet_get_window_size(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> size_t {
    (*(sss.sstream).offset(stream_index as isize)).win_size
}

pub unsafe fn HTS_SStreamSet_get_window_left_width(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut window_index: size_t,
) -> libc::c_int {
    *((*(sss.sstream).offset(stream_index as isize)).win_l_width).offset(window_index as isize)
}

pub unsafe fn HTS_SStreamSet_get_window_right_width(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut window_index: size_t,
) -> libc::c_int {
    *((*(sss.sstream).offset(stream_index as isize)).win_r_width).offset(window_index as isize)
}

pub unsafe fn HTS_SStreamSet_get_window_coefficient(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut window_index: size_t,
    mut coefficient_index: libc::c_int,
) -> libc::c_double {
    *(*((*(sss.sstream).offset(stream_index as isize)).win_coefficient)
        .offset(window_index as isize))
    .offset(coefficient_index as isize)
}

pub unsafe fn HTS_SStreamSet_get_window_max_width(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> size_t {
    (*(sss.sstream).offset(stream_index as isize)).win_max_width
}

pub unsafe fn HTS_SStreamSet_use_gv(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    (if !((*(sss.sstream).offset(stream_index as isize)).gv_mean).is_null() {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    }) as HTS_Boolean
}

pub unsafe fn HTS_SStreamSet_get_duration(
    sss: &mut HTS_SStreamSet,
    mut state_index: size_t,
) -> size_t {
    *(sss.duration).offset(state_index as isize)
}

pub unsafe fn HTS_SStreamSet_get_mean(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    *(*((*(sss.sstream).offset(stream_index as isize)).mean).offset(state_index as isize))
        .offset(vector_index as isize)
}

pub unsafe fn HTS_SStreamSet_set_mean(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
    mut f: libc::c_double,
) {
    *(*((*(sss.sstream).offset(stream_index as isize)).mean).offset(state_index as isize))
        .offset(vector_index as isize) = f;
}

pub unsafe fn HTS_SStreamSet_get_vari(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    *(*((*(sss.sstream).offset(stream_index as isize)).vari).offset(state_index as isize))
        .offset(vector_index as isize)
}

pub unsafe fn HTS_SStreamSet_set_vari(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
    mut f: libc::c_double,
) {
    *(*((*(sss.sstream).offset(stream_index as isize)).vari).offset(state_index as isize))
        .offset(vector_index as isize) = f;
}

pub unsafe fn HTS_SStreamSet_get_gv_mean(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    *((*(sss.sstream).offset(stream_index as isize)).gv_mean).offset(vector_index as isize)
}

pub unsafe fn HTS_SStreamSet_get_gv_vari(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    *((*(sss.sstream).offset(stream_index as isize)).gv_vari).offset(vector_index as isize)
}

pub unsafe fn HTS_SStreamSet_set_gv_switch(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut i: HTS_Boolean,
) {
    *((*(sss.sstream).offset(stream_index as isize)).gv_switch).offset(state_index as isize) = i;
}

pub unsafe fn HTS_SStreamSet_get_gv_switch(
    sss: &mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
) -> HTS_Boolean {
    *((*(sss.sstream).offset(stream_index as isize)).gv_switch).offset(state_index as isize)
}

pub unsafe fn HTS_SStreamSet_clear(sss: &mut HTS_SStreamSet) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut sst: *mut HTS_SStream = std::ptr::null_mut::<HTS_SStream>();
    if !(sss.sstream).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < sss.nstream {
            sst = &mut *(sss.sstream).offset(i as isize) as *mut HTS_SStream;
            j = 0 as libc::c_int as size_t;
            while j < sss.total_state {
                HTS_free(*((*sst).mean).offset(j as isize) as *mut libc::c_void);
                HTS_free(*((*sst).vari).offset(j as isize) as *mut libc::c_void);
                j = j.wrapping_add(1);
            }
            if !((*sst).msd).is_null() {
                HTS_free((*sst).msd as *mut libc::c_void);
            }
            HTS_free((*sst).mean as *mut libc::c_void);
            HTS_free((*sst).vari as *mut libc::c_void);
            j = 0 as libc::c_int as size_t;
            while j < (*sst).win_size {
                let fresh7 = &mut (*((*sst).win_coefficient).offset(j as isize));
                *fresh7 = (*fresh7).offset(*((*sst).win_l_width).offset(j as isize) as isize);
                HTS_free(*((*sst).win_coefficient).offset(j as isize) as *mut libc::c_void);
                j = j.wrapping_add(1);
            }
            HTS_free((*sst).win_coefficient as *mut libc::c_void);
            HTS_free((*sst).win_l_width as *mut libc::c_void);
            HTS_free((*sst).win_r_width as *mut libc::c_void);
            if !((*sst).gv_mean).is_null() {
                HTS_free((*sst).gv_mean as *mut libc::c_void);
            }
            if !((*sst).gv_vari).is_null() {
                HTS_free((*sst).gv_vari as *mut libc::c_void);
            }
            if !((*sst).gv_switch).is_null() {
                HTS_free((*sst).gv_switch as *mut libc::c_void);
            }
            i = i.wrapping_add(1);
        }
        HTS_free(sss.sstream as *mut libc::c_void);
    }
    if !(sss.duration).is_null() {
        HTS_free(sss.duration as *mut libc::c_void);
    }
}
