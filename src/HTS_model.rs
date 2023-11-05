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
    fn atoi(__nptr: *const libc::c_char) -> libc::c_int;
    fn free(_: *mut libc::c_void);
    fn abs(_: libc::c_int) -> libc::c_int;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn strchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
    fn strrchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
    fn strstr(_: *const libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn __ctype_b_loc() -> *mut *const libc::c_ushort;
    fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
}

use crate::{
    HTS_calloc, HTS_fclose, HTS_feof, HTS_fopen_from_data, HTS_fopen_from_fn, HTS_fopen_from_fp,
    HTS_fread_little_endian, HTS_free, HTS_fseek, HTS_ftell, HTS_get_pattern_token,
    HTS_get_token_from_fp, HTS_get_token_from_fp_with_separator,
    HTS_get_token_from_string_with_separator, HTS_strdup,
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

#[derive(Clone)]
pub struct HTS_Window {
    pub size: size_t,
    pub l_width: *mut libc::c_int,
    pub r_width: *mut libc::c_int,
    pub coefficient: *mut *mut libc::c_double,
    pub max_width: size_t,
}

#[derive(Clone)]
pub struct HTS_Pattern {
    pub string: *mut libc::c_char,
    pub next: *mut HTS_Pattern,
}

#[derive(Clone)]
pub struct HTS_Question {
    pub string: *mut libc::c_char,
    pub head: *mut HTS_Pattern,
    pub next: *mut HTS_Question,
}

#[derive(Clone)]
pub struct HTS_Node {
    pub index: libc::c_int,
    pub pdf: size_t,
    pub yes: *mut HTS_Node,
    pub no: *mut HTS_Node,
    pub next: *mut HTS_Node,
    pub quest: *mut HTS_Question,
}

#[derive(Clone)]
pub struct HTS_Tree {
    pub head: *mut HTS_Pattern,
    pub next: *mut HTS_Tree,
    pub root: *mut HTS_Node,
    pub state: size_t,
}

#[derive(Clone)]
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

#[derive(Clone)]
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

unsafe fn HTS_dp_match(
    mut string: *const libc::c_char,
    mut pattern: *const libc::c_char,
    mut pos: size_t,
    mut max: size_t,
) -> HTS_Boolean {
    if pos > max {
        return 0 as libc::c_int as HTS_Boolean;
    }
    if *string.offset(0 as libc::c_int as isize) as libc::c_int == '\0' as i32
        && *pattern.offset(0 as libc::c_int as isize) as libc::c_int == '\0' as i32
    {
        return 1 as libc::c_int as HTS_Boolean;
    }
    if *pattern.offset(0 as libc::c_int as isize) as libc::c_int == '*' as i32 {
        if HTS_dp_match(
            string.offset(1 as libc::c_int as isize),
            pattern,
            pos.wrapping_add(1 as libc::c_int as size_t),
            max,
        ) as libc::c_int
            == 1 as libc::c_int
        {
            return 1 as libc::c_int as HTS_Boolean;
        } else {
            return HTS_dp_match(string, pattern.offset(1 as libc::c_int as isize), pos, max);
        }
    }
    if (*string.offset(0 as libc::c_int as isize) as libc::c_int
        == *pattern.offset(0 as libc::c_int as isize) as libc::c_int
        || *pattern.offset(0 as libc::c_int as isize) as libc::c_int == '?' as i32)
        && HTS_dp_match(
            string.offset(1 as libc::c_int as isize),
            pattern.offset(1 as libc::c_int as isize),
            pos.wrapping_add(1 as libc::c_int as size_t),
            max.wrapping_add(1 as libc::c_int as size_t),
        ) as libc::c_int
            == 1 as libc::c_int
    {
        return 1 as libc::c_int as HTS_Boolean;
    }
    0 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_pattern_match(
    mut string: *const libc::c_char,
    mut pattern: *const libc::c_char,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut buff_length: size_t = 0;
    let mut max: size_t = 0 as libc::c_int as size_t;
    let mut nstar: size_t = 0 as libc::c_int as size_t;
    let mut nquestion: size_t = 0 as libc::c_int as size_t;
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut pattern_length: size_t = strlen(pattern);
    i = 0 as libc::c_int as size_t;
    while i < pattern_length {
        match *pattern.offset(i as isize) as libc::c_int {
            42 => {
                nstar = nstar.wrapping_add(1);
            }
            63 => {
                nquestion = nquestion.wrapping_add(1);
                max = max.wrapping_add(1);
            }
            _ => {
                max = max.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    if nstar == 2 as libc::c_int as size_t
        && nquestion == 0 as libc::c_int as size_t
        && *pattern.offset(0 as libc::c_int as isize) as libc::c_int == '*' as i32
        && *pattern.offset(i.wrapping_sub(1 as libc::c_int as size_t) as isize) as libc::c_int
            == '*' as i32
    {
        buff_length = i.wrapping_sub(2 as libc::c_int as size_t);
        i = 0 as libc::c_int as size_t;
        j = 1 as libc::c_int as size_t;
        while i < buff_length {
            buff[i as usize] = *pattern.offset(j as isize);
            i = i.wrapping_add(1);
            j = j.wrapping_add(1);
        }
        buff[buff_length as usize] = '\0' as i32 as libc::c_char;
        if !(strstr(string, buff.as_mut_ptr())).is_null() {
            1 as libc::c_int as HTS_Boolean
        } else {
            0 as libc::c_int as HTS_Boolean
        }
    } else {
        HTS_dp_match(
            string,
            pattern,
            0 as libc::c_int as size_t,
            (strlen(string)).wrapping_sub(max),
        )
    }
}
unsafe fn HTS_is_num(mut buff: *const libc::c_char) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut length: size_t = strlen(buff);
    i = 0 as libc::c_int as size_t;
    while i < length {
        if !(*(*__ctype_b_loc()).offset(*buff.offset(i as isize) as libc::c_int as isize)
            as libc::c_int
            & _ISdigit as libc::c_int as libc::c_ushort as libc::c_int
            != 0
            || *buff.offset(i as isize) as libc::c_int == '-' as i32)
        {
            return 0 as libc::c_int as HTS_Boolean;
        }
        i = i.wrapping_add(1);
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_name2num(mut buff: *const libc::c_char) -> size_t {
    let mut i: size_t = 0;
    i = (strlen(buff)).wrapping_sub(1 as libc::c_int as libc::c_ulong);
    while '0' as i32 <= *buff.offset(i as isize) as libc::c_int
        && *buff.offset(i as isize) as libc::c_int <= '9' as i32
    {
        i = i.wrapping_sub(1);
    }
    i = i.wrapping_add(1);
    atoi(&*buff.offset(i as isize)) as size_t
}
unsafe fn HTS_get_state_num(mut string: *const libc::c_char) -> size_t {
    let mut left: *const libc::c_char = std::ptr::null::<libc::c_char>();
    let mut right: *const libc::c_char = std::ptr::null::<libc::c_char>();
    left = strchr(string, '[' as i32);
    if left.is_null() {
        return 0 as libc::c_int as size_t;
    }
    left = left.offset(1);
    right = strchr(left, ']' as i32);
    if right.is_null() {
        return 0 as libc::c_int as size_t;
    }
    atoi(left) as size_t
}
unsafe fn HTS_Question_initialize(mut question: *mut HTS_Question) {
    (*question).string = std::ptr::null_mut::<libc::c_char>();
    (*question).head = std::ptr::null_mut::<HTS_Pattern>();
    (*question).next = std::ptr::null_mut::<HTS_Question>();
}
unsafe fn HTS_Question_clear(mut question: *mut HTS_Question) {
    let mut pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    let mut next_pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    if !((*question).string).is_null() {
        HTS_free((*question).string as *mut libc::c_void);
    }
    pattern = (*question).head;
    while !pattern.is_null() {
        next_pattern = (*pattern).next;
        HTS_free((*pattern).string as *mut libc::c_void);
        HTS_free(pattern as *mut libc::c_void);
        pattern = next_pattern;
    }
    HTS_Question_initialize(question);
}
unsafe fn HTS_Question_load(mut question: *mut HTS_Question, mut fp: *mut HTS_File) -> HTS_Boolean {
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    let mut last_pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    if question.is_null() || fp.is_null() {
        return 0 as libc::c_int as HTS_Boolean;
    }
    HTS_Question_clear(question);
    if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*question).string = HTS_strdup(buff.as_mut_ptr());
    if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
        HTS_Question_clear(question);
        return 0 as libc::c_int as HTS_Boolean;
    }
    last_pattern = std::ptr::null_mut::<HTS_Pattern>();
    if strcmp(
        buff.as_mut_ptr(),
        b"{\0" as *const u8 as *const libc::c_char,
    ) == 0 as libc::c_int
    {
        loop {
            if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
                HTS_Question_clear(question);
                return 0 as libc::c_int as HTS_Boolean;
            }
            pattern = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_Pattern>() as libc::c_ulong,
            ) as *mut HTS_Pattern;
            if !((*question).head).is_null() {
                (*last_pattern).next = pattern;
            } else {
                (*question).head = pattern;
            }
            (*pattern).string = HTS_strdup(buff.as_mut_ptr());
            (*pattern).next = std::ptr::null_mut::<HTS_Pattern>();
            if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
                HTS_Question_clear(question);
                return 0 as libc::c_int as HTS_Boolean;
            }
            if strcmp(
                buff.as_mut_ptr(),
                b"}\0" as *const u8 as *const libc::c_char,
            ) == 0
            {
                break;
            }
            last_pattern = pattern;
        }
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Question_match(
    mut question: *mut HTS_Question,
    mut string: *const libc::c_char,
) -> HTS_Boolean {
    let mut pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    pattern = (*question).head;
    while !pattern.is_null() {
        if HTS_pattern_match(string, (*pattern).string) != 0 {
            return 1 as libc::c_int as HTS_Boolean;
        }
        pattern = (*pattern).next;
    }
    0 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Question_find(
    mut question: *mut HTS_Question,
    mut string: *const libc::c_char,
) -> *mut HTS_Question {
    while !question.is_null() {
        if strcmp(string, (*question).string) == 0 as libc::c_int {
            return question;
        }
        question = (*question).next;
    }
    std::ptr::null_mut::<HTS_Question>()
}
unsafe fn HTS_Node_initialize(mut node: *mut HTS_Node) {
    (*node).index = 0 as libc::c_int;
    (*node).pdf = 0 as libc::c_int as size_t;
    (*node).yes = std::ptr::null_mut::<HTS_Node>();
    (*node).no = std::ptr::null_mut::<HTS_Node>();
    (*node).next = std::ptr::null_mut::<HTS_Node>();
    (*node).quest = std::ptr::null_mut::<HTS_Question>();
}
unsafe fn HTS_Node_clear(mut node: *mut HTS_Node) {
    if !((*node).yes).is_null() {
        HTS_Node_clear((*node).yes);
        HTS_free((*node).yes as *mut libc::c_void);
    }
    if !((*node).no).is_null() {
        HTS_Node_clear((*node).no);
        HTS_free((*node).no as *mut libc::c_void);
    }
    HTS_Node_initialize(node);
}
unsafe fn HTS_Node_find(mut node: *mut HTS_Node, mut num: libc::c_int) -> *mut HTS_Node {
    while !node.is_null() {
        if (*node).index == num {
            return node;
        }
        node = (*node).next;
    }
    std::ptr::null_mut::<HTS_Node>()
}
unsafe fn HTS_Tree_initialize(mut tree: *mut HTS_Tree) {
    (*tree).head = std::ptr::null_mut::<HTS_Pattern>();
    (*tree).next = std::ptr::null_mut::<HTS_Tree>();
    (*tree).root = std::ptr::null_mut::<HTS_Node>();
    (*tree).state = 0 as libc::c_int as size_t;
}
unsafe fn HTS_Tree_clear(mut tree: *mut HTS_Tree) {
    let mut pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    let mut next_pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    pattern = (*tree).head;
    while !pattern.is_null() {
        next_pattern = (*pattern).next;
        HTS_free((*pattern).string as *mut libc::c_void);
        HTS_free(pattern as *mut libc::c_void);
        pattern = next_pattern;
    }
    if !((*tree).root).is_null() {
        HTS_Node_clear((*tree).root);
        HTS_free((*tree).root as *mut libc::c_void);
    }
    HTS_Tree_initialize(tree);
}
unsafe fn HTS_Tree_parse_pattern(mut tree: *mut HTS_Tree, mut string: *mut libc::c_char) {
    let mut left: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut right: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    let mut last_pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    (*tree).head = std::ptr::null_mut::<HTS_Pattern>();
    last_pattern = std::ptr::null_mut::<HTS_Pattern>();
    left = strchr(string, '{' as i32);
    if !left.is_null() {
        string = left.offset(1 as libc::c_int as isize);
        if *string as libc::c_int == '(' as i32 {
            string = string.offset(1);
        }
        right = strrchr(string, '}' as i32);
        if string < right
            && *right.offset(-(1 as libc::c_int as isize)) as libc::c_int == ')' as i32
        {
            right = right.offset(-1);
        }
        *right = ',' as i32 as libc::c_char;
        loop {
            left = strchr(string, ',' as i32);
            if left.is_null() {
                break;
            }
            pattern = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_Pattern>() as libc::c_ulong,
            ) as *mut HTS_Pattern;
            if !((*tree).head).is_null() {
                (*last_pattern).next = pattern;
            } else {
                (*tree).head = pattern;
            }
            *left = '\0' as i32 as libc::c_char;
            (*pattern).string = HTS_strdup(string);
            string = left.offset(1 as libc::c_int as isize);
            (*pattern).next = std::ptr::null_mut::<HTS_Pattern>();
            last_pattern = pattern;
        }
    }
}
unsafe fn HTS_Tree_load(
    mut tree: *mut HTS_Tree,
    mut fp: *mut HTS_File,
    mut question: *mut HTS_Question,
) -> HTS_Boolean {
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut node: *mut HTS_Node = std::ptr::null_mut::<HTS_Node>();
    let mut last_node: *mut HTS_Node = std::ptr::null_mut::<HTS_Node>();
    if tree.is_null() || fp.is_null() {
        return 0 as libc::c_int as HTS_Boolean;
    }
    if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
        HTS_Tree_clear(tree);
        return 0 as libc::c_int as HTS_Boolean;
    }
    node = HTS_calloc(
        1 as libc::c_int as size_t,
        ::core::mem::size_of::<HTS_Node>() as libc::c_ulong,
    ) as *mut HTS_Node;
    HTS_Node_initialize(node);
    last_node = node;
    (*tree).root = last_node;
    if strcmp(
        buff.as_mut_ptr(),
        b"{\0" as *const u8 as *const libc::c_char,
    ) == 0 as libc::c_int
    {
        while HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 1 as libc::c_int
            && strcmp(
                buff.as_mut_ptr(),
                b"}\0" as *const u8 as *const libc::c_char,
            ) != 0 as libc::c_int
        {
            node = HTS_Node_find(last_node, atoi(buff.as_mut_ptr()));
            if node.is_null() {
                HTS_error!(
                    0 as libc::c_int,
                    b"HTS_Tree_load: Cannot find node %d.\n\0" as *const u8 as *const libc::c_char,
                    atoi(buff.as_mut_ptr()),
                );
                HTS_Tree_clear(tree);
                return 0 as libc::c_int as HTS_Boolean;
            }
            if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
                HTS_Tree_clear(tree);
                return 0 as libc::c_int as HTS_Boolean;
            }
            (*node).quest = HTS_Question_find(question, buff.as_mut_ptr());
            if ((*node).quest).is_null() {
                HTS_error!(
                    0 as libc::c_int,
                    b"HTS_Tree_load: Cannot find question %s.\n\0" as *const u8
                        as *const libc::c_char,
                    buff.as_mut_ptr(),
                );
                HTS_Tree_clear(tree);
                return 0 as libc::c_int as HTS_Boolean;
            }
            (*node).yes = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_Node>() as libc::c_ulong,
            ) as *mut HTS_Node;
            (*node).no = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_Node>() as libc::c_ulong,
            ) as *mut HTS_Node;
            HTS_Node_initialize((*node).yes);
            HTS_Node_initialize((*node).no);
            if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
                (*node).quest = std::ptr::null_mut::<HTS_Question>();
                free((*node).yes as *mut libc::c_void);
                free((*node).no as *mut libc::c_void);
                HTS_Tree_clear(tree);
                return 0 as libc::c_int as HTS_Boolean;
            }
            if HTS_is_num(buff.as_mut_ptr()) != 0 {
                (*(*node).no).index = atoi(buff.as_mut_ptr());
            } else {
                (*(*node).no).pdf = HTS_name2num(buff.as_mut_ptr());
            }
            (*(*node).no).next = last_node;
            last_node = (*node).no;
            if HTS_get_pattern_token(fp, buff.as_mut_ptr()) as libc::c_int == 0 as libc::c_int {
                (*node).quest = std::ptr::null_mut::<HTS_Question>();
                free((*node).yes as *mut libc::c_void);
                free((*node).no as *mut libc::c_void);
                HTS_Tree_clear(tree);
                return 0 as libc::c_int as HTS_Boolean;
            }
            if HTS_is_num(buff.as_mut_ptr()) != 0 {
                (*(*node).yes).index = atoi(buff.as_mut_ptr());
            } else {
                (*(*node).yes).pdf = HTS_name2num(buff.as_mut_ptr());
            }
            (*(*node).yes).next = last_node;
            last_node = (*node).yes;
        }
    } else {
        (*node).pdf = HTS_name2num(buff.as_mut_ptr());
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Tree_search_node(mut tree: *mut HTS_Tree, mut string: *const libc::c_char) -> size_t {
    let mut node: *mut HTS_Node = (*tree).root;
    while !node.is_null() {
        if ((*node).quest).is_null() {
            return (*node).pdf;
        }
        if HTS_Question_match((*node).quest, string) != 0 {
            if (*(*node).yes).pdf > 0 as libc::c_int as size_t {
                return (*(*node).yes).pdf;
            }
            node = (*node).yes;
        } else {
            if (*(*node).no).pdf > 0 as libc::c_int as size_t {
                return (*(*node).no).pdf;
            }
            node = (*node).no;
        }
    }
    HTS_error!(
        0 as libc::c_int,
        b"HTS_Tree_search_node: Cannot find node.\n\0" as *const u8 as *const libc::c_char,
    );
    1 as libc::c_int as size_t
}
unsafe fn HTS_Window_initialize(mut win: *mut HTS_Window) {
    (*win).size = 0 as libc::c_int as size_t;
    (*win).l_width = std::ptr::null_mut::<libc::c_int>();
    (*win).r_width = std::ptr::null_mut::<libc::c_int>();
    (*win).coefficient = std::ptr::null_mut::<*mut libc::c_double>();
    (*win).max_width = 0 as libc::c_int as size_t;
}
unsafe fn HTS_Window_clear(mut win: *mut HTS_Window) {
    let mut i: size_t = 0;
    if !((*win).coefficient).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*win).size {
            let fresh0 = &mut (*((*win).coefficient).offset(i as isize));
            *fresh0 = (*fresh0).offset(*((*win).l_width).offset(i as isize) as isize);
            HTS_free(*((*win).coefficient).offset(i as isize) as *mut libc::c_void);
            i = i.wrapping_add(1);
        }
        HTS_free((*win).coefficient as *mut libc::c_void);
    }
    if !((*win).l_width).is_null() {
        HTS_free((*win).l_width as *mut libc::c_void);
    }
    if !((*win).r_width).is_null() {
        HTS_free((*win).r_width as *mut libc::c_void);
    }
    HTS_Window_initialize(win);
}
unsafe fn HTS_Window_load(
    mut win: *mut HTS_Window,
    mut fp: *mut *mut HTS_File,
    mut size: size_t,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut fsize: size_t = 0;
    let mut length: size_t = 0;
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut result: HTS_Boolean = 1 as libc::c_int as HTS_Boolean;
    if win.is_null() || fp.is_null() || size == 0 as libc::c_int as size_t {
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*win).size = size;
    (*win).l_width = HTS_calloc(
        (*win).size,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    ) as *mut libc::c_int;
    (*win).r_width = HTS_calloc(
        (*win).size,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    ) as *mut libc::c_int;
    (*win).coefficient = HTS_calloc(
        (*win).size,
        ::core::mem::size_of::<*mut libc::c_double>() as libc::c_ulong,
    ) as *mut *mut libc::c_double;
    i = 0 as libc::c_int as size_t;
    while i < (*win).size {
        if HTS_get_token_from_fp(*fp.offset(i as isize), buff.as_mut_ptr()) as libc::c_int
            == 0 as libc::c_int
        {
            result = 0 as libc::c_int as HTS_Boolean;
            fsize = 1 as libc::c_int as size_t;
        } else {
            fsize = atoi(buff.as_mut_ptr()) as size_t;
            if fsize == 0 as libc::c_int as size_t {
                result = 0 as libc::c_int as HTS_Boolean;
                fsize = 1 as libc::c_int as size_t;
            }
        }
        let fresh1 = &mut (*((*win).coefficient).offset(i as isize));
        *fresh1 = HTS_calloc(
            fsize,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        j = 0 as libc::c_int as size_t;
        while j < fsize {
            if HTS_get_token_from_fp(*fp.offset(i as isize), buff.as_mut_ptr()) as libc::c_int
                == 0 as libc::c_int
            {
                result = 0 as libc::c_int as HTS_Boolean;
                *(*((*win).coefficient).offset(i as isize)).offset(j as isize) = 0.0f64;
            } else {
                *(*((*win).coefficient).offset(i as isize)).offset(j as isize) =
                    atof(buff.as_mut_ptr());
            }
            j = j.wrapping_add(1);
        }
        length = fsize / 2 as libc::c_int as size_t;
        let fresh2 = &mut (*((*win).coefficient).offset(i as isize));
        *fresh2 = (*fresh2).offset(length as isize);
        *((*win).l_width).offset(i as isize) = -(1 as libc::c_int) * length as libc::c_int;
        *((*win).r_width).offset(i as isize) = length as libc::c_int;
        if fsize % 2 as libc::c_int as size_t == 0 as libc::c_int as size_t {
            let fresh3 = &mut (*((*win).r_width).offset(i as isize));
            *fresh3 -= 1;
        }
        i = i.wrapping_add(1);
    }
    (*win).max_width = 0 as libc::c_int as size_t;
    i = 0 as libc::c_int as size_t;
    while i < (*win).size {
        if (*win).max_width < abs(*((*win).l_width).offset(i as isize)) as size_t {
            (*win).max_width = abs(*((*win).l_width).offset(i as isize)) as size_t;
        }
        if (*win).max_width < abs(*((*win).r_width).offset(i as isize)) as size_t {
            (*win).max_width = abs(*((*win).r_width).offset(i as isize)) as size_t;
        }
        i = i.wrapping_add(1);
    }
    if result as libc::c_int == 0 as libc::c_int {
        HTS_Window_clear(win);
        return 0 as libc::c_int as HTS_Boolean;
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Model_initialize(mut model: *mut HTS_Model) {
    (*model).vector_length = 0 as libc::c_int as size_t;
    (*model).num_windows = 0 as libc::c_int as size_t;
    (*model).is_msd = 0 as libc::c_int as HTS_Boolean;
    (*model).ntree = 0 as libc::c_int as size_t;
    (*model).npdf = std::ptr::null_mut::<size_t>();
    (*model).pdf = std::ptr::null_mut::<*mut *mut libc::c_float>();
    (*model).tree = std::ptr::null_mut::<HTS_Tree>();
    (*model).question = std::ptr::null_mut::<HTS_Question>();
}
unsafe fn HTS_Model_clear(mut model: *mut HTS_Model) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut question: *mut HTS_Question = std::ptr::null_mut::<HTS_Question>();
    let mut next_question: *mut HTS_Question = std::ptr::null_mut::<HTS_Question>();
    let mut tree: *mut HTS_Tree = std::ptr::null_mut::<HTS_Tree>();
    let mut next_tree: *mut HTS_Tree = std::ptr::null_mut::<HTS_Tree>();
    question = (*model).question;
    while !question.is_null() {
        next_question = (*question).next;
        HTS_Question_clear(question);
        HTS_free(question as *mut libc::c_void);
        question = next_question;
    }
    tree = (*model).tree;
    while !tree.is_null() {
        next_tree = (*tree).next;
        HTS_Tree_clear(tree);
        HTS_free(tree as *mut libc::c_void);
        tree = next_tree;
    }
    if !((*model).pdf).is_null() {
        i = 2 as libc::c_int as size_t;
        while i <= ((*model).ntree).wrapping_add(1 as libc::c_int as size_t) {
            j = 1 as libc::c_int as size_t;
            while j <= *((*model).npdf).offset(i as isize) {
                HTS_free(
                    *(*((*model).pdf).offset(i as isize)).offset(j as isize) as *mut libc::c_void
                );
                j = j.wrapping_add(1);
            }
            let fresh4 = &mut (*((*model).pdf).offset(i as isize));
            *fresh4 = (*fresh4).offset(1);
            HTS_free(*((*model).pdf).offset(i as isize) as *mut libc::c_void);
            i = i.wrapping_add(1);
        }
        (*model).pdf = ((*model).pdf).offset(2 as libc::c_int as isize);
        HTS_free((*model).pdf as *mut libc::c_void);
    }
    if !((*model).npdf).is_null() {
        (*model).npdf = ((*model).npdf).offset(2 as libc::c_int as isize);
        HTS_free((*model).npdf as *mut libc::c_void);
    }
    HTS_Model_initialize(model);
}
unsafe fn HTS_Model_load_tree(mut model: *mut HTS_Model, mut fp: *mut HTS_File) -> HTS_Boolean {
    let mut buff: [libc::c_char; 1024] = [0; 1024];
    let mut question: *mut HTS_Question = std::ptr::null_mut::<HTS_Question>();
    let mut last_question: *mut HTS_Question = std::ptr::null_mut::<HTS_Question>();
    let mut tree: *mut HTS_Tree = std::ptr::null_mut::<HTS_Tree>();
    let mut last_tree: *mut HTS_Tree = std::ptr::null_mut::<HTS_Tree>();
    let mut state: size_t = 0;
    if model.is_null() {
        HTS_error!(
            0 as libc::c_int,
            b"HTS_Model_load_tree: File for trees is not specified.\n\0" as *const u8
                as *const libc::c_char,
        );
        return 0 as libc::c_int as HTS_Boolean;
    }
    if fp.is_null() {
        (*model).ntree = 1 as libc::c_int as size_t;
        return 1 as libc::c_int as HTS_Boolean;
    }
    (*model).ntree = 0 as libc::c_int as size_t;
    last_question = std::ptr::null_mut::<HTS_Question>();
    last_tree = std::ptr::null_mut::<HTS_Tree>();
    while HTS_feof(fp) == 0 {
        HTS_get_pattern_token(fp, buff.as_mut_ptr());
        if strcmp(
            buff.as_mut_ptr(),
            b"QS\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            question = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_Question>() as libc::c_ulong,
            ) as *mut HTS_Question;
            HTS_Question_initialize(question);
            if HTS_Question_load(question, fp) as libc::c_int == 0 as libc::c_int {
                free(question as *mut libc::c_void);
                HTS_Model_clear(model);
                return 0 as libc::c_int as HTS_Boolean;
            }
            if !((*model).question).is_null() {
                (*last_question).next = question;
            } else {
                (*model).question = question;
            }
            (*question).next = std::ptr::null_mut::<HTS_Question>();
            last_question = question;
        }
        state = HTS_get_state_num(buff.as_mut_ptr());
        if state != 0 as libc::c_int as size_t {
            tree = HTS_calloc(
                1 as libc::c_int as size_t,
                ::core::mem::size_of::<HTS_Tree>() as libc::c_ulong,
            ) as *mut HTS_Tree;
            HTS_Tree_initialize(tree);
            (*tree).state = state;
            HTS_Tree_parse_pattern(tree, buff.as_mut_ptr());
            if HTS_Tree_load(tree, fp, (*model).question) as libc::c_int == 0 as libc::c_int {
                free(tree as *mut libc::c_void);
                HTS_Model_clear(model);
                return 0 as libc::c_int as HTS_Boolean;
            }
            if !((*model).tree).is_null() {
                (*last_tree).next = tree;
            } else {
                (*model).tree = tree;
            }
            (*tree).next = std::ptr::null_mut::<HTS_Tree>();
            last_tree = tree;
            (*model).ntree = ((*model).ntree).wrapping_add(1);
            (*model).ntree;
        }
    }
    if ((*model).tree).is_null() {
        (*model).ntree = 1 as libc::c_int as size_t;
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Model_load_pdf(
    mut model: *mut HTS_Model,
    mut fp: *mut HTS_File,
    mut vector_length: size_t,
    mut num_windows: size_t,
    mut is_msd: HTS_Boolean,
) -> HTS_Boolean {
    let mut i: uint32_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut result: HTS_Boolean = 1 as libc::c_int as HTS_Boolean;
    let mut len: size_t = 0;
    if model.is_null() || fp.is_null() || (*model).ntree <= 0 as libc::c_int as size_t {
        HTS_error!(
            1 as libc::c_int,
            b"HTS_Model_load_pdf: File for pdfs is not specified.\n\0" as *const u8
                as *const libc::c_char,
        );
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*model).vector_length = vector_length;
    (*model).num_windows = num_windows;
    (*model).is_msd = is_msd;
    (*model).npdf = HTS_calloc(
        (*model).ntree,
        ::core::mem::size_of::<size_t>() as libc::c_ulong,
    ) as *mut size_t;
    (*model).npdf = ((*model).npdf).offset(-(2 as libc::c_int as isize));
    j = 2 as libc::c_int as size_t;
    while j <= ((*model).ntree).wrapping_add(1 as libc::c_int as size_t) {
        if HTS_fread_little_endian(
            &mut i as *mut uint32_t as *mut libc::c_void,
            ::core::mem::size_of::<uint32_t>() as libc::c_ulong,
            1 as libc::c_int as size_t,
            fp,
        ) != 1 as libc::c_int as size_t
        {
            result = 0 as libc::c_int as HTS_Boolean;
            break;
        } else {
            *((*model).npdf).offset(j as isize) = i as size_t;
            j = j.wrapping_add(1);
        }
    }
    j = 2 as libc::c_int as size_t;
    while j <= ((*model).ntree).wrapping_add(1 as libc::c_int as size_t) {
        if *((*model).npdf).offset(j as isize) <= 0 as libc::c_int as size_t {
            HTS_error!(
                1 as libc::c_int,
                b"HTS_Model_load_pdf: # of pdfs at %d-th state should be positive.\n\0" as *const u8
                    as *const libc::c_char,
                j,
            );
            result = 0 as libc::c_int as HTS_Boolean;
            break;
        } else {
            j = j.wrapping_add(1);
        }
    }
    if result as libc::c_int == 0 as libc::c_int {
        (*model).npdf = ((*model).npdf).offset(2 as libc::c_int as isize);
        free((*model).npdf as *mut libc::c_void);
        HTS_Model_initialize(model);
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*model).pdf = HTS_calloc(
        (*model).ntree,
        ::core::mem::size_of::<*mut *mut libc::c_float>() as libc::c_ulong,
    ) as *mut *mut *mut libc::c_float;
    (*model).pdf = ((*model).pdf).offset(-(2 as libc::c_int as isize));
    if is_msd != 0 {
        len = ((*model).vector_length * (*model).num_windows * 2 as libc::c_int as size_t)
            .wrapping_add(1 as libc::c_int as size_t);
    } else {
        len = (*model).vector_length * (*model).num_windows * 2 as libc::c_int as size_t;
    }
    j = 2 as libc::c_int as size_t;
    while j <= ((*model).ntree).wrapping_add(1 as libc::c_int as size_t) {
        let fresh5 = &mut (*((*model).pdf).offset(j as isize));
        *fresh5 = HTS_calloc(
            *((*model).npdf).offset(j as isize),
            ::core::mem::size_of::<*mut libc::c_float>() as libc::c_ulong,
        ) as *mut *mut libc::c_float;
        let fresh6 = &mut (*((*model).pdf).offset(j as isize));
        *fresh6 = (*fresh6).offset(-1);
        k = 1 as libc::c_int as size_t;
        while k <= *((*model).npdf).offset(j as isize) {
            let fresh7 = &mut (*(*((*model).pdf).offset(j as isize)).offset(k as isize));
            *fresh7 = HTS_calloc(
                len,
                ::core::mem::size_of::<libc::c_float>() as libc::c_ulong,
            ) as *mut libc::c_float;
            if HTS_fread_little_endian(
                *(*((*model).pdf).offset(j as isize)).offset(k as isize) as *mut libc::c_void,
                ::core::mem::size_of::<libc::c_float>() as libc::c_ulong,
                len,
                fp,
            ) != len
            {
                result = 0 as libc::c_int as HTS_Boolean;
            }
            k = k.wrapping_add(1);
        }
        j = j.wrapping_add(1);
    }
    if result as libc::c_int == 0 as libc::c_int {
        HTS_Model_clear(model);
        return 0 as libc::c_int as HTS_Boolean;
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Model_load(
    mut model: *mut HTS_Model,
    mut pdf: *mut HTS_File,
    mut tree: *mut HTS_File,
    mut vector_length: size_t,
    mut num_windows: size_t,
    mut is_msd: HTS_Boolean,
) -> HTS_Boolean {
    if model.is_null()
        || pdf.is_null()
        || vector_length == 0 as libc::c_int as size_t
        || num_windows == 0 as libc::c_int as size_t
    {
        return 0 as libc::c_int as HTS_Boolean;
    }
    HTS_Model_clear(model);
    if HTS_Model_load_tree(model, tree) as libc::c_int != 1 as libc::c_int {
        HTS_Model_clear(model);
        return 0 as libc::c_int as HTS_Boolean;
    }
    if HTS_Model_load_pdf(model, pdf, vector_length, num_windows, is_msd) as libc::c_int
        != 1 as libc::c_int
    {
        HTS_Model_clear(model);
        return 0 as libc::c_int as HTS_Boolean;
    }
    1 as libc::c_int as HTS_Boolean
}
unsafe fn HTS_Model_get_index(
    mut model: *mut HTS_Model,
    mut state_index: size_t,
    mut string: *const libc::c_char,
    mut tree_index: *mut size_t,
    mut pdf_index: *mut size_t,
) {
    let mut tree: *mut HTS_Tree = std::ptr::null_mut::<HTS_Tree>();
    let mut pattern: *mut HTS_Pattern = std::ptr::null_mut::<HTS_Pattern>();
    let mut find: HTS_Boolean = 0;
    *tree_index = 2 as libc::c_int as size_t;
    *pdf_index = 1 as libc::c_int as size_t;
    if ((*model).tree).is_null() {
        return;
    }
    find = 0 as libc::c_int as HTS_Boolean;
    tree = (*model).tree;
    while !tree.is_null() {
        if (*tree).state == state_index {
            pattern = (*tree).head;
            if pattern.is_null() {
                find = 1 as libc::c_int as HTS_Boolean;
            }
            while !pattern.is_null() {
                if HTS_pattern_match(string, (*pattern).string) != 0 {
                    find = 1 as libc::c_int as HTS_Boolean;
                    break;
                } else {
                    pattern = (*pattern).next;
                }
            }
            if find != 0 {
                break;
            }
        }
        *tree_index = (*tree_index).wrapping_add(1);
        tree = (*tree).next;
    }
    if !tree.is_null() {
        *pdf_index = HTS_Tree_search_node(tree, string);
    } else {
        *pdf_index = HTS_Tree_search_node((*model).tree, string);
    };
}

pub fn HTS_ModelSet_initialize() -> HTS_ModelSet {
    HTS_ModelSet {
        hts_voice_version: std::ptr::null_mut::<libc::c_char>(),
        sampling_frequency: 0 as libc::c_int as size_t,
        frame_period: 0 as libc::c_int as size_t,
        num_voices: 0 as libc::c_int as size_t,
        num_states: 0 as libc::c_int as size_t,
        num_streams: 0 as libc::c_int as size_t,
        stream_type: std::ptr::null_mut::<libc::c_char>(),
        fullcontext_format: std::ptr::null_mut::<libc::c_char>(),
        fullcontext_version: std::ptr::null_mut::<libc::c_char>(),
        gv_off_context: std::ptr::null_mut::<HTS_Question>(),
        option: std::ptr::null_mut::<*mut libc::c_char>(),
        duration: std::ptr::null_mut::<HTS_Model>(),
        window: std::ptr::null_mut::<HTS_Window>(),
        stream: std::ptr::null_mut::<*mut HTS_Model>(),
        gv: std::ptr::null_mut::<*mut HTS_Model>(),
    }
}

pub unsafe fn HTS_ModelSet_clear(mut ms: *mut HTS_ModelSet) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    if !((*ms).hts_voice_version).is_null() {
        free((*ms).hts_voice_version as *mut libc::c_void);
    }
    if !((*ms).stream_type).is_null() {
        free((*ms).stream_type as *mut libc::c_void);
    }
    if !((*ms).fullcontext_format).is_null() {
        free((*ms).fullcontext_format as *mut libc::c_void);
    }
    if !((*ms).fullcontext_version).is_null() {
        free((*ms).fullcontext_version as *mut libc::c_void);
    }
    if !((*ms).gv_off_context).is_null() {
        HTS_Question_clear((*ms).gv_off_context);
        free((*ms).gv_off_context as *mut libc::c_void);
    }
    if !((*ms).option).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*ms).num_streams {
            if !(*((*ms).option).offset(i as isize)).is_null() {
                free(*((*ms).option).offset(i as isize) as *mut libc::c_void);
            }
            i = i.wrapping_add(1);
        }
        free((*ms).option as *mut libc::c_void);
    }
    if !((*ms).duration).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*ms).num_voices {
            HTS_Model_clear(&mut *((*ms).duration).offset(i as isize));
            i = i.wrapping_add(1);
        }
        free((*ms).duration as *mut libc::c_void);
    }
    if !((*ms).window).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*ms).num_streams {
            HTS_Window_clear(&mut *((*ms).window).offset(i as isize));
            i = i.wrapping_add(1);
        }
        free((*ms).window as *mut libc::c_void);
    }
    if !((*ms).stream).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*ms).num_voices {
            j = 0 as libc::c_int as size_t;
            while j < (*ms).num_streams {
                HTS_Model_clear(&mut *(*((*ms).stream).offset(i as isize)).offset(j as isize));
                j = j.wrapping_add(1);
            }
            free(*((*ms).stream).offset(i as isize) as *mut libc::c_void);
            i = i.wrapping_add(1);
        }
        HTS_free((*ms).stream as *mut libc::c_void);
    }
    if !((*ms).gv).is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*ms).num_voices {
            j = 0 as libc::c_int as size_t;
            while j < (*ms).num_streams {
                HTS_Model_clear(&mut *(*((*ms).gv).offset(i as isize)).offset(j as isize));
                j = j.wrapping_add(1);
            }
            free(*((*ms).gv).offset(i as isize) as *mut libc::c_void);
            i = i.wrapping_add(1);
        }
        free((*ms).gv as *mut libc::c_void);
    }
}
unsafe fn HTS_match_head_string(
    mut str: *const libc::c_char,
    mut pattern: *const libc::c_char,
    mut matched_size: *mut size_t,
) -> HTS_Boolean {
    *matched_size = 0 as libc::c_int as size_t;
    loop {
        if *pattern.offset(*matched_size as isize) as libc::c_int == '\0' as i32 {
            return 1 as libc::c_int as HTS_Boolean;
        }
        if *str.offset(*matched_size as isize) as libc::c_int == '\0' as i32 {
            return 0 as libc::c_int as HTS_Boolean;
        }
        if *str.offset(*matched_size as isize) as libc::c_int
            != *pattern.offset(*matched_size as isize) as libc::c_int
        {
            return 0 as libc::c_int as HTS_Boolean;
        }
        *matched_size = (*matched_size).wrapping_add(1);
    }
}
unsafe fn HTS_strequal(mut s1: *const libc::c_char, mut s2: *const libc::c_char) -> HTS_Boolean {
    if s1.is_null() && s2.is_null() {
        1 as libc::c_int as HTS_Boolean
    } else if s1.is_null() || s2.is_null() {
        return 0 as libc::c_int as HTS_Boolean;
    } else {
        return (if strcmp(s1, s2) == 0 as libc::c_int {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        }) as HTS_Boolean;
    }
}

