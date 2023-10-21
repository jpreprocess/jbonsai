#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
use crate::{util::*, HTS_error};
extern "C" {
    fn sqrt(_: libc::c_double) -> libc::c_double;
}

use crate::{
    HTS_SStreamSet_get_duration, HTS_SStreamSet_get_gv_mean, HTS_SStreamSet_get_gv_switch,
    HTS_SStreamSet_get_gv_vari, HTS_SStreamSet_get_mean, HTS_SStreamSet_get_msd,
    HTS_SStreamSet_get_nstream, HTS_SStreamSet_get_total_frame, HTS_SStreamSet_get_total_state,
    HTS_SStreamSet_get_vari, HTS_SStreamSet_get_vector_length,
    HTS_SStreamSet_get_window_coefficient, HTS_SStreamSet_get_window_left_width,
    HTS_SStreamSet_get_window_max_width, HTS_SStreamSet_get_window_right_width,
    HTS_SStreamSet_get_window_size, HTS_SStreamSet_is_msd, HTS_SStreamSet_use_gv, HTS_alloc_matrix,
    HTS_calloc, HTS_free, HTS_free_matrix,
};

unsafe fn HTS_finv(x: libc::c_double) -> libc::c_double {
    if x >= 1.0e+19f64 {
        return 0.0f64;
    }
    if x <= -1.0e+19f64 {
        return 0.0f64;
    }
    if x <= 1.0e-19f64 && x >= 0 as libc::c_int as libc::c_double {
        return 1.0e+38f64;
    }
    if x >= -1.0e-19f64 && x < 0 as libc::c_int as libc::c_double {
        return -1.0e+38f64;
    }
    1.0f64 / x
}
unsafe fn HTS_PStream_calc_wuw_and_wum(mut pst: *mut HTS_PStream, mut m: size_t) {
    let mut t: size_t = 0;
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut shift: libc::c_int = 0;
    let mut wu: libc::c_double = 0.;
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        *((*pst).sm.wum).offset(t as isize) = 0.0f64;
        i = 0 as libc::c_int as size_t;
        while i < (*pst).width {
            *(*((*pst).sm.wuw).offset(t as isize)).offset(i as isize) = 0.0f64;
            i = i.wrapping_add(1);
        }
        i = 0 as libc::c_int as size_t;
        while i < (*pst).win_size {
            shift = *((*pst).win_l_width).offset(i as isize);
            while shift <= *((*pst).win_r_width).offset(i as isize) {
                if t as libc::c_int + shift >= 0 as libc::c_int
                    && ((t as libc::c_int + shift) as size_t) < (*pst).length
                    && *(*((*pst).win_coefficient).offset(i as isize)).offset(-shift as isize)
                        != 0.0f64
                {
                    wu = *(*((*pst).win_coefficient).offset(i as isize)).offset(-shift as isize)
                        * *(*((*pst).sm.ivar).offset(t.wrapping_add(shift as size_t) as isize))
                            .offset((i * (*pst).vector_length).wrapping_add(m) as isize);
                    *((*pst).sm.wum).offset(t as isize) += wu
                        * *(*((*pst).sm.mean).offset(t.wrapping_add(shift as size_t) as isize))
                            .offset((i * (*pst).vector_length).wrapping_add(m) as isize);
                    j = 0 as libc::c_int as size_t;
                    while j < (*pst).width && t.wrapping_add(j) < (*pst).length {
                        if j as libc::c_int <= *((*pst).win_r_width).offset(i as isize) + shift
                            && *(*((*pst).win_coefficient).offset(i as isize))
                                .offset(j.wrapping_sub(shift as size_t) as isize)
                                != 0.0f64
                        {
                            *(*((*pst).sm.wuw).offset(t as isize)).offset(j as isize) += wu
                                * *(*((*pst).win_coefficient).offset(i as isize))
                                    .offset(j.wrapping_sub(shift as size_t) as isize);
                        }
                        j = j.wrapping_add(1);
                    }
                }
                shift += 1;
            }
            i = i.wrapping_add(1);
        }
        t = t.wrapping_add(1);
    }
}
unsafe fn HTS_PStream_ldl_factorization(mut pst: *mut HTS_PStream) {
    let mut t: size_t = 0;
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        i = 1 as libc::c_int as size_t;
        while i < (*pst).width && t >= i {
            *(*((*pst).sm.wuw).offset(t as isize)).offset(0 as libc::c_int as isize) -=
                *(*((*pst).sm.wuw).offset(t.wrapping_sub(i) as isize)).offset(i as isize)
                    * *(*((*pst).sm.wuw).offset(t.wrapping_sub(i) as isize)).offset(i as isize)
                    * *(*((*pst).sm.wuw).offset(t.wrapping_sub(i) as isize))
                        .offset(0 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
        i = 1 as libc::c_int as size_t;
        while i < (*pst).width {
            j = 1 as libc::c_int as size_t;
            while i.wrapping_add(j) < (*pst).width && t >= j {
                *(*((*pst).sm.wuw).offset(t as isize)).offset(i as isize) -=
                    *(*((*pst).sm.wuw).offset(t.wrapping_sub(j) as isize)).offset(j as isize)
                        * *(*((*pst).sm.wuw).offset(t.wrapping_sub(j) as isize))
                            .offset(i.wrapping_add(j) as isize)
                        * *(*((*pst).sm.wuw).offset(t.wrapping_sub(j) as isize))
                            .offset(0 as libc::c_int as isize);
                j = j.wrapping_add(1);
            }
            *(*((*pst).sm.wuw).offset(t as isize)).offset(i as isize) /=
                *(*((*pst).sm.wuw).offset(t as isize)).offset(0 as libc::c_int as isize);
            i = i.wrapping_add(1);
        }
        t = t.wrapping_add(1);
    }
}
unsafe fn HTS_PStream_forward_substitution(mut pst: *mut HTS_PStream) {
    let mut t: size_t = 0;
    let mut i: size_t = 0;
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        *((*pst).sm.g).offset(t as isize) = *((*pst).sm.wum).offset(t as isize);
        i = 1 as libc::c_int as size_t;
        while i < (*pst).width && t >= i {
            *((*pst).sm.g).offset(t as isize) -=
                *(*((*pst).sm.wuw).offset(t.wrapping_sub(i) as isize)).offset(i as isize)
                    * *((*pst).sm.g).offset(t.wrapping_sub(i) as isize);
            i = i.wrapping_add(1);
        }
        t = t.wrapping_add(1);
    }
}
unsafe fn HTS_PStream_backward_substitution(mut pst: *mut HTS_PStream, mut m: size_t) {
    let mut rev: size_t = 0;
    let mut t: size_t = 0;
    let mut i: size_t = 0;
    rev = 0 as libc::c_int as size_t;
    while rev < (*pst).length {
        t = ((*pst).length)
            .wrapping_sub(1 as libc::c_int as size_t)
            .wrapping_sub(rev);
        *(*((*pst).par).offset(t as isize)).offset(m as isize) = *((*pst).sm.g).offset(t as isize)
            / *(*((*pst).sm.wuw).offset(t as isize)).offset(0 as libc::c_int as isize);
        i = 1 as libc::c_int as size_t;
        while i < (*pst).width && t.wrapping_add(i) < (*pst).length {
            *(*((*pst).par).offset(t as isize)).offset(m as isize) -=
                *(*((*pst).sm.wuw).offset(t as isize)).offset(i as isize)
                    * *(*((*pst).par).offset(t.wrapping_add(i) as isize)).offset(m as isize);
            i = i.wrapping_add(1);
        }
        rev = rev.wrapping_add(1);
    }
}
unsafe fn HTS_PStream_calc_gv(
    mut pst: *mut HTS_PStream,
    mut m: size_t,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
) {
    let mut t: size_t = 0;
    *mean = 0.0f64;
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        if *((*pst).gv_switch).offset(t as isize) != 0 {
            *mean += *(*((*pst).par).offset(t as isize)).offset(m as isize);
        }
        t = t.wrapping_add(1);
    }
    *mean /= (*pst).gv_length as libc::c_double;
    *vari = 0.0f64;
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        if *((*pst).gv_switch).offset(t as isize) != 0 {
            *vari += (*(*((*pst).par).offset(t as isize)).offset(m as isize) - *mean)
                * (*(*((*pst).par).offset(t as isize)).offset(m as isize) - *mean);
        }
        t = t.wrapping_add(1);
    }
    *vari /= (*pst).gv_length as libc::c_double;
}
unsafe fn HTS_PStream_conv_gv(mut pst: *mut HTS_PStream, mut m: size_t) {
    let mut t: size_t = 0;
    let mut ratio: libc::c_double = 0.;
    let mut mean: libc::c_double = 0.;
    let mut vari: libc::c_double = 0.;
    HTS_PStream_calc_gv(pst, m, &mut mean, &mut vari);
    ratio = sqrt(*((*pst).gv_mean).offset(m as isize) / vari);
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        if *((*pst).gv_switch).offset(t as isize) != 0 {
            *(*((*pst).par).offset(t as isize)).offset(m as isize) =
                ratio * (*(*((*pst).par).offset(t as isize)).offset(m as isize) - mean) + mean;
        }
        t = t.wrapping_add(1);
    }
}
unsafe fn HTS_PStream_calc_derivative(mut pst: *mut HTS_PStream, mut m: size_t) -> libc::c_double {
    let mut t: size_t = 0;
    let mut i: size_t = 0;
    let mut mean: libc::c_double = 0.;
    let mut vari: libc::c_double = 0.;
    let mut dv: libc::c_double = 0.;
    let mut h: libc::c_double = 0.;
    let mut gvobj: libc::c_double = 0.;
    let mut hmmobj: libc::c_double = 0.;
    let mut w: libc::c_double = 1.0f64 / ((*pst).win_size * (*pst).length) as libc::c_double;
    HTS_PStream_calc_gv(pst, m, &mut mean, &mut vari);
    gvobj = -0.5f64
        * 1.0f64
        * vari
        * *((*pst).gv_vari).offset(m as isize)
        * (vari - 2.0f64 * *((*pst).gv_mean).offset(m as isize));
    dv = -2.0f64
        * *((*pst).gv_vari).offset(m as isize)
        * (vari - *((*pst).gv_mean).offset(m as isize))
        / (*pst).length as libc::c_double;
    t = 0 as libc::c_int as size_t;
    while t < (*pst).length {
        *((*pst).sm.g).offset(t as isize) = *(*((*pst).sm.wuw).offset(t as isize))
            .offset(0 as libc::c_int as isize)
            * *(*((*pst).par).offset(t as isize)).offset(m as isize);
        i = 1 as libc::c_int as size_t;
        while i < (*pst).width {
            if t.wrapping_add(i) < (*pst).length {
                *((*pst).sm.g).offset(t as isize) += *(*((*pst).sm.wuw).offset(t as isize))
                    .offset(i as isize)
                    * *(*((*pst).par).offset(t.wrapping_add(i) as isize)).offset(m as isize);
            }
            if t.wrapping_add(1 as libc::c_int as size_t) > i {
                *((*pst).sm.g).offset(t as isize) +=
                    *(*((*pst).sm.wuw).offset(t.wrapping_sub(i) as isize)).offset(i as isize)
                        * *(*((*pst).par).offset(t.wrapping_sub(i) as isize)).offset(m as isize);
            }
            i = i.wrapping_add(1);
        }
        t = t.wrapping_add(1);
    }
    t = 0 as libc::c_int as size_t;
    hmmobj = 0.0f64;
    while t < (*pst).length {
        hmmobj += 1.0f64
            * w
            * *(*((*pst).par).offset(t as isize)).offset(m as isize)
            * (*((*pst).sm.wum).offset(t as isize) - 0.5f64 * *((*pst).sm.g).offset(t as isize));
        h = -1.0f64
            * w
            * *(*((*pst).sm.wuw).offset(t as isize))
                .offset((1 as libc::c_int - 1 as libc::c_int) as isize)
            - 1.0f64 * 2.0f64 / ((*pst).length * (*pst).length) as libc::c_double
                * (((*pst).length).wrapping_sub(1 as libc::c_int as size_t) as libc::c_double
                    * *((*pst).gv_vari).offset(m as isize)
                    * (vari - *((*pst).gv_mean).offset(m as isize))
                    + 2.0f64
                        * *((*pst).gv_vari).offset(m as isize)
                        * (*(*((*pst).par).offset(t as isize)).offset(m as isize) - mean)
                        * (*(*((*pst).par).offset(t as isize)).offset(m as isize) - mean));
        if *((*pst).gv_switch).offset(t as isize) != 0 {
            *((*pst).sm.g).offset(t as isize) = 1.0f64 / h
                * (1.0f64
                    * w
                    * (-*((*pst).sm.g).offset(t as isize) + *((*pst).sm.wum).offset(t as isize))
                    + 1.0f64
                        * dv
                        * (*(*((*pst).par).offset(t as isize)).offset(m as isize) - mean));
        } else {
            *((*pst).sm.g).offset(t as isize) = 1.0f64 / h
                * (1.0f64
                    * w
                    * (-*((*pst).sm.g).offset(t as isize) + *((*pst).sm.wum).offset(t as isize)));
        }
        t = t.wrapping_add(1);
    }
    -(hmmobj + gvobj)
}
unsafe fn HTS_PStream_gv_parmgen(mut pst: *mut HTS_PStream, mut m: size_t) {
    let mut t: size_t = 0;
    let mut i: size_t = 0;
    let mut step: libc::c_double = 0.1f64;
    let mut prev: libc::c_double = 0.0f64;
    let mut obj: libc::c_double = 0.;
    if (*pst).gv_length == 0 as libc::c_int as size_t {
        return;
    }
    HTS_PStream_conv_gv(pst, m);
    if 5 as libc::c_int > 0 as libc::c_int {
        HTS_PStream_calc_wuw_and_wum(pst, m);
        i = 1 as libc::c_int as size_t;
        while i <= 5 as libc::c_int as size_t {
            obj = HTS_PStream_calc_derivative(pst, m);
            if i > 1 as libc::c_int as size_t {
                if obj > prev {
                    step *= 0.5f64;
                }
                if obj < prev {
                    step *= 1.2f64;
                }
            }
            t = 0 as libc::c_int as size_t;
            while t < (*pst).length {
                *(*((*pst).par).offset(t as isize)).offset(m as isize) +=
                    step * *((*pst).sm.g).offset(t as isize);
                t = t.wrapping_add(1);
            }
            prev = obj;
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn HTS_PStream_mlpg(mut pst: *mut HTS_PStream) {
    let mut m: size_t = 0;
    if (*pst).length == 0 as libc::c_int as size_t {
        return;
    }
    m = 0 as libc::c_int as size_t;
    while m < (*pst).vector_length {
        HTS_PStream_calc_wuw_and_wum(pst, m);
        HTS_PStream_ldl_factorization(pst);
        HTS_PStream_forward_substitution(pst);
        HTS_PStream_backward_substitution(pst, m);
        if (*pst).gv_length > 0 as libc::c_int as size_t {
            HTS_PStream_gv_parmgen(pst, m);
        }
        m = m.wrapping_add(1);
    }
}

pub unsafe fn HTS_PStreamSet_initialize(mut pss: *mut HTS_PStreamSet) {
    (*pss).pstream = std::ptr::null_mut::<HTS_PStream>();
    (*pss).nstream = 0 as libc::c_int as size_t;
    (*pss).total_frame = 0 as libc::c_int as size_t;
}

pub unsafe fn HTS_PStreamSet_create(
    mut pss: *mut HTS_PStreamSet,
    mut sss: *mut HTS_SStreamSet,
    mut msd_threshold: *mut libc::c_double,
    mut gv_weight: *mut libc::c_double,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut l: size_t = 0;
    let mut m: size_t = 0;
    let mut shift: libc::c_int = 0;
    let mut frame: size_t = 0;
    let mut msd_frame: size_t = 0;
    let mut state: size_t = 0;
    let mut pst: *mut HTS_PStream = std::ptr::null_mut::<HTS_PStream>();
    let mut not_bound: HTS_Boolean = 0;
    if (*pss).nstream != 0 as libc::c_int as size_t {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_PstreamSet_create: HTS_PStreamSet should be clear.\n\0" as *const u8
                as *const libc::c_char,
        );
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*pss).nstream = HTS_SStreamSet_get_nstream(sss);
    (*pss).pstream = HTS_calloc(
        (*pss).nstream,
        ::core::mem::size_of::<HTS_PStream>() as libc::c_ulong,
    ) as *mut HTS_PStream;
    (*pss).total_frame = HTS_SStreamSet_get_total_frame(sss);
    i = 0 as libc::c_int as size_t;
    while i < (*pss).nstream {
        pst = &mut *((*pss).pstream).offset(i as isize) as *mut HTS_PStream;
        if HTS_SStreamSet_is_msd(sss, i) != 0 {
            (*pst).length = 0 as libc::c_int as size_t;
            state = 0 as libc::c_int as size_t;
            while state < HTS_SStreamSet_get_total_state(sss) {
                if HTS_SStreamSet_get_msd(sss, i, state) > *msd_threshold.offset(i as isize) {
                    (*pst).length =
                        ((*pst).length).wrapping_add(HTS_SStreamSet_get_duration(sss, state));
                }
                state = state.wrapping_add(1);
            }
            (*pst).msd_flag = HTS_calloc(
                (*pss).total_frame,
                ::core::mem::size_of::<HTS_Boolean>() as libc::c_ulong,
            ) as *mut HTS_Boolean;
            state = 0 as libc::c_int as size_t;
            frame = 0 as libc::c_int as size_t;
            while state < HTS_SStreamSet_get_total_state(sss) {
                if HTS_SStreamSet_get_msd(sss, i, state) > *msd_threshold.offset(i as isize) {
                    j = 0 as libc::c_int as size_t;
                    while j < HTS_SStreamSet_get_duration(sss, state) {
                        *((*pst).msd_flag).offset(frame as isize) = 1 as libc::c_int as HTS_Boolean;
                        frame = frame.wrapping_add(1);
                        j = j.wrapping_add(1);
                    }
                } else {
                    j = 0 as libc::c_int as size_t;
                    while j < HTS_SStreamSet_get_duration(sss, state) {
                        *((*pst).msd_flag).offset(frame as isize) = 0 as libc::c_int as HTS_Boolean;
                        frame = frame.wrapping_add(1);
                        j = j.wrapping_add(1);
                    }
                }
                state = state.wrapping_add(1);
            }
        } else {
            (*pst).length = (*pss).total_frame;
            (*pst).msd_flag = std::ptr::null_mut::<HTS_Boolean>();
        }
        (*pst).vector_length = HTS_SStreamSet_get_vector_length(sss, i);
        (*pst).width = (HTS_SStreamSet_get_window_max_width(sss, i) * 2 as libc::c_int as size_t)
            .wrapping_add(1 as libc::c_int as size_t);
        (*pst).win_size = HTS_SStreamSet_get_window_size(sss, i);
        if (*pst).length > 0 as libc::c_int as size_t {
            (*pst).sm.mean =
                HTS_alloc_matrix((*pst).length, (*pst).vector_length * (*pst).win_size);
            (*pst).sm.ivar =
                HTS_alloc_matrix((*pst).length, (*pst).vector_length * (*pst).win_size);
            (*pst).sm.wum = HTS_calloc(
                (*pst).length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            (*pst).sm.wuw = HTS_alloc_matrix((*pst).length, (*pst).width);
            (*pst).sm.g = HTS_calloc(
                (*pst).length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            (*pst).par = HTS_alloc_matrix((*pst).length, (*pst).vector_length);
        }
        (*pst).win_l_width = HTS_calloc(
            (*pst).win_size,
            ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ) as *mut libc::c_int;
        (*pst).win_r_width = HTS_calloc(
            (*pst).win_size,
            ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ) as *mut libc::c_int;
        (*pst).win_coefficient = HTS_calloc(
            (*pst).win_size,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < (*pst).win_size {
            *((*pst).win_l_width).offset(j as isize) =
                HTS_SStreamSet_get_window_left_width(sss, i, j);
            *((*pst).win_r_width).offset(j as isize) =
                HTS_SStreamSet_get_window_right_width(sss, i, j);
            if *((*pst).win_l_width).offset(j as isize) + *((*pst).win_r_width).offset(j as isize)
                == 0 as libc::c_int
            {
                let fresh0 = &mut (*((*pst).win_coefficient).offset(j as isize));
                *fresh0 = HTS_calloc(
                    (-(2 as libc::c_int) * *((*pst).win_l_width).offset(j as isize)
                        + 1 as libc::c_int) as size_t,
                    ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
                ) as *mut libc::c_double;
            } else {
                let fresh1 = &mut (*((*pst).win_coefficient).offset(j as isize));
                *fresh1 = HTS_calloc(
                    (-(2 as libc::c_int) * *((*pst).win_l_width).offset(j as isize)) as size_t,
                    ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
                ) as *mut libc::c_double;
            }
            let fresh2 = &mut (*((*pst).win_coefficient).offset(j as isize));
            *fresh2 = (*fresh2).offset(-(*((*pst).win_l_width).offset(j as isize) as isize));
            shift = *((*pst).win_l_width).offset(j as isize);
            while shift <= *((*pst).win_r_width).offset(j as isize) {
                *(*((*pst).win_coefficient).offset(j as isize)).offset(shift as isize) =
                    HTS_SStreamSet_get_window_coefficient(sss, i, j, shift);
                shift += 1;
            }
            j = j.wrapping_add(1);
        }
        if HTS_SStreamSet_use_gv(sss, i) != 0 {
            (*pst).gv_mean = HTS_calloc(
                (*pst).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            (*pst).gv_vari = HTS_calloc(
                (*pst).vector_length,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            j = 0 as libc::c_int as size_t;
            while j < (*pst).vector_length {
                *((*pst).gv_mean).offset(j as isize) =
                    HTS_SStreamSet_get_gv_mean(sss, i, j) * *gv_weight.offset(i as isize);
                *((*pst).gv_vari).offset(j as isize) = HTS_SStreamSet_get_gv_vari(sss, i, j);
                j = j.wrapping_add(1);
            }
            (*pst).gv_switch = HTS_calloc(
                (*pst).length,
                ::core::mem::size_of::<HTS_Boolean>() as libc::c_ulong,
            ) as *mut HTS_Boolean;
            if HTS_SStreamSet_is_msd(sss, i) != 0 {
                state = 0 as libc::c_int as size_t;
                frame = 0 as libc::c_int as size_t;
                msd_frame = 0 as libc::c_int as size_t;
                while state < HTS_SStreamSet_get_total_state(sss) {
                    j = 0 as libc::c_int as size_t;
                    while j < HTS_SStreamSet_get_duration(sss, state) {
                        if *((*pst).msd_flag).offset(frame as isize) != 0 {
                            let fresh3 = msd_frame;
                            msd_frame = msd_frame.wrapping_add(1);
                            *((*pst).gv_switch).offset(fresh3 as isize) =
                                HTS_SStreamSet_get_gv_switch(sss, i, state);
                        }
                        j = j.wrapping_add(1);
                        frame = frame.wrapping_add(1);
                    }
                    state = state.wrapping_add(1);
                }
            } else {
                state = 0 as libc::c_int as size_t;
                frame = 0 as libc::c_int as size_t;
                while state < HTS_SStreamSet_get_total_state(sss) {
                    j = 0 as libc::c_int as size_t;
                    while j < HTS_SStreamSet_get_duration(sss, state) {
                        let fresh4 = frame;
                        frame = frame.wrapping_add(1);
                        *((*pst).gv_switch).offset(fresh4 as isize) =
                            HTS_SStreamSet_get_gv_switch(sss, i, state);
                        j = j.wrapping_add(1);
                    }
                    state = state.wrapping_add(1);
                }
            }
            j = 0 as libc::c_int as size_t;
            (*pst).gv_length = 0 as libc::c_int as size_t;
            while j < (*pst).length {
                if *((*pst).gv_switch).offset(j as isize) != 0 {
                    (*pst).gv_length = ((*pst).gv_length).wrapping_add(1);
                    (*pst).gv_length;
                }
                j = j.wrapping_add(1);
            }
        } else {
            (*pst).gv_switch = std::ptr::null_mut::<HTS_Boolean>();
            (*pst).gv_length = 0 as libc::c_int as size_t;
            (*pst).gv_mean = std::ptr::null_mut::<libc::c_double>();
            (*pst).gv_vari = std::ptr::null_mut::<libc::c_double>();
        }
        if HTS_SStreamSet_is_msd(sss, i) != 0 {
            state = 0 as libc::c_int as size_t;
            frame = 0 as libc::c_int as size_t;
            msd_frame = 0 as libc::c_int as size_t;
            while state < HTS_SStreamSet_get_total_state(sss) {
                j = 0 as libc::c_int as size_t;
                while j < HTS_SStreamSet_get_duration(sss, state) {
                    if *((*pst).msd_flag).offset(frame as isize) != 0 {
                        k = 0 as libc::c_int as size_t;
                        while k < (*pst).win_size {
                            not_bound = 1 as libc::c_int as HTS_Boolean;
                            shift = *((*pst).win_l_width).offset(k as isize);
                            while shift <= *((*pst).win_r_width).offset(k as isize) {
                                if frame as libc::c_int + shift < 0 as libc::c_int
                                    || (*pss).total_frame as libc::c_int
                                        <= frame as libc::c_int + shift
                                    || *((*pst).msd_flag)
                                        .offset(frame.wrapping_add(shift as size_t) as isize)
                                        == 0
                                {
                                    not_bound = 0 as libc::c_int as HTS_Boolean;
                                    break;
                                } else {
                                    shift += 1;
                                }
                            }
                            l = 0 as libc::c_int as size_t;
                            while l < (*pst).vector_length {
                                m = ((*pst).vector_length * k).wrapping_add(l);
                                *(*((*pst).sm.mean).offset(msd_frame as isize))
                                    .offset(m as isize) = HTS_SStreamSet_get_mean(sss, i, state, m);
                                if not_bound as libc::c_int != 0 || k == 0 as libc::c_int as size_t
                                {
                                    *(*((*pst).sm.ivar).offset(msd_frame as isize))
                                        .offset(m as isize) =
                                        HTS_finv(HTS_SStreamSet_get_vari(sss, i, state, m));
                                } else {
                                    *(*((*pst).sm.ivar).offset(msd_frame as isize))
                                        .offset(m as isize) = 0.0f64;
                                }
                                l = l.wrapping_add(1);
                            }
                            k = k.wrapping_add(1);
                        }
                        msd_frame = msd_frame.wrapping_add(1);
                    }
                    frame = frame.wrapping_add(1);
                    j = j.wrapping_add(1);
                }
                state = state.wrapping_add(1);
            }
        } else {
            state = 0 as libc::c_int as size_t;
            frame = 0 as libc::c_int as size_t;
            while state < HTS_SStreamSet_get_total_state(sss) {
                j = 0 as libc::c_int as size_t;
                while j < HTS_SStreamSet_get_duration(sss, state) {
                    k = 0 as libc::c_int as size_t;
                    while k < (*pst).win_size {
                        not_bound = 1 as libc::c_int as HTS_Boolean;
                        shift = *((*pst).win_l_width).offset(k as isize);
                        while shift <= *((*pst).win_r_width).offset(k as isize) {
                            if frame as libc::c_int + shift < 0 as libc::c_int
                                || (*pss).total_frame as libc::c_int <= frame as libc::c_int + shift
                            {
                                not_bound = 0 as libc::c_int as HTS_Boolean;
                                break;
                            } else {
                                shift += 1;
                            }
                        }
                        l = 0 as libc::c_int as size_t;
                        while l < (*pst).vector_length {
                            m = ((*pst).vector_length * k).wrapping_add(l);
                            *(*((*pst).sm.mean).offset(frame as isize)).offset(m as isize) =
                                HTS_SStreamSet_get_mean(sss, i, state, m);
                            if not_bound as libc::c_int != 0 || k == 0 as libc::c_int as size_t {
                                *(*((*pst).sm.ivar).offset(frame as isize)).offset(m as isize) =
                                    HTS_finv(HTS_SStreamSet_get_vari(sss, i, state, m));
                            } else {
                                *(*((*pst).sm.ivar).offset(frame as isize)).offset(m as isize) =
                                    0.0f64;
                            }
                            l = l.wrapping_add(1);
                        }
                        k = k.wrapping_add(1);
                    }
                    frame = frame.wrapping_add(1);
                    j = j.wrapping_add(1);
                }
                state = state.wrapping_add(1);
            }
        }
        HTS_PStream_mlpg(pst);
        i = i.wrapping_add(1);
    }
    1 as libc::c_int as HTS_Boolean
}

pub unsafe fn HTS_PStreamSet_get_nstream(mut pss: *mut HTS_PStreamSet) -> size_t {
    (*pss).nstream
}

pub unsafe fn HTS_PStreamSet_get_vector_length(
    mut pss: *mut HTS_PStreamSet,
    mut stream_index: size_t,
) -> size_t {
    (*((*pss).pstream).offset(stream_index as isize)).vector_length
}

pub unsafe fn HTS_PStreamSet_get_total_frame(mut pss: *mut HTS_PStreamSet) -> size_t {
    (*pss).total_frame
}

pub unsafe fn HTS_PStreamSet_get_parameter(
    mut pss: *mut HTS_PStreamSet,
    mut stream_index: size_t,
    mut frame_index: size_t,
    mut vector_index: size_t,
) -> libc::c_double {
    *(*((*((*pss).pstream).offset(stream_index as isize)).par).offset(frame_index as isize))
        .offset(vector_index as isize)
}

pub unsafe fn HTS_PStreamSet_get_parameter_vector(
    mut pss: *mut HTS_PStreamSet,
    mut stream_index: size_t,
    mut frame_index: size_t,
) -> *mut libc::c_double {
    *((*((*pss).pstream).offset(stream_index as isize)).par).offset(frame_index as isize)
}

pub unsafe fn HTS_PStreamSet_get_msd_flag(
    mut pss: *mut HTS_PStreamSet,
    mut stream_index: size_t,
    mut frame_index: size_t,
) -> HTS_Boolean {
    *((*((*pss).pstream).offset(stream_index as isize)).msd_flag).offset(frame_index as isize)
}

pub unsafe fn HTS_PStreamSet_is_msd(
    mut pss: *mut HTS_PStreamSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    (if !((*((*pss).pstream).offset(stream_index as isize)).msd_flag).is_null() {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    }) as HTS_Boolean
}

pub unsafe fn HTS_PStreamSet_clear(mut pss: *mut HTS_PStreamSet) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut pstream: *mut HTS_PStream = std::ptr::null_mut::<HTS_PStream>();
    if !((*pss).pstream).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*pss).nstream {
            pstream = &mut *((*pss).pstream).offset(i as isize) as *mut HTS_PStream;
            if !((*pstream).sm.wum).is_null() {
                HTS_free((*pstream).sm.wum as *mut libc::c_void);
            }
            if !((*pstream).sm.g).is_null() {
                HTS_free((*pstream).sm.g as *mut libc::c_void);
            }
            if !((*pstream).sm.wuw).is_null() {
                HTS_free_matrix((*pstream).sm.wuw, (*pstream).length);
            }
            if !((*pstream).sm.ivar).is_null() {
                HTS_free_matrix((*pstream).sm.ivar, (*pstream).length);
            }
            if !((*pstream).sm.mean).is_null() {
                HTS_free_matrix((*pstream).sm.mean, (*pstream).length);
            }
            if !((*pstream).par).is_null() {
                HTS_free_matrix((*pstream).par, (*pstream).length);
            }
            if !((*pstream).msd_flag).is_null() {
                HTS_free((*pstream).msd_flag as *mut libc::c_void);
            }
            if !((*pstream).win_coefficient).is_null() {
                j = 0 as libc::c_int as size_t;
                while j < (*pstream).win_size {
                    let fresh5 = &mut (*((*pstream).win_coefficient).offset(j as isize));
                    *fresh5 =
                        (*fresh5).offset(*((*pstream).win_l_width).offset(j as isize) as isize);
                    HTS_free(*((*pstream).win_coefficient).offset(j as isize) as *mut libc::c_void);
                    j = j.wrapping_add(1);
                }
            }
            if !((*pstream).gv_mean).is_null() {
                HTS_free((*pstream).gv_mean as *mut libc::c_void);
            }
            if !((*pstream).gv_vari).is_null() {
                HTS_free((*pstream).gv_vari as *mut libc::c_void);
            }
            if !((*pstream).win_coefficient).is_null() {
                HTS_free((*pstream).win_coefficient as *mut libc::c_void);
            }
            if !((*pstream).win_l_width).is_null() {
                HTS_free((*pstream).win_l_width as *mut libc::c_void);
            }
            if !((*pstream).win_r_width).is_null() {
                HTS_free((*pstream).win_r_width as *mut libc::c_void);
            }
            if !((*pstream).gv_switch).is_null() {
                HTS_free((*pstream).gv_switch as *mut libc::c_void);
            }
            i = i.wrapping_add(1);
        }
        HTS_free((*pss).pstream as *mut libc::c_void);
    }
    HTS_PStreamSet_initialize(pss);
}
