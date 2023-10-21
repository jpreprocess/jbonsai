#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use crate::{util::*, HTS_error};
extern "C" {
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn HTS_calloc(num: size_t, size: size_t) -> *mut libc::c_void;
    fn HTS_free(p: *mut libc::c_void);
    fn HTS_ModelSet_get_gv_flag(
        ms: *mut HTS_ModelSet,
        string: *const libc::c_char,
    ) -> HTS_Boolean;
    fn HTS_ModelSet_get_nstate(ms: *mut HTS_ModelSet) -> size_t;
    fn HTS_ModelSet_get_nstream(ms: *mut HTS_ModelSet) -> size_t;
    fn HTS_ModelSet_get_nvoices(ms: *mut HTS_ModelSet) -> size_t;
    fn HTS_ModelSet_get_vector_length(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
    ) -> size_t;
    fn HTS_ModelSet_is_msd(ms: *mut HTS_ModelSet, stream_index: size_t) -> HTS_Boolean;
    fn HTS_ModelSet_get_window_size(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
    ) -> size_t;
    fn HTS_ModelSet_get_window_left_width(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
        window_index: size_t,
    ) -> libc::c_int;
    fn HTS_ModelSet_get_window_right_width(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
        window_index: size_t,
    ) -> libc::c_int;
    fn HTS_ModelSet_get_window_coefficient(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
        window_index: size_t,
        coefficient_index: size_t,
    ) -> libc::c_double;
    fn HTS_ModelSet_get_window_max_width(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
    ) -> size_t;
    fn HTS_ModelSet_use_gv(ms: *mut HTS_ModelSet, stream_index: size_t) -> HTS_Boolean;
    fn HTS_ModelSet_get_duration(
        ms: *mut HTS_ModelSet,
        string: *const libc::c_char,
        iw: *const libc::c_double,
        mean: *mut libc::c_double,
        vari: *mut libc::c_double,
    );
    fn HTS_ModelSet_get_parameter(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
        state_index: size_t,
        string: *const libc::c_char,
        iw: *const *const libc::c_double,
        mean: *mut libc::c_double,
        vari: *mut libc::c_double,
        msd: *mut libc::c_double,
    );
    fn HTS_ModelSet_get_gv(
        ms: *mut HTS_ModelSet,
        stream_index: size_t,
        string: *const libc::c_char,
        iw: *const *const libc::c_double,
        mean: *mut libc::c_double,
        vari: *mut libc::c_double,
    );
    fn HTS_Label_get_size(label: *mut HTS_Label) -> size_t;
    fn HTS_Label_get_string(label: *mut HTS_Label, index: size_t) -> *const libc::c_char;
    fn HTS_Label_get_end_frame(label: *mut HTS_Label, index: size_t) -> libc::c_double;
}