pub unsafe fn HTS_ModelSet_load(
    mut ms: *mut HTS_ModelSet,
    mut voices: *mut *mut libc::c_char,
    mut num_voices: size_t,
) -> HTS_Boolean {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut k: size_t = 0;
    let mut s: size_t = 0;
    let mut e: size_t = 0;
    let mut error: HTS_Boolean = 0 as libc::c_int as HTS_Boolean;
    let mut fp: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
    let mut buff1: [libc::c_char; 1024] = [0; 1024];
    let mut buff2: [libc::c_char; 1024] = [0; 1024];
    let mut matched_size: size_t = 0;
    let mut stream_type_list: *mut *mut libc::c_char = std::ptr::null_mut::<*mut libc::c_char>();
    let mut vector_length: *mut size_t = std::ptr::null_mut::<size_t>();
    let mut is_msd: *mut HTS_Boolean = std::ptr::null_mut::<HTS_Boolean>();
    let mut num_windows: *mut size_t = std::ptr::null_mut::<size_t>();
    let mut use_gv: *mut HTS_Boolean = std::ptr::null_mut::<HTS_Boolean>();
    let mut gv_off_context: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_hts_voice_version: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_sampling_frequency: size_t = 0;
    let mut temp_frame_period: size_t = 0;
    let mut temp_num_states: size_t = 0;
    let mut temp_num_streams: size_t = 0;
    let mut temp_stream_type: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_fullcontext_format: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_fullcontext_version: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_gv_off_context: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_vector_length: *mut size_t = std::ptr::null_mut::<size_t>();
    let mut temp_is_msd: *mut HTS_Boolean = std::ptr::null_mut::<HTS_Boolean>();
    let mut temp_num_windows: *mut size_t = std::ptr::null_mut::<size_t>();
    let mut temp_use_gv: *mut HTS_Boolean = std::ptr::null_mut::<HTS_Boolean>();
    let mut temp_option: *mut *mut libc::c_char = std::ptr::null_mut::<*mut libc::c_char>();
    let mut temp_duration_pdf: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_duration_tree: *mut libc::c_char = std::ptr::null_mut::<libc::c_char>();
    let mut temp_stream_win: *mut *mut *mut libc::c_char =
        std::ptr::null_mut::<*mut *mut libc::c_char>();
    let mut temp_stream_pdf: *mut *mut libc::c_char = std::ptr::null_mut::<*mut libc::c_char>();
    let mut temp_stream_tree: *mut *mut libc::c_char = std::ptr::null_mut::<*mut libc::c_char>();
    let mut temp_gv_pdf: *mut *mut libc::c_char = std::ptr::null_mut::<*mut libc::c_char>();
    let mut temp_gv_tree: *mut *mut libc::c_char = std::ptr::null_mut::<*mut libc::c_char>();
    let mut start_of_data: libc::c_long = 0;
    let mut pdf_fp: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
    let mut tree_fp: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
    let mut win_fp: *mut *mut HTS_File = std::ptr::null_mut::<*mut HTS_File>();
    let mut gv_off_context_fp: *mut HTS_File = std::ptr::null_mut::<HTS_File>();
    HTS_ModelSet_clear(ms);
    if ms.is_null() || voices.is_null() || num_voices < 1 as libc::c_int as size_t {
        return 0 as libc::c_int as HTS_Boolean;
    }
    (*ms).num_voices = num_voices;
    i = 0 as libc::c_int as size_t;
    while i < num_voices && error as libc::c_int == 0 as libc::c_int {
        fp = HTS_fopen_from_fn(
            *voices.offset(i as isize),
            b"rb\0" as *const u8 as *const libc::c_char,
        );
        if fp.is_null() {
            error = 1 as libc::c_int as HTS_Boolean;
            break;
        } else {
            temp_hts_voice_version = std::ptr::null_mut::<libc::c_char>();
            temp_sampling_frequency = 0 as libc::c_int as size_t;
            temp_frame_period = 0 as libc::c_int as size_t;
            temp_num_states = 0 as libc::c_int as size_t;
            temp_num_streams = 0 as libc::c_int as size_t;
            temp_stream_type = std::ptr::null_mut::<libc::c_char>();
            temp_fullcontext_format = std::ptr::null_mut::<libc::c_char>();
            temp_fullcontext_version = std::ptr::null_mut::<libc::c_char>();
            temp_gv_off_context = std::ptr::null_mut::<libc::c_char>();
            if HTS_get_token_from_fp_with_separator(
                fp,
                buff1.as_mut_ptr(),
                '\n' as i32 as libc::c_char,
            ) as libc::c_int
                != 1 as libc::c_int
            {
                error = 1 as libc::c_int as HTS_Boolean;
                break;
            } else if HTS_strequal(
                buff1.as_mut_ptr(),
                b"[GLOBAL]\0" as *const u8 as *const libc::c_char,
            ) as libc::c_int
                != 1 as libc::c_int
            {
                error = 1 as libc::c_int as HTS_Boolean;
                break;
            } else {
                loop {
                    if HTS_get_token_from_fp_with_separator(
                        fp,
                        buff1.as_mut_ptr(),
                        '\n' as i32 as libc::c_char,
                    ) as libc::c_int
                        != 1 as libc::c_int
                    {
                        error = 1 as libc::c_int as HTS_Boolean;
                        break;
                    } else {
                        if HTS_strequal(
                            buff1.as_mut_ptr(),
                            b"[STREAM]\0" as *const u8 as *const libc::c_char,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            break;
                        }
                        if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"HTS_VOICE_VERSION:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            if !temp_hts_voice_version.is_null() {
                                free(temp_hts_voice_version as *mut libc::c_void);
                            }
                            temp_hts_voice_version =
                                HTS_strdup(&mut *buff1.as_mut_ptr().offset(matched_size as isize));
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"SAMPLING_FREQUENCY:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            temp_sampling_frequency =
                                atoi(&mut *buff1.as_mut_ptr().offset(matched_size as isize))
                                    as size_t;
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"FRAME_PERIOD:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            temp_frame_period =
                                atoi(&mut *buff1.as_mut_ptr().offset(matched_size as isize))
                                    as size_t;
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"NUM_STATES:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            temp_num_states =
                                atoi(&mut *buff1.as_mut_ptr().offset(matched_size as isize))
                                    as size_t;
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"NUM_STREAMS:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            temp_num_streams =
                                atoi(&mut *buff1.as_mut_ptr().offset(matched_size as isize))
                                    as size_t;
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"STREAM_TYPE:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            if !temp_stream_type.is_null() {
                                free(temp_stream_type as *mut libc::c_void);
                            }
                            temp_stream_type =
                                HTS_strdup(&mut *buff1.as_mut_ptr().offset(matched_size as isize));
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"FULLCONTEXT_FORMAT:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            if !temp_fullcontext_format.is_null() {
                                free(temp_fullcontext_format as *mut libc::c_void);
                            }
                            temp_fullcontext_format =
                                HTS_strdup(&mut *buff1.as_mut_ptr().offset(matched_size as isize));
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"FULLCONTEXT_VERSION:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            if !temp_fullcontext_version.is_null() {
                                free(temp_fullcontext_version as *mut libc::c_void);
                            }
                            temp_fullcontext_version =
                                HTS_strdup(&mut *buff1.as_mut_ptr().offset(matched_size as isize));
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"GV_OFF_CONTEXT:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            if !temp_gv_off_context.is_null() {
                                free(temp_gv_off_context as *mut libc::c_void);
                            }
                            temp_gv_off_context =
                                HTS_strdup(&mut *buff1.as_mut_ptr().offset(matched_size as isize));
                        } else if HTS_match_head_string(
                            buff1.as_mut_ptr(),
                            b"COMMENT:\0" as *const u8 as *const libc::c_char,
                            &mut matched_size,
                        ) as libc::c_int
                            != 1 as libc::c_int
                        {
                            HTS_error!(
                                0 as libc::c_int,
                                b"HTS_ModelSet_load: Unknown option %s.\n\0" as *const u8
                                    as *const libc::c_char,
                                buff1.as_mut_ptr(),
                            );
                        }
                    }
                }
                if i == 0 as libc::c_int as size_t {
                    (*ms).hts_voice_version = temp_hts_voice_version;
                    (*ms).sampling_frequency = temp_sampling_frequency;
                    (*ms).frame_period = temp_frame_period;
                    (*ms).num_states = temp_num_states;
                    (*ms).num_streams = temp_num_streams;
                    (*ms).stream_type = temp_stream_type;
                    (*ms).fullcontext_format = temp_fullcontext_format;
                    (*ms).fullcontext_version = temp_fullcontext_version;
                    gv_off_context = temp_gv_off_context;
                } else {
                    if HTS_strequal((*ms).hts_voice_version, temp_hts_voice_version) as libc::c_int
                        != 1 as libc::c_int
                    {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if (*ms).sampling_frequency != temp_sampling_frequency {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if (*ms).frame_period != temp_frame_period {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if (*ms).num_states != temp_num_states {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if (*ms).num_streams != temp_num_streams {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if HTS_strequal((*ms).stream_type, temp_stream_type) as libc::c_int
                        != 1 as libc::c_int
                    {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if HTS_strequal((*ms).fullcontext_format, temp_fullcontext_format)
                        as libc::c_int
                        != 1 as libc::c_int
                    {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if HTS_strequal((*ms).fullcontext_version, temp_fullcontext_version)
                        as libc::c_int
                        != 1 as libc::c_int
                    {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if HTS_strequal(gv_off_context, temp_gv_off_context) as libc::c_int
                        != 1 as libc::c_int
                    {
                        error = 1 as libc::c_int as HTS_Boolean;
                    }
                    if !temp_hts_voice_version.is_null() {
                        free(temp_hts_voice_version as *mut libc::c_void);
                    }
                    if !temp_stream_type.is_null() {
                        free(temp_stream_type as *mut libc::c_void);
                    }
                    if !temp_fullcontext_format.is_null() {
                        free(temp_fullcontext_format as *mut libc::c_void);
                    }
                    if !temp_fullcontext_version.is_null() {
                        free(temp_fullcontext_version as *mut libc::c_void);
                    }
                    if !temp_gv_off_context.is_null() {
                        free(temp_gv_off_context as *mut libc::c_void);
                    }
                }
                if i == 0 as libc::c_int as size_t {
                    stream_type_list = HTS_calloc(
                        (*ms).num_streams,
                        ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                    ) as *mut *mut libc::c_char;
                    j = 0 as libc::c_int as size_t;
                    matched_size = 0 as libc::c_int as size_t;
                    while j < (*ms).num_streams {
                        if HTS_get_token_from_string_with_separator(
                            (*ms).stream_type,
                            &mut matched_size,
                            buff2.as_mut_ptr(),
                            ',' as i32 as libc::c_char,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            let fresh8 = &mut (*stream_type_list.offset(j as isize));
                            *fresh8 = HTS_strdup(buff2.as_mut_ptr());
                        } else {
                            let fresh9 = &mut (*stream_type_list.offset(j as isize));
                            *fresh9 = std::ptr::null_mut::<libc::c_char>();
                            error = 1 as libc::c_int as HTS_Boolean;
                        }
                        j = j.wrapping_add(1);
                    }
                }
                if error as libc::c_int != 0 as libc::c_int {
                    HTS_fclose(fp);
                    break;
                } else {
                    temp_vector_length = HTS_calloc(
                        (*ms).num_streams,
                        ::core::mem::size_of::<size_t>() as libc::c_ulong,
                    ) as *mut size_t;
                    j = 0 as libc::c_int as size_t;
                    while j < (*ms).num_streams {
                        *temp_vector_length.offset(j as isize) = 0 as libc::c_int as size_t;
                        j = j.wrapping_add(1);
                    }
                    temp_is_msd = HTS_calloc(
                        (*ms).num_streams,
                        ::core::mem::size_of::<HTS_Boolean>() as libc::c_ulong,
                    ) as *mut HTS_Boolean;
                    j = 0 as libc::c_int as size_t;
                    while j < (*ms).num_streams {
                        *temp_is_msd.offset(j as isize) = 0 as libc::c_int as HTS_Boolean;
                        j = j.wrapping_add(1);
                    }
                    temp_num_windows = HTS_calloc(
                        (*ms).num_streams,
                        ::core::mem::size_of::<size_t>() as libc::c_ulong,
                    ) as *mut size_t;
                    j = 0 as libc::c_int as size_t;
                    while j < (*ms).num_streams {
                        *temp_num_windows.offset(j as isize) = 0 as libc::c_int as size_t;
                        j = j.wrapping_add(1);
                    }
                    temp_use_gv = HTS_calloc(
                        (*ms).num_streams,
                        ::core::mem::size_of::<HTS_Boolean>() as libc::c_ulong,
                    ) as *mut HTS_Boolean;
                    j = 0 as libc::c_int as size_t;
                    while j < (*ms).num_streams {
                        *temp_use_gv.offset(j as isize) = 0 as libc::c_int as HTS_Boolean;
                        j = j.wrapping_add(1);
                    }
                    temp_option = HTS_calloc(
                        (*ms).num_streams,
                        ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                    ) as *mut *mut libc::c_char;
                    j = 0 as libc::c_int as size_t;
                    while j < (*ms).num_streams {
                        let fresh10 = &mut (*temp_option.offset(j as isize));
                        *fresh10 = std::ptr::null_mut::<libc::c_char>();
                        j = j.wrapping_add(1);
                    }
                    loop {
                        if HTS_get_token_from_fp_with_separator(
                            fp,
                            buff1.as_mut_ptr(),
                            '\n' as i32 as libc::c_char,
                        ) as libc::c_int
                            != 1 as libc::c_int
                        {
                            error = 1 as libc::c_int as HTS_Boolean;
                            break;
                        } else {
                            if strcmp(
                                buff1.as_mut_ptr(),
                                b"[POSITION]\0" as *const u8 as *const libc::c_char,
                            ) == 0 as libc::c_int
                            {
                                break;
                            }
                            if HTS_match_head_string(
                                buff1.as_mut_ptr(),
                                b"VECTOR_LENGTH[\0" as *const u8 as *const libc::c_char,
                                &mut matched_size,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                if HTS_get_token_from_string_with_separator(
                                    buff1.as_mut_ptr(),
                                    &mut matched_size,
                                    buff2.as_mut_ptr(),
                                    ']' as i32 as libc::c_char,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    let fresh11 = matched_size;
                                    matched_size = matched_size.wrapping_add(1);
                                    if buff1[fresh11 as usize] as libc::c_int == ':' as i32 {
                                        j = 0 as libc::c_int as size_t;
                                        while j < (*ms).num_streams {
                                            if strcmp(
                                                *stream_type_list.offset(j as isize),
                                                buff2.as_mut_ptr(),
                                            ) == 0 as libc::c_int
                                            {
                                                *temp_vector_length.offset(j as isize) = atoi(
                                                    &mut *buff1
                                                        .as_mut_ptr()
                                                        .offset(matched_size as isize),
                                                )
                                                    as size_t;
                                                break;
                                            } else {
                                                j = j.wrapping_add(1);
                                            }
                                        }
                                    }
                                }
                            } else if HTS_match_head_string(
                                buff1.as_mut_ptr(),
                                b"IS_MSD[\0" as *const u8 as *const libc::c_char,
                                &mut matched_size,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                if HTS_get_token_from_string_with_separator(
                                    buff1.as_mut_ptr(),
                                    &mut matched_size,
                                    buff2.as_mut_ptr(),
                                    ']' as i32 as libc::c_char,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    let fresh12 = matched_size;
                                    matched_size = matched_size.wrapping_add(1);
                                    if buff1[fresh12 as usize] as libc::c_int == ':' as i32 {
                                        j = 0 as libc::c_int as size_t;
                                        while j < (*ms).num_streams {
                                            if strcmp(
                                                *stream_type_list.offset(j as isize),
                                                buff2.as_mut_ptr(),
                                            ) == 0 as libc::c_int
                                            {
                                                *temp_is_msd.offset(j as isize) =
                                                    (if buff1[matched_size as usize] as libc::c_int
                                                        == '1' as i32
                                                    {
                                                        1 as libc::c_int
                                                    } else {
                                                        0 as libc::c_int
                                                    })
                                                        as HTS_Boolean;
                                                break;
                                            } else {
                                                j = j.wrapping_add(1);
                                            }
                                        }
                                    }
                                }
                            } else if HTS_match_head_string(
                                buff1.as_mut_ptr(),
                                b"NUM_WINDOWS[\0" as *const u8 as *const libc::c_char,
                                &mut matched_size,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                if HTS_get_token_from_string_with_separator(
                                    buff1.as_mut_ptr(),
                                    &mut matched_size,
                                    buff2.as_mut_ptr(),
                                    ']' as i32 as libc::c_char,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    let fresh13 = matched_size;
                                    matched_size = matched_size.wrapping_add(1);
                                    if buff1[fresh13 as usize] as libc::c_int == ':' as i32 {
                                        j = 0 as libc::c_int as size_t;
                                        while j < (*ms).num_streams {
                                            if strcmp(
                                                *stream_type_list.offset(j as isize),
                                                buff2.as_mut_ptr(),
                                            ) == 0 as libc::c_int
                                            {
                                                *temp_num_windows.offset(j as isize) = atoi(
                                                    &mut *buff1
                                                        .as_mut_ptr()
                                                        .offset(matched_size as isize),
                                                )
                                                    as size_t;
                                                break;
                                            } else {
                                                j = j.wrapping_add(1);
                                            }
                                        }
                                    }
                                }
                            } else if HTS_match_head_string(
                                buff1.as_mut_ptr(),
                                b"USE_GV[\0" as *const u8 as *const libc::c_char,
                                &mut matched_size,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                if HTS_get_token_from_string_with_separator(
                                    buff1.as_mut_ptr(),
                                    &mut matched_size,
                                    buff2.as_mut_ptr(),
                                    ']' as i32 as libc::c_char,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    let fresh14 = matched_size;
                                    matched_size = matched_size.wrapping_add(1);
                                    if buff1[fresh14 as usize] as libc::c_int == ':' as i32 {
                                        j = 0 as libc::c_int as size_t;
                                        while j < (*ms).num_streams {
                                            if strcmp(
                                                *stream_type_list.offset(j as isize),
                                                buff2.as_mut_ptr(),
                                            ) == 0 as libc::c_int
                                            {
                                                *temp_use_gv.offset(j as isize) =
                                                    (if buff1[matched_size as usize] as libc::c_int
                                                        == '1' as i32
                                                    {
                                                        1 as libc::c_int
                                                    } else {
                                                        0 as libc::c_int
                                                    })
                                                        as HTS_Boolean;
                                                break;
                                            } else {
                                                j = j.wrapping_add(1);
                                            }
                                        }
                                    }
                                }
                            } else if HTS_match_head_string(
                                buff1.as_mut_ptr(),
                                b"OPTION[\0" as *const u8 as *const libc::c_char,
                                &mut matched_size,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                if HTS_get_token_from_string_with_separator(
                                    buff1.as_mut_ptr(),
                                    &mut matched_size,
                                    buff2.as_mut_ptr(),
                                    ']' as i32 as libc::c_char,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    let fresh15 = matched_size;
                                    matched_size = matched_size.wrapping_add(1);
                                    if buff1[fresh15 as usize] as libc::c_int == ':' as i32 {
                                        j = 0 as libc::c_int as size_t;
                                        while j < (*ms).num_streams {
                                            if strcmp(
                                                *stream_type_list.offset(j as isize),
                                                buff2.as_mut_ptr(),
                                            ) == 0 as libc::c_int
                                            {
                                                if !(*temp_option.offset(j as isize)).is_null() {
                                                    free(*temp_option.offset(j as isize)
                                                        as *mut libc::c_void);
                                                }
                                                let fresh16 =
                                                    &mut (*temp_option.offset(j as isize));
                                                *fresh16 = HTS_strdup(
                                                    &mut *buff1
                                                        .as_mut_ptr()
                                                        .offset(matched_size as isize),
                                                );
                                                break;
                                            } else {
                                                j = j.wrapping_add(1);
                                            }
                                        }
                                    }
                                }
                            } else {
                                HTS_error!(
                                    0 as libc::c_int,
                                    b"HTS_ModelSet_load: Unknown option %s.\n\0" as *const u8
                                        as *const libc::c_char,
                                    buff1.as_mut_ptr(),
                                );
                            }
                        }
                    }
                    if i == 0 as libc::c_int as size_t {
                        vector_length = temp_vector_length;
                        is_msd = temp_is_msd;
                        num_windows = temp_num_windows;
                        use_gv = temp_use_gv;
                        (*ms).option = temp_option;
                    } else {
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if *vector_length.offset(j as isize)
                                != *temp_vector_length.offset(j as isize)
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if *is_msd.offset(j as isize) as libc::c_int
                                != *is_msd.offset(j as isize) as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if *num_windows.offset(j as isize)
                                != *temp_num_windows.offset(j as isize)
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if *use_gv.offset(j as isize) as libc::c_int
                                != *temp_use_gv.offset(j as isize) as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if HTS_strequal(
                                *((*ms).option).offset(j as isize),
                                *temp_option.offset(j as isize),
                            ) as libc::c_int
                                != 1 as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            j = j.wrapping_add(1);
                        }
                        free(temp_vector_length as *mut libc::c_void);
                        free(temp_is_msd as *mut libc::c_void);
                        free(temp_num_windows as *mut libc::c_void);
                        free(temp_use_gv as *mut libc::c_void);
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if !(*temp_option.offset(j as isize)).is_null() {
                                free(*temp_option.offset(j as isize) as *mut libc::c_void);
                            }
                            j = j.wrapping_add(1);
                        }
                        free(temp_option as *mut libc::c_void);
                    }
                    if error as libc::c_int != 0 as libc::c_int {
                        HTS_fclose(fp);
                        break;
                    } else {
                        temp_duration_pdf = std::ptr::null_mut::<libc::c_char>();
                        temp_duration_tree = std::ptr::null_mut::<libc::c_char>();
                        temp_stream_win = HTS_calloc(
                            (*ms).num_streams,
                            ::core::mem::size_of::<*mut *mut libc::c_char>() as libc::c_ulong,
                        ) as *mut *mut *mut libc::c_char;
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            let fresh17 = &mut (*temp_stream_win.offset(j as isize));
                            *fresh17 = HTS_calloc(
                                *num_windows.offset(j as isize),
                                ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                            ) as *mut *mut libc::c_char;
                            k = 0 as libc::c_int as size_t;
                            while k < *num_windows.offset(j as isize) {
                                let fresh18 = &mut (*(*temp_stream_win.offset(j as isize))
                                    .offset(k as isize));
                                *fresh18 = std::ptr::null_mut::<libc::c_char>();
                                k = k.wrapping_add(1);
                            }
                            j = j.wrapping_add(1);
                        }
                        temp_stream_pdf = HTS_calloc(
                            (*ms).num_streams,
                            ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                        ) as *mut *mut libc::c_char;
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            let fresh19 = &mut (*temp_stream_pdf.offset(j as isize));
                            *fresh19 = std::ptr::null_mut::<libc::c_char>();
                            j = j.wrapping_add(1);
                        }
                        temp_stream_tree = HTS_calloc(
                            (*ms).num_streams,
                            ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                        ) as *mut *mut libc::c_char;
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            let fresh20 = &mut (*temp_stream_tree.offset(j as isize));
                            *fresh20 = std::ptr::null_mut::<libc::c_char>();
                            j = j.wrapping_add(1);
                        }
                        temp_gv_pdf = HTS_calloc(
                            (*ms).num_streams,
                            ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                        ) as *mut *mut libc::c_char;
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            let fresh21 = &mut (*temp_gv_pdf.offset(j as isize));
                            *fresh21 = std::ptr::null_mut::<libc::c_char>();
                            j = j.wrapping_add(1);
                        }
                        temp_gv_tree = HTS_calloc(
                            (*ms).num_streams,
                            ::core::mem::size_of::<*mut libc::c_char>() as libc::c_ulong,
                        ) as *mut *mut libc::c_char;
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            let fresh22 = &mut (*temp_gv_tree.offset(j as isize));
                            *fresh22 = std::ptr::null_mut::<libc::c_char>();
                            j = j.wrapping_add(1);
                        }
                        loop {
                            if HTS_get_token_from_fp_with_separator(
                                fp,
                                buff1.as_mut_ptr(),
                                '\n' as i32 as libc::c_char,
                            ) as libc::c_int
                                != 1 as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                                break;
                            } else {
                                if strcmp(
                                    buff1.as_mut_ptr(),
                                    b"[DATA]\0" as *const u8 as *const libc::c_char,
                                ) == 0 as libc::c_int
                                {
                                    break;
                                }
                                if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"DURATION_PDF:\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if !temp_duration_pdf.is_null() {
                                        free(temp_duration_pdf as *mut libc::c_void);
                                    }
                                    temp_duration_pdf = HTS_strdup(
                                        &mut *buff1.as_mut_ptr().offset(matched_size as isize),
                                    );
                                } else if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"DURATION_TREE:\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if !temp_duration_tree.is_null() {
                                        free(temp_duration_tree as *mut libc::c_void);
                                    }
                                    temp_duration_tree = HTS_strdup(
                                        &mut *buff1.as_mut_ptr().offset(matched_size as isize),
                                    );
                                } else if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"STREAM_WIN[\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if HTS_get_token_from_string_with_separator(
                                        buff1.as_mut_ptr(),
                                        &mut matched_size,
                                        buff2.as_mut_ptr(),
                                        ']' as i32 as libc::c_char,
                                    ) as libc::c_int
                                        == 1 as libc::c_int
                                    {
                                        let fresh23 = matched_size;
                                        matched_size = matched_size.wrapping_add(1);
                                        if buff1[fresh23 as usize] as libc::c_int == ':' as i32 {
                                            j = 0 as libc::c_int as size_t;
                                            while j < (*ms).num_streams {
                                                if strcmp(
                                                    *stream_type_list.offset(j as isize),
                                                    buff2.as_mut_ptr(),
                                                ) == 0 as libc::c_int
                                                {
                                                    k = 0 as libc::c_int as size_t;
                                                    while k < *num_windows.offset(j as isize) {
                                                        if HTS_get_token_from_string_with_separator(
                                                            buff1.as_mut_ptr(),
                                                            &mut matched_size,
                                                            buff2.as_mut_ptr(),
                                                            ',' as i32 as libc::c_char,
                                                        )
                                                            as libc::c_int
                                                            == 1 as libc::c_int
                                                        {
                                                            let fresh24 =
                                                                &mut (*(*temp_stream_win
                                                                    .offset(j as isize))
                                                                .offset(k as isize));
                                                            *fresh24 =
                                                                HTS_strdup(buff2.as_mut_ptr());
                                                        } else {
                                                            error = 1 as libc::c_int as HTS_Boolean;
                                                        }
                                                        k = k.wrapping_add(1);
                                                    }
                                                    break;
                                                } else {
                                                    j = j.wrapping_add(1);
                                                }
                                            }
                                        }
                                    }
                                } else if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"STREAM_PDF[\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if HTS_get_token_from_string_with_separator(
                                        buff1.as_mut_ptr(),
                                        &mut matched_size,
                                        buff2.as_mut_ptr(),
                                        ']' as i32 as libc::c_char,
                                    ) as libc::c_int
                                        == 1 as libc::c_int
                                    {
                                        let fresh25 = matched_size;
                                        matched_size = matched_size.wrapping_add(1);
                                        if buff1[fresh25 as usize] as libc::c_int == ':' as i32 {
                                            j = 0 as libc::c_int as size_t;
                                            while j < (*ms).num_streams {
                                                if strcmp(
                                                    *stream_type_list.offset(j as isize),
                                                    buff2.as_mut_ptr(),
                                                ) == 0 as libc::c_int
                                                {
                                                    if !(*temp_stream_pdf.offset(j as isize))
                                                        .is_null()
                                                    {
                                                        free(*temp_stream_pdf.offset(j as isize)
                                                            as *mut libc::c_void);
                                                    }
                                                    let fresh26 =
                                                        &mut (*temp_stream_pdf.offset(j as isize));
                                                    *fresh26 = HTS_strdup(
                                                        &mut *buff1
                                                            .as_mut_ptr()
                                                            .offset(matched_size as isize),
                                                    );
                                                    break;
                                                } else {
                                                    j = j.wrapping_add(1);
                                                }
                                            }
                                        }
                                    }
                                } else if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"STREAM_TREE[\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if HTS_get_token_from_string_with_separator(
                                        buff1.as_mut_ptr(),
                                        &mut matched_size,
                                        buff2.as_mut_ptr(),
                                        ']' as i32 as libc::c_char,
                                    ) as libc::c_int
                                        == 1 as libc::c_int
                                    {
                                        let fresh27 = matched_size;
                                        matched_size = matched_size.wrapping_add(1);
                                        if buff1[fresh27 as usize] as libc::c_int == ':' as i32 {
                                            j = 0 as libc::c_int as size_t;
                                            while j < (*ms).num_streams {
                                                if strcmp(
                                                    *stream_type_list.offset(j as isize),
                                                    buff2.as_mut_ptr(),
                                                ) == 0 as libc::c_int
                                                {
                                                    if !(*temp_stream_tree.offset(j as isize))
                                                        .is_null()
                                                    {
                                                        free(*temp_stream_tree.offset(j as isize)
                                                            as *mut libc::c_void);
                                                    }
                                                    let fresh28 =
                                                        &mut (*temp_stream_tree.offset(j as isize));
                                                    *fresh28 = HTS_strdup(
                                                        &mut *buff1
                                                            .as_mut_ptr()
                                                            .offset(matched_size as isize),
                                                    );
                                                    break;
                                                } else {
                                                    j = j.wrapping_add(1);
                                                }
                                            }
                                        }
                                    }
                                } else if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"GV_PDF[\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if HTS_get_token_from_string_with_separator(
                                        buff1.as_mut_ptr(),
                                        &mut matched_size,
                                        buff2.as_mut_ptr(),
                                        ']' as i32 as libc::c_char,
                                    ) as libc::c_int
                                        == 1 as libc::c_int
                                    {
                                        let fresh29 = matched_size;
                                        matched_size = matched_size.wrapping_add(1);
                                        if buff1[fresh29 as usize] as libc::c_int == ':' as i32 {
                                            j = 0 as libc::c_int as size_t;
                                            while j < (*ms).num_streams {
                                                if strcmp(
                                                    *stream_type_list.offset(j as isize),
                                                    buff2.as_mut_ptr(),
                                                ) == 0 as libc::c_int
                                                {
                                                    if !(*temp_gv_pdf.offset(j as isize)).is_null()
                                                    {
                                                        free(*temp_gv_pdf.offset(j as isize)
                                                            as *mut libc::c_void);
                                                    }
                                                    let fresh30 =
                                                        &mut (*temp_gv_pdf.offset(j as isize));
                                                    *fresh30 = HTS_strdup(
                                                        &mut *buff1
                                                            .as_mut_ptr()
                                                            .offset(matched_size as isize),
                                                    );
                                                    break;
                                                } else {
                                                    j = j.wrapping_add(1);
                                                }
                                            }
                                        }
                                    }
                                } else if HTS_match_head_string(
                                    buff1.as_mut_ptr(),
                                    b"GV_TREE[\0" as *const u8 as *const libc::c_char,
                                    &mut matched_size,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    if HTS_get_token_from_string_with_separator(
                                        buff1.as_mut_ptr(),
                                        &mut matched_size,
                                        buff2.as_mut_ptr(),
                                        ']' as i32 as libc::c_char,
                                    ) as libc::c_int
                                        == 1 as libc::c_int
                                    {
                                        let fresh31 = matched_size;
                                        matched_size = matched_size.wrapping_add(1);
                                        if buff1[fresh31 as usize] as libc::c_int == ':' as i32 {
                                            j = 0 as libc::c_int as size_t;
                                            while j < (*ms).num_streams {
                                                if strcmp(
                                                    *stream_type_list.offset(j as isize),
                                                    buff2.as_mut_ptr(),
                                                ) == 0 as libc::c_int
                                                {
                                                    if !(*temp_gv_tree.offset(j as isize)).is_null()
                                                    {
                                                        free(*temp_gv_tree.offset(j as isize)
                                                            as *mut libc::c_void);
                                                    }
                                                    let fresh32 =
                                                        &mut (*temp_gv_tree.offset(j as isize));
                                                    *fresh32 = HTS_strdup(
                                                        &mut *buff1
                                                            .as_mut_ptr()
                                                            .offset(matched_size as isize),
                                                    );
                                                    break;
                                                } else {
                                                    j = j.wrapping_add(1);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    HTS_error!(
                                        0 as libc::c_int,
                                        b"HTS_ModelSet_load: Unknown option %s.\n\0" as *const u8
                                            as *const libc::c_char,
                                        buff1.as_mut_ptr(),
                                    );
                                }
                            }
                        }
                        if temp_duration_pdf.is_null() {
                            error = 1 as libc::c_int as HTS_Boolean;
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            k = 0 as libc::c_int as size_t;
                            while k < *num_windows.offset(j as isize) {
                                if (*(*temp_stream_win.offset(j as isize)).offset(k as isize))
                                    .is_null()
                                {
                                    error = 1 as libc::c_int as HTS_Boolean;
                                }
                                k = k.wrapping_add(1);
                            }
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if (*temp_stream_pdf.offset(j as isize)).is_null() {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            j = j.wrapping_add(1);
                        }
                        if i == 0 as libc::c_int as size_t {
                            (*ms).duration = HTS_calloc(
                                num_voices,
                                ::core::mem::size_of::<HTS_Model>() as libc::c_ulong,
                            ) as *mut HTS_Model;
                            j = 0 as libc::c_int as size_t;
                            while j < num_voices {
                                HTS_Model_initialize(&mut *((*ms).duration).offset(j as isize));
                                j = j.wrapping_add(1);
                            }
                            (*ms).window = HTS_calloc(
                                (*ms).num_streams,
                                ::core::mem::size_of::<HTS_Window>() as libc::c_ulong,
                            ) as *mut HTS_Window;
                            j = 0 as libc::c_int as size_t;
                            while j < (*ms).num_streams {
                                HTS_Window_initialize(&mut *((*ms).window).offset(j as isize));
                                j = j.wrapping_add(1);
                            }
                            (*ms).stream = HTS_calloc(
                                num_voices,
                                ::core::mem::size_of::<*mut HTS_Model>() as libc::c_ulong,
                            ) as *mut *mut HTS_Model;
                            j = 0 as libc::c_int as size_t;
                            while j < num_voices {
                                let fresh33 = &mut (*((*ms).stream).offset(j as isize));
                                *fresh33 = HTS_calloc(
                                    (*ms).num_streams,
                                    ::core::mem::size_of::<HTS_Model>() as libc::c_ulong,
                                ) as *mut HTS_Model;
                                k = 0 as libc::c_int as size_t;
                                while k < (*ms).num_streams {
                                    HTS_Model_initialize(
                                        &mut *(*((*ms).stream).offset(j as isize))
                                            .offset(k as isize),
                                    );
                                    k = k.wrapping_add(1);
                                }
                                j = j.wrapping_add(1);
                            }
                            (*ms).gv = HTS_calloc(
                                num_voices,
                                ::core::mem::size_of::<*mut HTS_Model>() as libc::c_ulong,
                            ) as *mut *mut HTS_Model;
                            j = 0 as libc::c_int as size_t;
                            while j < num_voices {
                                let fresh34 = &mut (*((*ms).gv).offset(j as isize));
                                *fresh34 = HTS_calloc(
                                    (*ms).num_streams,
                                    ::core::mem::size_of::<HTS_Model>() as libc::c_ulong,
                                ) as *mut HTS_Model;
                                k = 0 as libc::c_int as size_t;
                                while k < (*ms).num_streams {
                                    HTS_Model_initialize(
                                        &mut *(*((*ms).gv).offset(j as isize)).offset(k as isize),
                                    );
                                    k = k.wrapping_add(1);
                                }
                                j = j.wrapping_add(1);
                            }
                        }
                        start_of_data = HTS_ftell(fp) as libc::c_long;
                        pdf_fp = std::ptr::null_mut::<HTS_File>();
                        tree_fp = std::ptr::null_mut::<HTS_File>();
                        matched_size = 0 as libc::c_int as size_t;
                        if HTS_get_token_from_string_with_separator(
                            temp_duration_pdf,
                            &mut matched_size,
                            buff2.as_mut_ptr(),
                            '-' as i32 as libc::c_char,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            s = atoi(buff2.as_mut_ptr()) as size_t;
                            e = atoi(&mut *temp_duration_pdf.offset(matched_size as isize))
                                as size_t;
                            HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                            pdf_fp = HTS_fopen_from_fp(
                                fp,
                                e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                            );
                            HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                        }
                        matched_size = 0 as libc::c_int as size_t;
                        if HTS_get_token_from_string_with_separator(
                            temp_duration_tree,
                            &mut matched_size,
                            buff2.as_mut_ptr(),
                            '-' as i32 as libc::c_char,
                        ) as libc::c_int
                            == 1 as libc::c_int
                        {
                            s = atoi(buff2.as_mut_ptr()) as size_t;
                            e = atoi(&mut *temp_duration_tree.offset(matched_size as isize))
                                as size_t;
                            HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                            tree_fp = HTS_fopen_from_fp(
                                fp,
                                e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                            );
                            HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                        }
                        if HTS_Model_load(
                            &mut *((*ms).duration).offset(i as isize),
                            pdf_fp,
                            tree_fp,
                            (*ms).num_states,
                            1 as libc::c_int as size_t,
                            0 as libc::c_int as HTS_Boolean,
                        ) as libc::c_int
                            != 1 as libc::c_int
                        {
                            error = 1 as libc::c_int as HTS_Boolean;
                        }
                        HTS_fclose(pdf_fp);
                        HTS_fclose(tree_fp);
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            win_fp = HTS_calloc(
                                *num_windows.offset(j as isize),
                                ::core::mem::size_of::<*mut HTS_File>() as libc::c_ulong,
                            ) as *mut *mut HTS_File;
                            k = 0 as libc::c_int as size_t;
                            while k < *num_windows.offset(j as isize) {
                                let fresh35 = &mut (*win_fp.offset(k as isize));
                                *fresh35 = std::ptr::null_mut::<HTS_File>();
                                k = k.wrapping_add(1);
                            }
                            k = 0 as libc::c_int as size_t;
                            while k < *num_windows.offset(j as isize) {
                                matched_size = 0 as libc::c_int as size_t;
                                if HTS_get_token_from_string_with_separator(
                                    *(*temp_stream_win.offset(j as isize)).offset(k as isize),
                                    &mut matched_size,
                                    buff2.as_mut_ptr(),
                                    '-' as i32 as libc::c_char,
                                ) as libc::c_int
                                    == 1 as libc::c_int
                                {
                                    s = atoi(buff2.as_mut_ptr()) as size_t;
                                    e = atoi(
                                        &mut *(*(*temp_stream_win.offset(j as isize))
                                            .offset(k as isize))
                                        .offset(matched_size as isize),
                                    ) as size_t;
                                    HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                                    let fresh36 = &mut (*win_fp.offset(k as isize));
                                    *fresh36 = HTS_fopen_from_fp(
                                        fp,
                                        e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                                    );
                                    HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                                }
                                k = k.wrapping_add(1);
                            }
                            if HTS_Window_load(
                                &mut *((*ms).window).offset(j as isize),
                                win_fp,
                                *num_windows.offset(j as isize),
                            ) as libc::c_int
                                != 1 as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            k = 0 as libc::c_int as size_t;
                            while k < *num_windows.offset(j as isize) {
                                HTS_fclose(*win_fp.offset(k as isize));
                                k = k.wrapping_add(1);
                            }
                            free(win_fp as *mut libc::c_void);
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            pdf_fp = std::ptr::null_mut::<HTS_File>();
                            tree_fp = std::ptr::null_mut::<HTS_File>();
                            matched_size = 0 as libc::c_int as size_t;
                            if HTS_get_token_from_string_with_separator(
                                *temp_stream_pdf.offset(j as isize),
                                &mut matched_size,
                                buff2.as_mut_ptr(),
                                '-' as i32 as libc::c_char,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                s = atoi(buff2.as_mut_ptr()) as size_t;
                                e = atoi(
                                    &mut *(*temp_stream_pdf.offset(j as isize))
                                        .offset(matched_size as isize),
                                ) as size_t;
                                HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                                pdf_fp = HTS_fopen_from_fp(
                                    fp,
                                    e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                                );
                                HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                            }
                            matched_size = 0 as libc::c_int as size_t;
                            if HTS_get_token_from_string_with_separator(
                                *temp_stream_tree.offset(j as isize),
                                &mut matched_size,
                                buff2.as_mut_ptr(),
                                '-' as i32 as libc::c_char,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                s = atoi(buff2.as_mut_ptr()) as size_t;
                                e = atoi(
                                    &mut *(*temp_stream_tree.offset(j as isize))
                                        .offset(matched_size as isize),
                                ) as size_t;
                                HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                                tree_fp = HTS_fopen_from_fp(
                                    fp,
                                    e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                                );
                                HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                            }
                            if HTS_Model_load(
                                &mut *(*((*ms).stream).offset(i as isize)).offset(j as isize),
                                pdf_fp,
                                tree_fp,
                                *vector_length.offset(j as isize),
                                *num_windows.offset(j as isize),
                                *is_msd.offset(j as isize),
                            ) as libc::c_int
                                != 1 as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            HTS_fclose(pdf_fp);
                            HTS_fclose(tree_fp);
                            j = j.wrapping_add(1);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            pdf_fp = std::ptr::null_mut::<HTS_File>();
                            tree_fp = std::ptr::null_mut::<HTS_File>();
                            matched_size = 0 as libc::c_int as size_t;
                            if HTS_get_token_from_string_with_separator(
                                *temp_gv_pdf.offset(j as isize),
                                &mut matched_size,
                                buff2.as_mut_ptr(),
                                '-' as i32 as libc::c_char,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                s = atoi(buff2.as_mut_ptr()) as size_t;
                                e = atoi(
                                    &mut *(*temp_gv_pdf.offset(j as isize))
                                        .offset(matched_size as isize),
                                ) as size_t;
                                HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                                pdf_fp = HTS_fopen_from_fp(
                                    fp,
                                    e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                                );
                                HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                            }
                            matched_size = 0 as libc::c_int as size_t;
                            if HTS_get_token_from_string_with_separator(
                                *temp_gv_tree.offset(j as isize),
                                &mut matched_size,
                                buff2.as_mut_ptr(),
                                '-' as i32 as libc::c_char,
                            ) as libc::c_int
                                == 1 as libc::c_int
                            {
                                s = atoi(buff2.as_mut_ptr()) as size_t;
                                e = atoi(
                                    &mut *(*temp_gv_tree.offset(j as isize))
                                        .offset(matched_size as isize),
                                ) as size_t;
                                HTS_fseek(fp, s as libc::c_long, 1 as libc::c_int);
                                tree_fp = HTS_fopen_from_fp(
                                    fp,
                                    e.wrapping_sub(s).wrapping_add(1 as libc::c_int as size_t),
                                );
                                HTS_fseek(fp, start_of_data, 0 as libc::c_int);
                            }
                            if *use_gv.offset(j as isize) as libc::c_int == 1 as libc::c_int
                                && HTS_Model_load(
                                    &mut *(*((*ms).gv).offset(i as isize)).offset(j as isize),
                                    pdf_fp,
                                    tree_fp,
                                    *vector_length.offset(j as isize),
                                    1 as libc::c_int as size_t,
                                    0 as libc::c_int as HTS_Boolean,
                                ) as libc::c_int
                                    != 1 as libc::c_int
                            {
                                error = 1 as libc::c_int as HTS_Boolean;
                            }
                            HTS_fclose(pdf_fp);
                            HTS_fclose(tree_fp);
                            j = j.wrapping_add(1);
                        }
                        if !temp_duration_pdf.is_null() {
                            free(temp_duration_pdf as *mut libc::c_void);
                        }
                        if !temp_duration_tree.is_null() {
                            free(temp_duration_tree as *mut libc::c_void);
                        }
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            k = 0 as libc::c_int as size_t;
                            while k < *num_windows.offset(j as isize) {
                                if !(*(*temp_stream_win.offset(j as isize)).offset(k as isize))
                                    .is_null()
                                {
                                    free(*(*temp_stream_win.offset(j as isize)).offset(k as isize)
                                        as *mut libc::c_void);
                                }
                                k = k.wrapping_add(1);
                            }
                            free(*temp_stream_win.offset(j as isize) as *mut libc::c_void);
                            j = j.wrapping_add(1);
                        }
                        free(temp_stream_win as *mut libc::c_void);
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if !(*temp_stream_pdf.offset(j as isize)).is_null() {
                                free(*temp_stream_pdf.offset(j as isize) as *mut libc::c_void);
                            }
                            j = j.wrapping_add(1);
                        }
                        free(temp_stream_pdf as *mut libc::c_void);
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if !(*temp_stream_tree.offset(j as isize)).is_null() {
                                free(*temp_stream_tree.offset(j as isize) as *mut libc::c_void);
                            }
                            j = j.wrapping_add(1);
                        }
                        free(temp_stream_tree as *mut libc::c_void);
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if !(*temp_gv_pdf.offset(j as isize)).is_null() {
                                free(*temp_gv_pdf.offset(j as isize) as *mut libc::c_void);
                            }
                            j = j.wrapping_add(1);
                        }
                        free(temp_gv_pdf as *mut libc::c_void);
                        j = 0 as libc::c_int as size_t;
                        while j < (*ms).num_streams {
                            if !(*temp_gv_tree.offset(j as isize)).is_null() {
                                free(*temp_gv_tree.offset(j as isize) as *mut libc::c_void);
                            }
                            j = j.wrapping_add(1);
                        }
                        free(temp_gv_tree as *mut libc::c_void);
                        HTS_fclose(fp);
                        if error as libc::c_int != 0 as libc::c_int {
                            break;
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    }
    if !gv_off_context.is_null() {
        sprintf(
            buff1.as_mut_ptr(),
            b"GV-Off { %s }\0" as *const u8 as *const libc::c_char,
            gv_off_context,
        );
        gv_off_context_fp = HTS_fopen_from_data(
            buff1.as_mut_ptr() as *mut libc::c_void,
            (strlen(buff1.as_mut_ptr())).wrapping_add(1 as libc::c_int as libc::c_ulong),
        );
        (*ms).gv_off_context = HTS_calloc(
            1 as libc::c_int as size_t,
            ::core::mem::size_of::<HTS_Question>() as libc::c_ulong,
        ) as *mut HTS_Question;
        HTS_Question_initialize((*ms).gv_off_context);
        HTS_Question_load((*ms).gv_off_context, gv_off_context_fp);
        HTS_fclose(gv_off_context_fp);
        free(gv_off_context as *mut libc::c_void);
    }
    if !stream_type_list.is_null() {
        i = 0 as libc::c_int as size_t;
        while i < (*ms).num_streams {
            if !(*stream_type_list.offset(i as isize)).is_null() {
                free(*stream_type_list.offset(i as isize) as *mut libc::c_void);
            }
            i = i.wrapping_add(1);
        }
        free(stream_type_list as *mut libc::c_void);
    }
    if !vector_length.is_null() {
        free(vector_length as *mut libc::c_void);
    }
    if !is_msd.is_null() {
        free(is_msd as *mut libc::c_void);
    }
    if !num_windows.is_null() {
        free(num_windows as *mut libc::c_void);
    }
    if !use_gv.is_null() {
        free(use_gv as *mut libc::c_void);
    }
    (error == 0) as libc::c_int as HTS_Boolean
}

