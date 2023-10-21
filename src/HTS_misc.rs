use libc::FILE;

use crate::util::*;
use crate::HTS_error;

extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn exit(_: libc::c_int) -> !;
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn fgetc(__stream: *mut FILE) -> libc::c_int;
    fn fread(
        _: *mut libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
    fn fwrite(
        _: *const libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int) -> libc::c_int;
    fn fgetpos(__stream: *mut FILE, __pos: *mut fpos_t) -> libc::c_int;
    fn feof(__stream: *mut FILE) -> libc::c_int;
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
    fn fflush(__stream: *mut FILE) -> libc::c_int;
    fn fclose(__stream: *mut FILE) -> libc::c_int;
    static mut stderr: *mut FILE;
    static mut stdout: *mut FILE;
    // fn vfprintf(
    //     _: *mut FILE,
    //     _: *const libc::c_char,
    //     _: ::core::ffi::VaList,
    // ) -> libc::c_int;
    fn fprintf(_: *mut FILE, _: *const libc::c_char, _: ...) -> libc::c_int;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: libc::c_uint,
    pub fp_offset: libc::c_uint,
    pub overflow_arg_area: *mut libc::c_void,
    pub reg_save_area: *mut libc::c_void,
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __mbstate_t {
    pub __count: libc::c_int,
    pub __value: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub __wch: libc::c_uint,
    pub __wchb: [libc::c_char; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _G_fpos_t {
    pub __pos: __off_t,
    pub __state: __mbstate_t,
}
pub type __fpos_t = _G_fpos_t;
pub type _IO_lock_t = ();
pub type fpos_t = __fpos_t;
pub type HTS_Data = _HTS_Data;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _HTS_Data {
    pub data: *mut libc::c_uchar,
    pub size: size_t,
    pub index: size_t,
}

#[no_mangle]
pub unsafe extern "C" fn HTS_fopen_from_fn(
    name: *const libc::c_char,
    opt: *const libc::c_char,
) -> *mut HTS_File {
    let fp: *mut HTS_File = HTS_calloc(
        1 as libc::c_int as size_t,
        ::core::mem::size_of::<HTS_File>() as libc::c_ulong,
    ) as *mut HTS_File;
    (*fp).type_0 = 0 as libc::c_int as libc::c_uchar;
    (*fp).pointer = fopen(name, opt) as *mut libc::c_void;
    if ((*fp).pointer).is_null() {
        HTS_error!(
            0 as libc::c_int,
            b"HTS_fopen: Cannot open %s.\n\0" as *const u8 as *const libc::c_char,
            name,
        );
        HTS_free(fp as *mut libc::c_void);
        return std::ptr::null_mut::<HTS_File>();
    }
    fp
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fopen_from_fp(
    fp: *mut HTS_File,
    size: size_t,
) -> *mut HTS_File {
    if fp.is_null() || size == 0 as libc::c_int as size_t {
        return std::ptr::null_mut::<HTS_File>();
    } else if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        let mut d: *mut HTS_Data = std::ptr::null_mut::<HTS_Data>();
        let mut f: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
        d = HTS_calloc(
            1 as libc::c_int as size_t,
            ::core::mem::size_of::<HTS_Data>() as libc::c_ulong,
        ) as *mut HTS_Data;
        (*d).data = HTS_calloc(
            size,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong,
        ) as *mut libc::c_uchar;
        (*d).size = size;
        (*d).index = 0 as libc::c_int as size_t;
        if fread(
            (*d).data as *mut libc::c_void,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong,
            size,
            (*fp).pointer as *mut FILE,
        ) != size
        {
            free((*d).data as *mut libc::c_void);
            free(d as *mut libc::c_void);
            return std::ptr::null_mut::<HTS_File>();
        }
        f = HTS_calloc(
            1 as libc::c_int as size_t,
            ::core::mem::size_of::<HTS_File>() as libc::c_ulong,
        ) as *mut HTS_File;
        (*f).type_0 = 1 as libc::c_int as libc::c_uchar;
        (*f).pointer = d as *mut libc::c_void;
        return f;
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        let mut f_0: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
        let mut tmp1: *mut HTS_Data = std::ptr::null_mut::<HTS_Data>();
        let mut tmp2: *mut HTS_Data = std::ptr::null_mut::<HTS_Data>();
        tmp1 = (*fp).pointer as *mut HTS_Data;
        if ((*tmp1).index).wrapping_add(size) > (*tmp1).size {
            return std::ptr::null_mut::<HTS_File>();
        }
        tmp2 = HTS_calloc(
            1 as libc::c_int as size_t,
            ::core::mem::size_of::<HTS_Data>() as libc::c_ulong,
        ) as *mut HTS_Data;
        (*tmp2).data = HTS_calloc(
            size,
            ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong,
        ) as *mut libc::c_uchar;
        (*tmp2).size = size;
        (*tmp2).index = 0 as libc::c_int as size_t;
        memcpy(
            (*tmp2).data as *mut libc::c_void,
            &mut *((*tmp1).data).offset((*tmp1).index as isize) as *mut libc::c_uchar
                as *const libc::c_void,
            size,
        );
        (*tmp1).index = ((*tmp1).index).wrapping_add(size);
        f_0 = HTS_calloc(
            1 as libc::c_int as size_t,
            ::core::mem::size_of::<HTS_File>() as libc::c_ulong,
        ) as *mut HTS_File;
        (*f_0).type_0 = 1 as libc::c_int as libc::c_uchar;
        (*f_0).pointer = tmp2 as *mut libc::c_void;
        return f_0;
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_fopen_from_fp: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
    std::ptr::null_mut::<HTS_File>()
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fopen_from_data(
    data: *mut libc::c_void,
    size: size_t,
) -> *mut HTS_File {
    let mut d: *mut HTS_Data = std::ptr::null_mut::<HTS_Data>();
    let mut f: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
    if data.is_null() || size == 0 as libc::c_int as size_t {
        return std::ptr::null_mut::<HTS_File>();
    }
    d = HTS_calloc(
        1 as libc::c_int as size_t,
        ::core::mem::size_of::<HTS_Data>() as libc::c_ulong,
    ) as *mut HTS_Data;
    (*d).data = HTS_calloc(
        size,
        ::core::mem::size_of::<libc::c_uchar>() as libc::c_ulong,
    ) as *mut libc::c_uchar;
    (*d).size = size;
    (*d).index = 0 as libc::c_int as size_t;
    memcpy((*d).data as *mut libc::c_void, data, size);
    f = HTS_calloc(
        1 as libc::c_int as size_t,
        ::core::mem::size_of::<HTS_File>() as libc::c_ulong,
    ) as *mut HTS_File;
    (*f).type_0 = 1 as libc::c_int as libc::c_uchar;
    (*f).pointer = d as *mut libc::c_void;
    f
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fclose(fp: *mut HTS_File) {
    if fp.is_null() {
    } else if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        if !((*fp).pointer).is_null() {
            fclose((*fp).pointer as *mut FILE);
        }
        HTS_free(fp as *mut libc::c_void);
        return;
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        if !((*fp).pointer).is_null() {
            let d: *mut HTS_Data = (*fp).pointer as *mut HTS_Data;
            if !((*d).data).is_null() {
                HTS_free((*d).data as *mut libc::c_void);
            }
            HTS_free(d as *mut libc::c_void);
        }
        HTS_free(fp as *mut libc::c_void);
        return;
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_fclose: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fgetc(fp: *mut HTS_File) -> libc::c_int {
    if fp.is_null() {
        return -(1 as libc::c_int);
    } else if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        return fgetc((*fp).pointer as *mut FILE);
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        let d: *mut HTS_Data = (*fp).pointer as *mut HTS_Data;
        if (*d).size <= (*d).index {
            return -(1 as libc::c_int);
        }
        let fresh0 = (*d).index;
        (*d).index = ((*d).index).wrapping_add(1);
        return *((*d).data).offset(fresh0 as isize) as libc::c_int;
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_fgetc: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
    -(1 as libc::c_int)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_feof(fp: *mut HTS_File) -> libc::c_int {
    if fp.is_null() {
        return 1 as libc::c_int;
    } else if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        return feof((*fp).pointer as *mut FILE);
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        let d: *mut HTS_Data = (*fp).pointer as *mut HTS_Data;
        return if (*d).size <= (*d).index {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        };
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_feof: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
    1 as libc::c_int
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fseek(
    fp: *mut HTS_File,
    offset: libc::c_long,
    origin: libc::c_int,
) -> libc::c_int {
    if fp.is_null() {
        return 1 as libc::c_int;
    } else if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        return fseek((*fp).pointer as *mut FILE, offset, origin);
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        let d: *mut HTS_Data = (*fp).pointer as *mut HTS_Data;
        if origin == 0 as libc::c_int {
            (*d).index = offset as size_t;
        } else if origin == 1 as libc::c_int {
            (*d).index = ((*d).index).wrapping_add(offset as size_t);
        } else if origin == 2 as libc::c_int {
            (*d).index = ((*d).size).wrapping_add(offset as size_t);
        } else {
            return 1 as libc::c_int;
        }
        return 0 as libc::c_int;
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_fseek: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
    1 as libc::c_int
}
#[no_mangle]
pub unsafe extern "C" fn HTS_ftell(fp: *mut HTS_File) -> size_t {
    if fp.is_null() {
        return 0 as libc::c_int as size_t;
    } else if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        let mut pos: fpos_t = _G_fpos_t {
            __pos: 0,
            __state: __mbstate_t {
                __count: 0,
                __value: C2RustUnnamed { __wch: 0 },
            },
        };
        fgetpos((*fp).pointer as *mut FILE, &mut pos);
        return pos.__pos as size_t;
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        let d: *mut HTS_Data = (*fp).pointer as *mut HTS_Data;
        return (*d).index;
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_ftell: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
    0 as libc::c_int as size_t
}
unsafe extern "C" fn HTS_fread(
    buf: *mut libc::c_void,
    size: size_t,
    n: size_t,
    fp: *mut HTS_File,
) -> size_t {
    if fp.is_null() || size == 0 as libc::c_int as size_t || n == 0 as libc::c_int as size_t {
        return 0 as libc::c_int as size_t;
    }
    if (*fp).type_0 as libc::c_int == 0 as libc::c_int {
        return fread(buf, size, n, (*fp).pointer as *mut FILE);
    } else if (*fp).type_0 as libc::c_int == 1 as libc::c_int {
        let d: *mut HTS_Data = (*fp).pointer as *mut HTS_Data;
        let mut i: size_t = 0;
        let length: size_t = size * n;
        let c: *mut libc::c_uchar = buf as *mut libc::c_uchar;
        i = 0 as libc::c_int as size_t;
        while i < length {
            if (*d).index >= (*d).size {
                break;
            }
            let fresh1 = (*d).index;
            (*d).index = ((*d).index).wrapping_add(1);
            *c.offset(i as isize) = *((*d).data).offset(fresh1 as isize);
            i = i.wrapping_add(1);
            i;
        }
        if i == 0 as libc::c_int as size_t {
            return 0 as libc::c_int as size_t;
        } else {
            return i / size;
        }
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_fread: Unknown file type.\n\0" as *const u8 as *const libc::c_char,
    );
    0 as libc::c_int as size_t
}
unsafe extern "C" fn HTS_byte_swap(p: *mut libc::c_void, size: size_t, block: size_t) {
    let mut q: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut tmp: libc::c_char = 0;
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    q = p as *mut libc::c_char;
    i = 0 as libc::c_int as size_t;
    while i < block {
        j = 0 as libc::c_int as size_t;
        while j < size / 2 as libc::c_int as size_t {
            tmp = *q.offset(j as isize);
            *q.offset(j as isize) = *q.offset(
                size.wrapping_sub(1 as libc::c_int as size_t)
                    .wrapping_sub(j) as isize,
            );
            *q.offset(
                size.wrapping_sub(1 as libc::c_int as size_t)
                    .wrapping_sub(j) as isize,
            ) = tmp;
            j = j.wrapping_add(1);
            j;
        }
        q = q.offset(size as isize);
        i = i.wrapping_add(1);
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fread_big_endian(
    buf: *mut libc::c_void,
    size: size_t,
    n: size_t,
    fp: *mut HTS_File,
) -> size_t {
    let block: size_t = HTS_fread(buf, size, n, fp);
    HTS_byte_swap(buf, size, block);
    block
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fread_little_endian(
    buf: *mut libc::c_void,
    size: size_t,
    n: size_t,
    fp: *mut HTS_File,
) -> size_t {
    let block: size_t = HTS_fread(buf, size, n, fp);
    block
}
#[no_mangle]
pub unsafe extern "C" fn HTS_fwrite_little_endian(
    buf: *const libc::c_void,
    size: size_t,
    n: size_t,
    fp: *mut FILE,
) -> size_t {
    fwrite(buf, size, n, fp)
}
#[no_mangle]
pub unsafe extern "C" fn HTS_get_pattern_token(
    fp: *mut HTS_File,
    buff: *mut libc::c_char,
) -> HTS_Boolean {
    let mut c: libc::c_char = 0;
    let mut i: size_t = 0;
    let mut squote: HTS_Boolean = 0 as libc::c_int as HTS_Boolean;
    let mut dquote: HTS_Boolean = 0 as libc::c_int as HTS_Boolean;
    if fp.is_null() || HTS_feof(fp) != 0 {
        return 0 as libc::c_int as HTS_Boolean;
    }
    c = HTS_fgetc(fp) as libc::c_char;
    while c as libc::c_int == ' ' as i32 || c as libc::c_int == '\n' as i32 {
        if HTS_feof(fp) != 0 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        c = HTS_fgetc(fp) as libc::c_char;
    }
    if c as libc::c_int == '\'' as i32 {
        if HTS_feof(fp) != 0 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        c = HTS_fgetc(fp) as libc::c_char;
        squote = 1 as libc::c_int as HTS_Boolean;
    }
    if c as libc::c_int == '"' as i32 {
        if HTS_feof(fp) != 0 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        c = HTS_fgetc(fp) as libc::c_char;
        dquote = 1 as libc::c_int as HTS_Boolean;
    }
    if c as libc::c_int == ',' as i32 {
        strcpy(buff, b",\0" as *const u8 as *const libc::c_char);
        return 1 as libc::c_int as HTS_Boolean;
    }
    i = 0 as libc::c_int as size_t;
    loop {
        let fresh2 = i;
        i = i.wrapping_add(1);
        *buff.offset(fresh2 as isize) = c;
        c = HTS_fgetc(fp) as libc::c_char;
        if squote as libc::c_int != 0 && c as libc::c_int == '\'' as i32 {
            break;
        }
        if dquote as libc::c_int != 0 && c as libc::c_int == '"' as i32 {
            break;
        }
        if !(squote == 0 && dquote == 0) {
            continue;
        }
        if c as libc::c_int == ' ' as i32 {
            break;
        }
        if c as libc::c_int == '\n' as i32 {
            break;
        }
        if HTS_feof(fp) != 0 {
            break;
        }
    }
    *buff.offset(i as isize) = '\0' as i32 as libc::c_char;
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_get_token_from_fp(
    fp: *mut HTS_File,
    buff: *mut libc::c_char,
) -> HTS_Boolean {
    let mut c: libc::c_char = 0;
    let mut i: size_t = 0;
    if fp.is_null() || HTS_feof(fp) != 0 {
        return 0 as libc::c_int as HTS_Boolean;
    }
    c = HTS_fgetc(fp) as libc::c_char;
    while c as libc::c_int == ' ' as i32
        || c as libc::c_int == '\n' as i32
        || c as libc::c_int == '\t' as i32
    {
        if HTS_feof(fp) != 0 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        c = HTS_fgetc(fp) as libc::c_char;
        if c as libc::c_int == -(1 as libc::c_int) {
            return 0 as libc::c_int as HTS_Boolean;
        }
    }
    i = 0 as libc::c_int as size_t;
    while c as libc::c_int != ' ' as i32
        && c as libc::c_int != '\n' as i32
        && c as libc::c_int != '\t' as i32
    {
        let fresh3 = i;
        i = i.wrapping_add(1);
        *buff.offset(fresh3 as isize) = c;
        if HTS_feof(fp) != 0 {
            break;
        }
        c = HTS_fgetc(fp) as libc::c_char;
        if c as libc::c_int == -(1 as libc::c_int) {
            break;
        }
    }
    *buff.offset(i as isize) = '\0' as i32 as libc::c_char;
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_get_token_from_fp_with_separator(
    fp: *mut HTS_File,
    buff: *mut libc::c_char,
    separator: libc::c_char,
) -> HTS_Boolean {
    let mut c: libc::c_char = 0;
    let mut i: size_t = 0;
    if fp.is_null() || HTS_feof(fp) != 0 {
        return 0 as libc::c_int as HTS_Boolean;
    }
    c = HTS_fgetc(fp) as libc::c_char;
    while c as libc::c_int == separator as libc::c_int {
        if HTS_feof(fp) != 0 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        c = HTS_fgetc(fp) as libc::c_char;
        if c as libc::c_int == -(1 as libc::c_int) {
            return 0 as libc::c_int as HTS_Boolean;
        }
    }
    i = 0 as libc::c_int as size_t;
    while c as libc::c_int != separator as libc::c_int {
        let fresh4 = i;
        i = i.wrapping_add(1);
        *buff.offset(fresh4 as isize) = c;
        if HTS_feof(fp) != 0 {
            break;
        }
        c = HTS_fgetc(fp) as libc::c_char;
        if c as libc::c_int == -(1 as libc::c_int) {
            break;
        }
    }
    *buff.offset(i as isize) = '\0' as i32 as libc::c_char;
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_get_token_from_string(
    string: *const libc::c_char,
    index: *mut size_t,
    buff: *mut libc::c_char,
) -> HTS_Boolean {
    let mut c: libc::c_char = 0;
    let mut i: size_t = 0;
    c = *string.offset(*index as isize);
    if c as libc::c_int == '\0' as i32 {
        return 0 as libc::c_int as HTS_Boolean;
    }
    let fresh5 = *index;
    *index = (*index).wrapping_add(1);
    c = *string.offset(fresh5 as isize);
    if c as libc::c_int == '\0' as i32 {
        return 0 as libc::c_int as HTS_Boolean;
    }
    while c as libc::c_int == ' ' as i32
        || c as libc::c_int == '\n' as i32
        || c as libc::c_int == '\t' as i32
    {
        if c as libc::c_int == '\0' as i32 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        let fresh6 = *index;
        *index = (*index).wrapping_add(1);
        c = *string.offset(fresh6 as isize);
    }
    i = 0 as libc::c_int as size_t;
    while c as libc::c_int != ' ' as i32
        && c as libc::c_int != '\n' as i32
        && c as libc::c_int != '\t' as i32
        && c as libc::c_int != '\0' as i32
    {
        *buff.offset(i as isize) = c;
        let fresh7 = *index;
        *index = (*index).wrapping_add(1);
        c = *string.offset(fresh7 as isize);
        i = i.wrapping_add(1);
        i;
    }
    *buff.offset(i as isize) = '\0' as i32 as libc::c_char;
    1 as libc::c_int as HTS_Boolean
}
#[no_mangle]
pub unsafe extern "C" fn HTS_get_token_from_string_with_separator(
    str: *const libc::c_char,
    index: *mut size_t,
    buff: *mut libc::c_char,
    separator: libc::c_char,
) -> HTS_Boolean {
    let mut c: libc::c_char = 0;
    let mut len: size_t = 0 as libc::c_int as size_t;
    if str.is_null() {
        return 0 as libc::c_int as HTS_Boolean;
    }
    c = *str.offset(*index as isize);
    if c as libc::c_int == '\0' as i32 {
        return 0 as libc::c_int as HTS_Boolean;
    }
    while c as libc::c_int == separator as libc::c_int {
        if c as libc::c_int == '\0' as i32 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        *index = (*index).wrapping_add(1);
        *index;
        c = *str.offset(*index as isize);
    }
    while c as libc::c_int != separator as libc::c_int && c as libc::c_int != '\0' as i32 {
        let fresh8 = len;
        len = len.wrapping_add(1);
        *buff.offset(fresh8 as isize) = c;
        *index = (*index).wrapping_add(1);
        *index;
        c = *str.offset(*index as isize);
    }
    if c as libc::c_int != '\0' as i32 {
        *index = (*index).wrapping_add(1);
        *index;
    }
    *buff.offset(len as isize) = '\0' as i32 as libc::c_char;
    if len > 0 as libc::c_int as size_t {
        1 as libc::c_int as HTS_Boolean
    } else {
        0 as libc::c_int as HTS_Boolean
    }
}
#[no_mangle]
pub unsafe extern "C" fn HTS_calloc(num: size_t, size: size_t) -> *mut libc::c_void {
    let n: size_t = num * size;
    let mut mem: *mut libc::c_void = std::ptr::null_mut::<libc::c_void>();
    if n == 0 as libc::c_int as size_t {
        return std::ptr::null_mut::<libc::c_void>();
    }
    mem = malloc(n);
    memset(mem, 0 as libc::c_int, n);
    if mem.is_null() {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_calloc: Cannot allocate memory.\n\0" as *const u8 as *const libc::c_char,
        );
    }
    mem
}
#[no_mangle]
pub unsafe extern "C" fn HTS_free(ptr: *mut libc::c_void) {
    free(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn HTS_strdup(string: *const libc::c_char) -> *mut libc::c_char {
    let buff: *mut libc::c_char = HTS_calloc(
        (strlen(string)).wrapping_add(1 as libc::c_int as libc::c_ulong),
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
    ) as *mut libc::c_char;
    strcpy(buff, string);
    buff
}
#[no_mangle]
pub unsafe extern "C" fn HTS_alloc_matrix(
    x: size_t,
    y: size_t,
) -> *mut *mut libc::c_double {
    let mut i: size_t = 0;
    let mut p: *mut *mut libc::c_double = std::ptr::null_mut::<*mut libc::c_double>();
    if x == 0 as libc::c_int as size_t || y == 0 as libc::c_int as size_t {
        return std::ptr::null_mut::<*mut libc::c_double>();
    }
    p = HTS_calloc(
        x,
        ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
    ) as *mut *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < x {
        let fresh9 = &mut (*p.offset(i as isize));
        *fresh9 = HTS_calloc(y, ::core::mem::size_of::<libc::c_double>() as libc::c_ulong)
            as *mut libc::c_double;
        i = i.wrapping_add(1);
        i;
    }
    p
}
#[no_mangle]
pub unsafe extern "C" fn HTS_free_matrix(p: *mut *mut libc::c_double, x: size_t) {
    let mut i: size_t = 0;
    i = 0 as libc::c_int as size_t;
    while i < x {
        HTS_free(*p.offset(i as isize) as *mut libc::c_void);
        i = i.wrapping_add(1);
        i;
    }
    HTS_free(p as *mut libc::c_void);
}

// #[no_mangle]
// pub unsafe extern "C" fn HTS_error!(
//     mut error: libc::c_int,
//     mut message: *const libc::c_char,
//     mut args:*const libc::c_char
// ) {
//     // let mut arg: ::core::ffi::VaListImpl;
//     // fflush(stdout);
//     // fflush(stderr);
//     // if error > 0 as libc::c_int {
//     //     fprintf(stderr, b"\nError: \0" as *const u8 as *const libc::c_char);
//     // } else {
//     //     fprintf(stderr, b"\nWarning: \0" as *const u8 as *const libc::c_char);
//     // }
//     // arg = args.clone();
//     // vfprintf(stderr, message, arg.as_va_list());
//     // fflush(stderr);
//     // if error > 0 as libc::c_int {
//     //     exit(error);
//     // }
// }