unsafe extern "C" fn HTS_set_default_duration(
    mut duration: *mut size_t,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
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
        i;
    }
    return sum as libc::c_double;
}
unsafe extern "C" fn HTS_set_specified_duration(
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
                b"HTS_set_specified_duration: Specified frame length is too short.\n\0"
                    as *const u8 as *const libc::c_char,
            );
        }
        i = 0 as libc::c_int as size_t;
        while i < size {
            *duration.offset(i as isize) = 1 as libc::c_int as size_t;
            i = i.wrapping_add(1);
            i;
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
        i;
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
        i;
    }
    while target_length != sum {
        if target_length > sum {
            j = -(1 as libc::c_int);
            i = 0 as libc::c_int as size_t;
            while i < size {
                temp2 = fabs(
                    rho
                        - (*duration.offset(i as isize) as libc::c_double
                            + 1 as libc::c_int as libc::c_double
                            - *mean.offset(i as isize)) / *vari.offset(i as isize),
                );
                if j < 0 as libc::c_int || temp1 > temp2 {
                    j = i as libc::c_int;
                    temp1 = temp2;
                }
                i = i.wrapping_add(1);
                i;
            }
            sum = sum.wrapping_add(1);
            sum;
            let ref mut fresh0 = *duration.offset(j as isize);
            *fresh0 = (*fresh0).wrapping_add(1);
            *fresh0;
        } else {
            j = -(1 as libc::c_int);
            i = 0 as libc::c_int as size_t;
            while i < size {
                if *duration.offset(i as isize) > 1 as libc::c_int as size_t {
                    temp2 = fabs(
                        rho
                            - (*duration.offset(i as isize) as libc::c_double
                                - 1 as libc::c_int as libc::c_double
                                - *mean.offset(i as isize)) / *vari.offset(i as isize),
                    );
                    if j < 0 as libc::c_int || temp1 > temp2 {
                        j = i as libc::c_int;
                        temp1 = temp2;
                    }
                }
                i = i.wrapping_add(1);
                i;
            }
            sum = sum.wrapping_sub(1);
            sum;
            let ref mut fresh1 = *duration.offset(j as isize);
            *fresh1 = (*fresh1).wrapping_sub(1);
            *fresh1;
        }
    }
    return target_length as libc::c_double;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_initialize(mut sss: *mut HTS_SStreamSet) {
    (*sss).nstream = 0 as libc::c_int as size_t;
    (*sss).nstate = 0 as libc::c_int as size_t;
    (*sss).sstream = 0 as *mut HTS_SStream;
    (*sss).duration = 0 as *mut size_t;
    (*sss).total_state = 0 as libc::c_int as size_t;
    (*sss).total_frame = 0 as libc::c_int as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_create(
    mut sss: *mut HTS_SStreamSet,
    mut ms: *mut HTS_ModelSet,
    mut label: *mut HTS_Label,
    mut phoneme_alignment_flag: HTS_Boolean,
    mut speed: libc::c_double,
    mut duration_iw: *mut libc::c_double,
    mut parameter_iw: *mut *mut libc::c_double,
    mut gv_iw: *mut *mut libc::c_double,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut temp: libc::c_double = 0.;
    let mut shift: libc::c_int = 0;
    let mut state: size_t = 0;
    let mut sst: *mut HTS_SStream = 0 as *mut HTS_SStream;
    let mut duration_mean: *mut libc::c_double = 0 as *mut libc::c_double;
    let mut duration_vari: *mut libc::c_double = 0 as *mut libc::c_double;
    let mut frame_length: libc::c_double = 0.;
    let mut next_time: size_t = 0;
    let mut next_state: size_t = 0;
    i = 0 as libc::c_int as size_t;
    temp = 0.0f64;
    while i < HTS_ModelSet_get_nvoices(ms) {
        temp += *duration_iw.offset(i as isize);
        i = i.wrapping_add(1);
        i;
    }
    if temp == 0.0f64 {
        return 0 as libc::c_int as HTS_Boolean
    } else if temp != 1.0f64 {
        i = 0 as libc::c_int as size_t;
        while i < HTS_ModelSet_get_nvoices(ms) {
            if *duration_iw.offset(i as isize) != 0.0f64 {
                *duration_iw.offset(i as isize) /= temp;
            }
            i = i.wrapping_add(1);
            i;
        }
    }
    i = 0 as libc::c_int as size_t;
    while i < HTS_ModelSet_get_nstream(ms) {
        j = 0 as libc::c_int as size_t;
        temp = 0.0f64;
        while j < HTS_ModelSet_get_nvoices(ms) {
            temp += *(*parameter_iw.offset(j as isize)).offset(i as isize);
            j = j.wrapping_add(1);
            j;
        }
        if temp == 0.0f64 {
            return 0 as libc::c_int as HTS_Boolean
        } else if temp != 1.0f64 {
            j = 0 as libc::c_int as size_t;
            while j < HTS_ModelSet_get_nvoices(ms) {
                if *(*parameter_iw.offset(j as isize)).offset(i as isize) != 0.0f64 {
                    *(*parameter_iw.offset(j as isize)).offset(i as isize) /= temp;
                }
                j = j.wrapping_add(1);
                j;
            }
        }
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            j = 0 as libc::c_int as size_t;
            temp = 0.0f64;
            while j < HTS_ModelSet_get_nvoices(ms) {
                temp += *(*gv_iw.offset(j as isize)).offset(i as isize);
                j = j.wrapping_add(1);
                j;
            }
            if temp == 0.0f64 {
                return 0 as libc::c_int as HTS_Boolean
            } else if temp != 1.0f64 {
                j = 0 as libc::c_int as size_t;
                while j < HTS_ModelSet_get_nvoices(ms) {
                    if *(*gv_iw.offset(j as isize)).offset(i as isize) != 0.0f64 {
                        *(*gv_iw.offset(j as isize)).offset(i as isize) /= temp;
                    }
                    j = j.wrapping_add(1);
                    j;
                }
            }
        }
        i = i.wrapping_add(1);
        i;
    }
    (*sss).nstate = HTS_ModelSet_get_nstate(ms);
    (*sss).nstream = HTS_ModelSet_get_nstream(ms);
    (*sss).total_frame = 0 as libc::c_int as size_t;
    (*sss).total_state = HTS_Label_get_size(label) * (*sss).nstate;
    (*sss)
        .duration = HTS_calloc(
        (*sss).total_state,
        ::core::mem::size_of::<size_t>() as libc::c_ulong,
    ) as *mut size_t;
    (*sss)
        .sstream = HTS_calloc(
        (*sss).nstream,
        ::core::mem::size_of::<HTS_SStream>() as libc::c_ulong,
    ) as *mut HTS_SStream;
    i = 0 as libc::c_int as size_t;
    while i < (*sss).nstream {
        sst = &mut *((*sss).sstream).offset(i as isize) as *mut HTS_SStream;
        (*sst).vector_length = HTS_ModelSet_get_vector_length(ms, i);
        (*sst)
            .mean = HTS_calloc(
            (*sss).total_state,
            ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        (*sst)
            .vari = HTS_calloc(
            (*sss).total_state,
            ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        if HTS_ModelSet_is_msd(ms, i) != 0 {
            (*sst)
                .msd = HTS_calloc(
                (*sss).total_state,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
        } else {
            (*sst).msd = 0 as *mut libc::c_double;
        }
        j = 0 as libc::c_int as size_t;
        while j < (*sss).total_state {
            let ref mut fresh2 = *((*sst).mean).offset(j as isize);
            *fresh2 = HTS_calloc(
                (*sst).vector_length * HTS_ModelSet_get_window_size(ms, i),
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            let ref mut fresh3 = *((*sst).vari).offset(j as isize);
            *fresh3 = HTS_calloc(
                (*sst).vector_length * HTS_ModelSet_get_window_size(ms, i),
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            j = j.wrapping_add(1);
            j;
        }
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            (*sst)
                .gv_switch = HTS_calloc(
                (*sss).total_state,
                ::core::mem::size_of::<HTS_Boolean>() as libc::c_ulong,
            ) as *mut HTS_Boolean;
            j = 0 as libc::c_int as size_t;
            while j < (*sss).total_state {
                *((*sst).gv_switch).offset(j as isize) = 1 as libc::c_int as HTS_Boolean;
                j = j.wrapping_add(1);
                j;
            }
        } else {
            (*sst).gv_switch = 0 as *mut HTS_Boolean;
        }
        i = i.wrapping_add(1);
        i;
    }
    duration_mean = HTS_calloc(
        (*sss).total_state,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    duration_vari = HTS_calloc(
        (*sss).total_state,
        ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
    ) as *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        HTS_ModelSet_get_duration(
            ms,
            HTS_Label_get_string(label, i),
            duration_iw,
            &mut *duration_mean.offset((i * (*sss).nstate) as isize),
            &mut *duration_vari.offset((i * (*sss).nstate) as isize),
        );
        i = i.wrapping_add(1);
        i;
    }
    if phoneme_alignment_flag as libc::c_int == 1 as libc::c_int {
        next_time = 0 as libc::c_int as size_t;
        next_state = 0 as libc::c_int as size_t;
        state = 0 as libc::c_int as size_t;
        i = 0 as libc::c_int as size_t;
        while i < HTS_Label_get_size(label) {
            temp = HTS_Label_get_end_frame(label, i);
            if temp >= 0 as libc::c_int as libc::c_double {
                next_time = next_time
                    .wrapping_add(
                        HTS_set_specified_duration(
                            &mut *((*sss).duration).offset(next_state as isize),
                            &mut *duration_mean.offset(next_state as isize),
                            &mut *duration_vari.offset(next_state as isize),
                            state.wrapping_add((*sss).nstate).wrapping_sub(next_state),
                            temp - next_time as libc::c_double,
                        ) as size_t,
                    );
                next_state = state.wrapping_add((*sss).nstate);
            } else if i.wrapping_add(1 as libc::c_int as size_t)
                == HTS_Label_get_size(label)
            {
                HTS_error!(
                    -(1 as libc::c_int),
                    b"HTS_SStreamSet_create: The time of final label is not specified.\n\0"
                        as *const u8 as *const libc::c_char,
                );
                HTS_set_default_duration(
                    &mut *((*sss).duration).offset(next_state as isize),
                    &mut *duration_mean.offset(next_state as isize),
                    &mut *duration_vari.offset(next_state as isize),
                    state.wrapping_add((*sss).nstate).wrapping_sub(next_state),
                );
            }
            state = state.wrapping_add((*sss).nstate);
            i = i.wrapping_add(1);
            i;
        }
    } else if speed != 1.0f64 {
        temp = 0.0f64;
        i = 0 as libc::c_int as size_t;
        while i < (*sss).total_state {
            temp += *duration_mean.offset(i as isize);
            i = i.wrapping_add(1);
            i;
        }
        frame_length = temp / speed;
        HTS_set_specified_duration(
            (*sss).duration,
            duration_mean,
            duration_vari,
            (*sss).total_state,
            frame_length,
        );
    } else {
        HTS_set_default_duration(
            (*sss).duration,
            duration_mean,
            duration_vari,
            (*sss).total_state,
        );
    }
    HTS_free(duration_mean as *mut libc::c_void);
    HTS_free(duration_vari as *mut libc::c_void);
    i = 0 as libc::c_int as size_t;
    state = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        j = 2 as libc::c_int as size_t;
        while j <= ((*sss).nstate).wrapping_add(1 as libc::c_int as size_t) {
            (*sss)
                .total_frame = ((*sss).total_frame)
                .wrapping_add(*((*sss).duration).offset(state as isize));
            k = 0 as libc::c_int as size_t;
            while k < (*sss).nstream {
                sst = &mut *((*sss).sstream).offset(k as isize) as *mut HTS_SStream;
                if !((*sst).msd).is_null() {
                    HTS_ModelSet_get_parameter(
                        ms,
                        k,
                        j,
                        HTS_Label_get_string(label, i),
                        parameter_iw as *const *const libc::c_double,
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
                        parameter_iw as *const *const libc::c_double,
                        *((*sst).mean).offset(state as isize),
                        *((*sst).vari).offset(state as isize),
                        0 as *mut libc::c_double,
                    );
                }
                k = k.wrapping_add(1);
                k;
            }
            state = state.wrapping_add(1);
            state;
            j = j.wrapping_add(1);
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
    i = 0 as libc::c_int as size_t;
    while i < (*sss).nstream {
        sst = &mut *((*sss).sstream).offset(i as isize) as *mut HTS_SStream;
        (*sst).win_size = HTS_ModelSet_get_window_size(ms, i);
        (*sst).win_max_width = HTS_ModelSet_get_window_max_width(ms, i);
        (*sst)
            .win_l_width = HTS_calloc(
            (*sst).win_size,
            ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ) as *mut libc::c_int;
        (*sst)
            .win_r_width = HTS_calloc(
            (*sst).win_size,
            ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ) as *mut libc::c_int;
        (*sst)
            .win_coefficient = HTS_calloc(
            (*sst).win_size,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < (*sst).win_size {
            *((*sst).win_l_width)
                .offset(j as isize) = HTS_ModelSet_get_window_left_width(ms, i, j);
            *((*sst).win_r_width)
                .offset(j as isize) = HTS_ModelSet_get_window_right_width(ms, i, j);
            if *((*sst).win_l_width).offset(j as isize)
                + *((*sst).win_r_width).offset(j as isize) == 0 as libc::c_int
            {
                let ref mut fresh4 = *((*sst).win_coefficient).offset(j as isize);
                *fresh4 = HTS_calloc(
                    (-(2 as libc::c_int) * *((*sst).win_l_width).offset(j as isize)
                        + 1 as libc::c_int) as size_t,
                    ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
                ) as *mut libc::c_double;
            } else {
                let ref mut fresh5 = *((*sst).win_coefficient).offset(j as isize);
                *fresh5 = HTS_calloc(
                    (-(2 as libc::c_int) * *((*sst).win_l_width).offset(j as isize))
                        as size_t,
                    ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
                ) as *mut libc::c_double;
            }
            let ref mut fresh6 = *((*sst).win_coefficient).offset(j as isize);
            *fresh6 = (*fresh6)
                .offset(-(*((*sst).win_l_width).offset(j as isize) as isize));
            shift = *((*sst).win_l_width).offset(j as isize);
            while shift <= *((*sst).win_r_width).offset(j as isize) {
                *(*((*sst).win_coefficient).offset(j as isize))
                    .offset(
                        shift as isize,
                    ) = HTS_ModelSet_get_window_coefficient(ms, i, j, shift as size_t);
                shift += 1;
                shift;
            }
            j = j.wrapping_add(1);
            j;
        }
        i = i.wrapping_add(1);
        i;
    }
    i = 0 as libc::c_int as size_t;
    while i < (*sss).nstream {
        sst = &mut *((*sss).sstream).offset(i as isize) as *mut HTS_SStream;
        if HTS_ModelSet_use_gv(ms, i) != 0 {
            (*sst)
                .gv_mean = HTS_calloc(
                (*sst).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            (*sst)
                .gv_vari = HTS_calloc(
                (*sst).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            HTS_ModelSet_get_gv(
                ms,
                i,
                HTS_Label_get_string(label, 0 as libc::c_int as size_t),
                gv_iw as *const *const libc::c_double,
                (*sst).gv_mean,
                (*sst).gv_vari,
            );
        } else {
            (*sst).gv_mean = 0 as *mut libc::c_double;
            (*sst).gv_vari = 0 as *mut libc::c_double;
        }
        i = i.wrapping_add(1);
        i;
    }
    i = 0 as libc::c_int as size_t;
    while i < HTS_Label_get_size(label) {
        if HTS_ModelSet_get_gv_flag(ms, HTS_Label_get_string(label, i)) as libc::c_int
            == 0 as libc::c_int
        {
            j = 0 as libc::c_int as size_t;
            while j < (*sss).nstream {
                if HTS_ModelSet_use_gv(ms, j) as libc::c_int == 1 as libc::c_int {
                    k = 0 as libc::c_int as size_t;
                    while k < (*sss).nstate {
                        *((*((*sss).sstream).offset(j as isize)).gv_switch)
                            .offset(
                                (i * (*sss).nstate).wrapping_add(k) as isize,
                            ) = 0 as libc::c_int as HTS_Boolean;
                        k = k.wrapping_add(1);
                        k;
                    }
                }
                j = j.wrapping_add(1);
                j;
            }
        }
        i = i.wrapping_add(1);
        i;
    }
    return 1 as libc::c_int as HTS_Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_nstream(
    mut sss: *mut HTS_SStreamSet,
) -> size_t {
    return (*sss).nstream;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_vector_length(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> size_t {
    return (*((*sss).sstream).offset(stream_index as isize)).vector_length;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_is_msd(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    return (if !((*((*sss).sstream).offset(stream_index as isize)).msd).is_null() {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    }) as HTS_Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_total_state(
    mut sss: *mut HTS_SStreamSet,
) -> size_t {
    return (*sss).total_state;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_total_frame(
    mut sss: *mut HTS_SStreamSet,
) -> size_t {
    return (*sss).total_frame;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_msd(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
) -> libc::c_double {
    return *((*((*sss).sstream).offset(stream_index as isize)).msd)
        .offset(state_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_window_size(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> size_t {
    return (*((*sss).sstream).offset(stream_index as isize)).win_size;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_window_left_width(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut window_index: size_t,
) -> libc::c_int {
    return *((*((*sss).sstream).offset(stream_index as isize)).win_l_width)
        .offset(window_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_window_right_width(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut window_index: size_t,
) -> libc::c_int {
    return *((*((*sss).sstream).offset(stream_index as isize)).win_r_width)
        .offset(window_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_window_coefficient(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut window_index: size_t,
    mut coefficient_index: libc::c_int,
) -> libc::c_double {
    return *(*((*((*sss).sstream).offset(stream_index as isize)).win_coefficient)
        .offset(window_index as isize))
        .offset(coefficient_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_window_max_width(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> size_t {
    return (*((*sss).sstream).offset(stream_index as isize)).win_max_width;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_use_gv(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    return (if !((*((*sss).sstream).offset(stream_index as isize)).gv_mean).is_null() {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    }) as HTS_Boolean;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_duration(
    mut sss: *mut HTS_SStreamSet,
    mut state_index: size_t,
) -> size_t {
    return *((*sss).duration).offset(state_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_mean(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    return *(*((*((*sss).sstream).offset(stream_index as isize)).mean)
        .offset(state_index as isize))
        .offset(vector_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_set_mean(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
    mut f: libc::c_double,
) {
    *(*((*((*sss).sstream).offset(stream_index as isize)).mean)
        .offset(state_index as isize))
        .offset(vector_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_vari(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    return *(*((*((*sss).sstream).offset(stream_index as isize)).vari)
        .offset(state_index as isize))
        .offset(vector_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_set_vari(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut vector_index: size_t,
    mut f: libc::c_double,
) {
    *(*((*((*sss).sstream).offset(stream_index as isize)).vari)
        .offset(state_index as isize))
        .offset(vector_index as isize) = f;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_gv_mean(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    return *((*((*sss).sstream).offset(stream_index as isize)).gv_mean)
        .offset(vector_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_gv_vari(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    return *((*((*sss).sstream).offset(stream_index as isize)).gv_vari)
        .offset(vector_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_set_gv_switch(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut i: HTS_Boolean,
) {
    *((*((*sss).sstream).offset(stream_index as isize)).gv_switch)
        .offset(state_index as isize) = i;
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_get_gv_switch(
    mut sss: *mut HTS_SStreamSet,
    mut stream_index: size_t,
    mut state_index: size_t,
) -> HTS_Boolean {
    return *((*((*sss).sstream).offset(stream_index as isize)).gv_switch)
        .offset(state_index as isize);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_SStreamSet_clear(mut sss: *mut HTS_SStreamSet) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut sst: *mut HTS_SStream = 0 as *mut HTS_SStream;
    if !((*sss).sstream).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*sss).nstream {
            sst = &mut *((*sss).sstream).offset(i as isize) as *mut HTS_SStream;
            j = 0 as libc::c_int as size_t;
            while j < (*sss).total_state {
                HTS_free(*((*sst).mean).offset(j as isize) as *mut libc::c_void);
                HTS_free(*((*sst).vari).offset(j as isize) as *mut libc::c_void);
                j = j.wrapping_add(1);
                j;
            }
            if !((*sst).msd).is_null() {
                HTS_free((*sst).msd as *mut libc::c_void);
            }
            HTS_free((*sst).mean as *mut libc::c_void);
            HTS_free((*sst).vari as *mut libc::c_void);
            j = 0 as libc::c_int as size_t;
            while j < (*sst).win_size {
                let ref mut fresh7 = *((*sst).win_coefficient).offset(j as isize);
                *fresh7 = (*fresh7)
                    .offset(*((*sst).win_l_width).offset(j as isize) as isize);
                HTS_free(
                    *((*sst).win_coefficient).offset(j as isize) as *mut libc::c_void,
                );
                j = j.wrapping_add(1);
                j;
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
            i;
        }
        HTS_free((*sss).sstream as *mut libc::c_void);
    }
    if !((*sss).duration).is_null() {
        HTS_free((*sss).duration as *mut libc::c_void);
    }
    HTS_SStreamSet_initialize(sss);
}