pub unsafe fn HTS_ModelSet_get_sampling_frequency(mut ms: *mut HTS_ModelSet) -> size_t {
    (*ms).sampling_frequency
}

pub unsafe fn HTS_ModelSet_get_fperiod(mut ms: *mut HTS_ModelSet) -> size_t {
    (*ms).frame_period
}

pub unsafe fn HTS_ModelSet_get_option(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
) -> *const libc::c_char {
    *((*ms).option).offset(stream_index as isize)
}

pub unsafe fn HTS_ModelSet_get_gv_flag(
    mut ms: *mut HTS_ModelSet,
    mut string: *const libc::c_char,
) -> HTS_Boolean {
    if ((*ms).gv_off_context).is_null() {
        1 as libc::c_int as HTS_Boolean
    } else if HTS_Question_match((*ms).gv_off_context, string) as libc::c_int == 1 as libc::c_int {
        return 0 as libc::c_int as HTS_Boolean;
    } else {
        return 1 as libc::c_int as HTS_Boolean;
    }
}

pub unsafe fn HTS_ModelSet_get_nstate(mut ms: *mut HTS_ModelSet) -> size_t {
    (*ms).num_states
}

pub unsafe fn HTS_ModelSet_get_fullcontext_label_format(
    mut ms: *mut HTS_ModelSet,
) -> *const libc::c_char {
    (*ms).fullcontext_format
}

