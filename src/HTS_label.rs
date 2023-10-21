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
    fn atof(__nptr: *const libc::c_char) -> libc::c_double;
    fn __ctype_b_loc() -> *mut *const libc::c_ushort;
    fn sscanf(_: *const libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
}

use crate::{
    HTS_calloc, HTS_fclose, HTS_fopen_from_fn, HTS_free, HTS_get_token_from_fp,
    HTS_get_token_from_string, HTS_strdup,
};

pub type C2RustUnnamed = libc::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;

unsafe extern "C" fn isdigit_string(mut str: *mut libc::c_char) -> HTS_Boolean {
    let mut i: libc::c_int = 0;
    if sscanf(
        str,
        b"%d\0" as *const u8 as *const libc::c_char,
        &mut i as *mut libc::c_int,
    ) == 1 as libc::c_int
    {
        1 as libc::c_int as HTS_Boolean
    } else {
        0 as libc::c_int as HTS_Boolean
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_initialize(mut label: *mut HTS_Label) {
    (*label).head = std::ptr::null_mut::<HTS_LabelString>();
    (*label).size = 0 as libc::c_int as size_t;
}
unsafe extern "C" fn HTS_Label_check_time(mut label: *mut HTS_Label) {
    let mut lstring: *mut HTS_LabelString = (*label).head;
    let mut next: *mut HTS_LabelString = std::ptr::null_mut::<HTS_LabelString>();
    if !lstring.is_null() {
        (*lstring).start = 0.0f64;
    }
    while !lstring.is_null() {
        next = (*lstring).next;
        if next.is_null() {
            break;
        }
        if (*lstring).end < 0.0f64 && (*next).start >= 0.0f64 {
            (*lstring).end = (*next).start;
        } else if (*lstring).end >= 0.0f64 && (*next).start < 0.0f64 {
            (*next).start = (*lstring).end;
        }
        if (*lstring).start < 0.0f64 {
            (*lstring).start = -1.0f64;
        }
        if (*lstring).end < 0.0f64 {
            (*lstring).end = -1.0f64;
        }
        lstring = next;
    }
}
unsafe extern "C" fn HTS_Label_load(
    mut label: *mut HTS_Label,
    mut sampling_rate: size_t,
    mut fperiod: size_t,
    mut fp: *mut HTS_File,
) {
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut lstring: *mut HTS_LabelString = std::ptr::null_mut::<HTS_LabelString>();
    let mut start: libc::c_double = 0.;
    let mut end: libc::c_double = 0.;
    let rate: libc::c_double =
        sampling_rate as libc::c_double / (fperiod as libc::c_double * 1e+7f64);
    if !((*label).head).is_null() || (*label).size != 0 as libc::c_int as size_t {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_Label_load_from_fp: label is not initialized.\n\0" as *const u8
                as *const libc::c_char,
        );
        return;
    }
    while HTS_get_token_from_fp(fp, buff.as_mut_ptr()) != 0 {
        if *(*__ctype_b_loc()).offset(buff[0 as libc::c_int as usize] as libc::c_int as isize)
            as libc::c_int
            & _ISgraph as libc::c_int as libc::c_ushort as libc::c_int
            == 0
        {
            break;
        }
        (*label).size = ((*label).size).wrapping_add(1);
        (*label).size;
        if !lstring.is_null() {
            (*lstring).next = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_LabelString>() as libc::c_ulong,
            ) as *mut HTS_LabelString;
            lstring = (*lstring).next;
        } else {
            lstring = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_LabelString>() as libc::c_ulong,
            ) as *mut HTS_LabelString;
            (*label).head = lstring;
        }
        if isdigit_string(buff.as_mut_ptr()) != 0 {
            start = atof(buff.as_mut_ptr());
            HTS_get_token_from_fp(fp, buff.as_mut_ptr());
            end = atof(buff.as_mut_ptr());
            HTS_get_token_from_fp(fp, buff.as_mut_ptr());
            (*lstring).start = rate * start;
            (*lstring).end = rate * end;
        } else {
            (*lstring).start = -1.0f64;
            (*lstring).end = -1.0f64;
        }
        (*lstring).next = std::ptr::null_mut::<HTS_LabelString>();
        (*lstring).name = HTS_strdup(buff.as_mut_ptr());
    }
    HTS_Label_check_time(label);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_load_from_fn(
    mut label: *mut HTS_Label,
    mut sampling_rate: size_t,
    mut fperiod: size_t,
    mut fn_0: *const libc::c_char,
) {
    let mut fp: *mut HTS_File = HTS_fopen_from_fn(fn_0, b"r\0" as *const u8 as *const libc::c_char);
    HTS_Label_load(label, sampling_rate, fperiod, fp);
    HTS_fclose(fp);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_load_from_strings(
    mut label: *mut HTS_Label,
    mut sampling_rate: size_t,
    mut fperiod: size_t,
    mut lines: *mut *mut libc::c_char,
    mut num_lines: size_t,
) {
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut lstring: *mut HTS_LabelString = std::ptr::null_mut::<HTS_LabelString>();
    let mut i: size_t = 0;
    let mut data_index: size_t = 0;
    let mut start: libc::c_double = 0.;
    let mut end: libc::c_double = 0.;
    let rate: libc::c_double =
        sampling_rate as libc::c_double / (fperiod as libc::c_double * 1e+7f64);
    if !((*label).head).is_null() || (*label).size != 0 as libc::c_int as size_t {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_Label_load_from_fp: label list is not initialized.\n\0" as *const u8
                as *const libc::c_char,
        );
        return;
    }
    i = 0 as libc::c_int as size_t;
    while i < num_lines {
        if *(*__ctype_b_loc()).offset(
            *(*lines.offset(i as isize)).offset(0 as libc::c_int as isize) as libc::c_int as isize,
        ) as libc::c_int
            & _ISgraph as libc::c_int as libc::c_ushort as libc::c_int
            == 0
        {
            break;
        }
        (*label).size = ((*label).size).wrapping_add(1);
        (*label).size;
        if !lstring.is_null() {
            (*lstring).next = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_LabelString>() as libc::c_ulong,
            ) as *mut HTS_LabelString;
            lstring = (*lstring).next;
        } else {
            lstring = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_LabelString>() as libc::c_ulong,
            ) as *mut HTS_LabelString;
            (*label).head = lstring;
        }
        data_index = 0 as libc::c_int as size_t;
        if isdigit_string(*lines.offset(i as isize)) != 0 {
            HTS_get_token_from_string(
                *lines.offset(i as isize),
                &mut data_index,
                buff.as_mut_ptr(),
            );
            start = atof(buff.as_mut_ptr());
            HTS_get_token_from_string(
                *lines.offset(i as isize),
                &mut data_index,
                buff.as_mut_ptr(),
            );
            end = atof(buff.as_mut_ptr());
            HTS_get_token_from_string(
                *lines.offset(i as isize),
                &mut data_index,
                buff.as_mut_ptr(),
            );
            (*lstring).name = HTS_strdup(buff.as_mut_ptr());
            (*lstring).start = rate * start;
            (*lstring).end = rate * end;
        } else {
            (*lstring).start = -1.0f64;
            (*lstring).end = -1.0f64;
            (*lstring).name = HTS_strdup(*lines.offset(i as isize));
        }
        (*lstring).next = std::ptr::null_mut::<HTS_LabelString>();
        i = i.wrapping_add(1);
    }
    HTS_Label_check_time(label);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_get_size(mut label: *mut HTS_Label) -> size_t {
    (*label).size
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_get_string(
    mut label: *mut HTS_Label,
    mut index: size_t,
) -> *const libc::c_char {
    let mut i: size_t = 0;
    let mut lstring: *mut HTS_LabelString = (*label).head;
    i = 0 as libc::c_int as size_t;
    while i < index && !lstring.is_null() {
        lstring = (*lstring).next;
        i = i.wrapping_add(1);
    }
    if lstring.is_null() {
        return std::ptr::null::<libc::c_char>();
    }
    (*lstring).name
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_get_start_frame(
    mut label: *mut HTS_Label,
    mut index: size_t,
) -> libc::c_double {
    let mut i: size_t = 0;
    let mut lstring: *mut HTS_LabelString = (*label).head;
    i = 0 as libc::c_int as size_t;
    while i < index && !lstring.is_null() {
        lstring = (*lstring).next;
        i = i.wrapping_add(1);
    }
    if lstring.is_null() {
        return -1.0f64;
    }
    (*lstring).start
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_get_end_frame(
    mut label: *mut HTS_Label,
    mut index: size_t,
) -> libc::c_double {
    let mut i: size_t = 0;
    let mut lstring: *mut HTS_LabelString = (*label).head;
    i = 0 as libc::c_int as size_t;
    while i < index && !lstring.is_null() {
        lstring = (*lstring).next;
        i = i.wrapping_add(1);
    }
    if lstring.is_null() {
        return -1.0f64;
    }
    (*lstring).end
}
#[no_mangle]
pub unsafe extern "C" fn HTS_Label_clear(mut label: *mut HTS_Label) {
    let mut lstring: *mut HTS_LabelString = std::ptr::null_mut::<HTS_LabelString>();
    let mut next_lstring: *mut HTS_LabelString = std::ptr::null_mut::<HTS_LabelString>();
    lstring = (*label).head;
    while !lstring.is_null() {
        next_lstring = (*lstring).next;
        HTS_free((*lstring).name as *mut libc::c_void);
        HTS_free(lstring as *mut libc::c_void);
        lstring = next_lstring;
    }
    HTS_Label_initialize(label);
}