pub unsafe fn HTS_ModelSet_get_fullcontext_label_version(
    mut ms: *mut HTS_ModelSet,
) -> *const libc::c_char {
    (*ms).fullcontext_version
}

pub unsafe fn HTS_ModelSet_get_nstream(mut ms: *mut HTS_ModelSet) -> size_t {
    (*ms).num_streams
}

pub unsafe fn HTS_ModelSet_get_nvoices(mut ms: *mut HTS_ModelSet) -> size_t {
    (*ms).num_voices
}

pub unsafe fn HTS_ModelSet_get_vector_length(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
) -> size_t {
    (*(*((*ms).stream).offset(0 as libc::c_int as isize)).offset(stream_index as isize))
        .vector_length
}

pub unsafe fn HTS_ModelSet_is_msd(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    (*(*((*ms).stream).offset(0 as libc::c_int as isize)).offset(stream_index as isize)).is_msd
}

pub unsafe fn HTS_ModelSet_get_window_size(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
) -> size_t {
    (*((*ms).window).offset(stream_index as isize)).size
}

pub unsafe fn HTS_ModelSet_get_window_left_width(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
    mut window_index: size_t,
) -> libc::c_int {
    *((*((*ms).window).offset(stream_index as isize)).l_width).offset(window_index as isize)
}

pub unsafe fn HTS_ModelSet_get_window_right_width(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
    mut window_index: size_t,
) -> libc::c_int {
    *((*((*ms).window).offset(stream_index as isize)).r_width).offset(window_index as isize)
}

pub unsafe fn HTS_ModelSet_get_window_coefficient(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
    mut window_index: size_t,
    mut coefficient_index: size_t,
) -> libc::c_double {
    *(*((*((*ms).window).offset(stream_index as isize)).coefficient).offset(window_index as isize))
        .offset(coefficient_index as isize)
}

pub unsafe fn HTS_ModelSet_get_window_max_width(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
) -> size_t {
    (*((*ms).window).offset(stream_index as isize)).max_width
}

pub unsafe fn HTS_ModelSet_use_gv(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
) -> HTS_Boolean {
    if (*(*((*ms).gv).offset(0 as libc::c_int as isize)).offset(stream_index as isize))
        .vector_length
        != 0 as libc::c_int as size_t
    {
        1 as libc::c_int as HTS_Boolean
    } else {
        0 as libc::c_int as HTS_Boolean
    }
}
unsafe fn HTS_Model_add_parameter(
    mut model: *mut HTS_Model,
    mut state_index: size_t,
    mut string: *const libc::c_char,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
    mut msd: *mut libc::c_double,
    mut weight: libc::c_double,
) {
    let mut i: size_t = 0;
    let mut tree_index: size_t = 0;
    let mut pdf_index: size_t = 0;
    let mut len: size_t = (*model).vector_length * (*model).num_windows;
    HTS_Model_get_index(model, state_index, string, &mut tree_index, &mut pdf_index);
    i = 0 as libc::c_int as size_t;
    while i < len {
        *mean.offset(i as isize) += weight
            * *(*(*((*model).pdf).offset(tree_index as isize)).offset(pdf_index as isize))
                .offset(i as isize) as libc::c_double;
        *vari.offset(i as isize) += weight
            * *(*(*((*model).pdf).offset(tree_index as isize)).offset(pdf_index as isize))
                .offset(i.wrapping_add(len) as isize) as libc::c_double;
        i = i.wrapping_add(1);
    }
    if !msd.is_null() && (*model).is_msd as libc::c_int == 1 as libc::c_int {
        *msd += weight
            * *(*(*((*model).pdf).offset(tree_index as isize)).offset(pdf_index as isize))
                .offset(len.wrapping_add(len) as isize) as libc::c_double;
    }
}

pub unsafe fn HTS_ModelSet_get_duration_index(
    mut ms: *mut HTS_ModelSet,
    mut voice_index: size_t,
    mut string: *const libc::c_char,
    mut tree_index: *mut size_t,
    mut pdf_index: *mut size_t,
) {
    HTS_Model_get_index(
        &mut *((*ms).duration).offset(voice_index as isize),
        2 as libc::c_int as size_t,
        string,
        tree_index,
        pdf_index,
    );
}

pub unsafe fn HTS_ModelSet_get_duration(
    mut ms: *mut HTS_ModelSet,
    mut string: *const libc::c_char,
    mut iw: &Vec<f64>,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
) {
    let mut i: size_t = 0;
    let mut len: size_t = (*ms).num_states;
    i = 0 as libc::c_int as size_t;
    while i < len {
        *mean.offset(i as isize) = 0.0f64;
        *vari.offset(i as isize) = 0.0f64;
        i = i.wrapping_add(1);
    }
    i = 0 as libc::c_int as size_t;
    while i < (*ms).num_voices {
        if iw[i as usize] != 0.0f64 {
            HTS_Model_add_parameter(
                &mut *((*ms).duration).offset(i as isize),
                2 as libc::c_int as size_t,
                string,
                mean,
                vari,
                std::ptr::null_mut::<libc::c_double>(),
                iw[i as usize],
            );
        }
        i = i.wrapping_add(1);
    }
}

pub unsafe fn HTS_ModelSet_get_parameter_index(
    mut ms: *mut HTS_ModelSet,
    mut voice_index: size_t,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut string: *const libc::c_char,
    mut tree_index: *mut size_t,
    mut pdf_index: *mut size_t,
) {
    HTS_Model_get_index(
        &mut *(*((*ms).stream).offset(voice_index as isize)).offset(stream_index as isize),
        state_index,
        string,
        tree_index,
        pdf_index,
    );
}

pub unsafe fn HTS_ModelSet_get_parameter(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
    mut state_index: size_t,
    mut string: *const libc::c_char,
    mut iw: &mut Vec<Vec<f64>>,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
    mut msd: *mut libc::c_double,
) {
    let mut i: size_t = 0;
    let mut len: size_t = (*(*((*ms).stream).offset(0 as libc::c_int as isize))
        .offset(stream_index as isize))
    .vector_length
        * (*(*((*ms).stream).offset(0 as libc::c_int as isize)).offset(stream_index as isize))
            .num_windows;
    i = 0 as libc::c_int as size_t;
    while i < len {
        *mean.offset(i as isize) = 0.0f64;
        *vari.offset(i as isize) = 0.0f64;
        i = i.wrapping_add(1);
    }
    if !msd.is_null() {
        *msd = 0.0f64;
    }
    i = 0 as libc::c_int as size_t;
    while i < (*ms).num_voices {
        if iw[i as usize][stream_index as usize] != 0.0f64 {
            HTS_Model_add_parameter(
                &mut *(*((*ms).stream).offset(i as isize)).offset(stream_index as isize),
                state_index,
                string,
                mean,
                vari,
                msd,
                iw[i as usize][stream_index as usize],
            );
        }
        i = i.wrapping_add(1);
    }
}

pub unsafe fn HTS_ModelSet_get_gv_index(
    mut ms: *mut HTS_ModelSet,
    mut voice_index: size_t,
    mut stream_index: size_t,
    mut string: *const libc::c_char,
    mut tree_index: *mut size_t,
    mut pdf_index: *mut size_t,
) {
    HTS_Model_get_index(
        &mut *(*((*ms).gv).offset(voice_index as isize)).offset(stream_index as isize),
        2 as libc::c_int as size_t,
        string,
        tree_index,
        pdf_index,
    );
}

pub unsafe fn HTS_ModelSet_get_gv(
    mut ms: *mut HTS_ModelSet,
    mut stream_index: size_t,
    mut string: *const libc::c_char,
    mut iw: &Vec<Vec<f64>>,
    mut mean: *mut libc::c_double,
    mut vari: *mut libc::c_double,
) {
    let mut i: size_t = 0;
    let mut len: size_t = (*(*((*ms).stream).offset(0 as libc::c_int as isize))
        .offset(stream_index as isize))
    .vector_length;
    i = 0 as libc::c_int as size_t;
    while i < len {
        *mean.offset(i as isize) = 0.0f64;
        *vari.offset(i as isize) = 0.0f64;
        i = i.wrapping_add(1);
    }
    i = 0 as libc::c_int as size_t;
    while i < (*ms).num_voices {
        if iw[i as usize][stream_index as usize] != 0.0f64 {
            HTS_Model_add_parameter(
                &mut *(*((*ms).gv).offset(i as isize)).offset(stream_index as isize),
                2 as libc::c_int as size_t,
                string,
                mean,
                vari,
                std::ptr::null_mut::<libc::c_double>(),
                iw[i as usize][stream_index as usize],
            );
        }
        i = i.wrapping_add(1);
    }
}

#[cfg(all(test, feature = "htsvoice"))]
mod tests {
    use std::ffi::CString;

    use super::*;
    use crate::{model::ModelSet, HTS_ModelSet};

    fn load_models() -> (HTS_ModelSet, ModelSet) {
        let model_str = CString::new("models/nitech_jp_atr503_m001.htsvoice").unwrap();
        let voices = &[model_str.as_ptr() as *mut i8];

        let mut hts = HTS_ModelSet_initialize();
        unsafe { HTS_ModelSet_load(&mut hts, voices.as_ptr() as *mut *mut i8, 1) };

        let jsyn =
            ModelSet::load_htsvoice_files(&["models/nitech_jp_atr503_m001.htsvoice"]).unwrap();
        (hts, jsyn)
    }

    #[test]
    fn get_sampling_frequency() {
        let (mut hts, jsyn) = load_models();
        assert_eq!(
            unsafe { HTS_ModelSet_get_sampling_frequency(&mut hts) },
            jsyn.get_sampling_frequency() as u64
        );
    }
    #[test]
    fn get_sampling_fperiod() {
        let (mut hts, jsyn) = load_models();
        assert_eq!(
            unsafe { HTS_ModelSet_get_fperiod(&mut hts) },
            jsyn.get_fperiod() as u64
        );
    }
}
