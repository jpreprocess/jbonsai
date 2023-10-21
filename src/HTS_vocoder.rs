#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
use crate::{util::*, HTS_calloc, HTS_free};
extern "C" {
    fn cos(_: libc::c_double) -> libc::c_double;
    fn exp(_: libc::c_double) -> libc::c_double;
    fn log(_: libc::c_double) -> libc::c_double;
    fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
}

#[derive(Clone)]
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


static HTS_pade: [f64; 21] = [
    1.00000000000f64,
    1.00000000000f64,
    0.00000000000f64,
    1.00000000000f64,
    0.00000000000f64,
    0.00000000000f64,
    1.00000000000f64,
    0.00000000000f64,
    0.00000000000f64,
    0.00000000000f64,
    1.00000000000f64,
    0.49992730000f64,
    0.10670050000f64,
    0.01170221000f64,
    0.00056562790f64,
    1.00000000000f64,
    0.49993910000f64,
    0.11070980000f64,
    0.01369984000f64,
    0.00095648530f64,
    0.00003041721f64,
];

unsafe fn HTS_movem(mut a: *mut libc::c_double, mut b: *mut libc::c_double, nitem: libc::c_int) {
    let mut i: libc::c_long = nitem as libc::c_long;
    if a > b {
        loop {
            let fresh0 = i;
            i -= 1;
            if fresh0 == 0 {
                break;
            }
            let fresh1 = a;
            a = a.offset(1);
            let fresh2 = b;
            b = b.offset(1);
            *fresh2 = *fresh1;
        }
    } else {
        a = a.offset(i as isize);
        b = b.offset(i as isize);
        loop {
            let fresh3 = i;
            i -= 1;
            if fresh3 == 0 {
                break;
            }
            a = a.offset(-1);
            b = b.offset(-1);
            *b = *a;
        }
    };
}
unsafe fn HTS_mlsafir(
    x: libc::c_double,
    mut b: *const libc::c_double,
    m: libc::c_int,
    a: libc::c_double,
    aa: libc::c_double,
    mut d: *mut libc::c_double,
) -> libc::c_double {
    let mut y: libc::c_double = 0.0f64;
    let mut i: libc::c_int = 0;
    *d.offset(0 as libc::c_int as isize) = x;
    *d.offset(1 as libc::c_int as isize) =
        aa * *d.offset(0 as libc::c_int as isize) + a * *d.offset(1 as libc::c_int as isize);
    i = 2 as libc::c_int;
    while i <= m {
        *d.offset(i as isize) += a
            * (*d.offset((i + 1 as libc::c_int) as isize)
                - *d.offset((i - 1 as libc::c_int) as isize));
        i += 1;
    }
    i = 2 as libc::c_int;
    while i <= m {
        y += *d.offset(i as isize) * *b.offset(i as isize);
        i += 1;
    }
    i = m + 1 as libc::c_int;
    while i > 1 as libc::c_int {
        *d.offset(i as isize) = *d.offset((i - 1 as libc::c_int) as isize);
        i -= 1;
    }
    y
}
unsafe fn HTS_mlsadf1(
    mut x: libc::c_double,
    mut b: *const libc::c_double,
    _m: libc::c_int,
    a: libc::c_double,
    aa: libc::c_double,
    pd: libc::c_int,
    mut d: *mut libc::c_double,
    mut ppade: *const libc::c_double,
) -> libc::c_double {
    let mut v: libc::c_double = 0.;
    let mut out: libc::c_double = 0.0f64;
    let mut pt: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut i: libc::c_int = 0;
    pt = &mut *d.offset((pd + 1 as libc::c_int) as isize) as *mut libc::c_double;
    i = pd;
    while i >= 1 as libc::c_int {
        *d.offset(i as isize) =
            aa * *pt.offset((i - 1 as libc::c_int) as isize) + a * *d.offset(i as isize);
        *pt.offset(i as isize) = *d.offset(i as isize) * *b.offset(1 as libc::c_int as isize);
        v = *pt.offset(i as isize) * *ppade.offset(i as isize);
        x += if 1 as libc::c_int & i != 0 { v } else { -v };
        out += v;
        i -= 1;
    }
    *pt.offset(0 as libc::c_int as isize) = x;
    out += x;
    out
}
unsafe fn HTS_mlsadf2(
    mut x: libc::c_double,
    mut b: *const libc::c_double,
    m: libc::c_int,
    a: libc::c_double,
    aa: libc::c_double,
    pd: libc::c_int,
    mut d: *mut libc::c_double,
    mut ppade: *const libc::c_double,
) -> libc::c_double {
    let mut v: libc::c_double = 0.;
    let mut out: libc::c_double = 0.0f64;
    let mut pt: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut i: libc::c_int = 0;
    pt = &mut *d.offset((pd * (m + 2 as libc::c_int)) as isize) as *mut libc::c_double;
    i = pd;
    while i >= 1 as libc::c_int {
        *pt.offset(i as isize) = HTS_mlsafir(
            *pt.offset((i - 1 as libc::c_int) as isize),
            b,
            m,
            a,
            aa,
            &mut *d.offset(((i - 1 as libc::c_int) * (m + 2 as libc::c_int)) as isize),
        );
        v = *pt.offset(i as isize) * *ppade.offset(i as isize);
        x += if 1 as libc::c_int & i != 0 { v } else { -v };
        out += v;
        i -= 1;
    }
    *pt.offset(0 as libc::c_int as isize) = x;
    out += x;
    out
}
unsafe fn HTS_mlsadf(
    mut x: libc::c_double,
    mut b: *const libc::c_double,
    m: libc::c_int,
    a: libc::c_double,
    pd: libc::c_int,
    mut d: *mut libc::c_double,
) -> libc::c_double {
    let aa: libc::c_double = 1 as libc::c_int as libc::c_double - a * a;
    let mut ppade: *const libc::c_double = &*HTS_pade
        .as_ptr()
        .offset((pd * (pd + 1 as libc::c_int) / 2 as libc::c_int) as isize)
        as *const libc::c_double;
    x = HTS_mlsadf1(x, b, m, a, aa, pd, d, ppade);
    x = HTS_mlsadf2(
        x,
        b,
        m,
        a,
        aa,
        pd,
        &mut *d.offset((2 as libc::c_int * (pd + 1 as libc::c_int)) as isize),
        ppade,
    );
    x
}
unsafe fn HTS_rnd(mut next: *mut libc::c_ulong) -> libc::c_double {
    let mut r: libc::c_double = 0.;
    *next = (*next)
        .wrapping_mul(1103515245 as libc::c_long as libc::c_ulong)
        .wrapping_add(12345 as libc::c_int as libc::c_ulong);
    r = (*next)
        .wrapping_div(65536 as libc::c_long as libc::c_ulong)
        .wrapping_rem(32768 as libc::c_long as libc::c_ulong) as libc::c_double;
    r / 32767 as libc::c_int as libc::c_double
}
unsafe fn HTS_nrandom(v: &mut HTS_Vocoder) -> libc::c_double {
    if v.sw as libc::c_int == 0 as libc::c_int {
        v.sw = 1 as libc::c_int as libc::c_uchar;
        loop {
            v.r1 = 2 as libc::c_int as libc::c_double * HTS_rnd(&mut v.next)
                - 1 as libc::c_int as libc::c_double;
            v.r2 = 2 as libc::c_int as libc::c_double * HTS_rnd(&mut v.next)
                - 1 as libc::c_int as libc::c_double;
            v.s = v.r1 * v.r1 + v.r2 * v.r2;
            if !(v.s > 1 as libc::c_int as libc::c_double
                || v.s == 0 as libc::c_int as libc::c_double)
            {
                break;
            }
        }
        v.s = sqrt(-(2 as libc::c_int) as libc::c_double * log(v.s) / v.s);
        v.r1 * v.s
    } else {
        v.sw = 0 as libc::c_int as libc::c_uchar;
        v.r2 * v.s
    }
}
unsafe fn HTS_mseq(v: &mut HTS_Vocoder) -> libc::c_int {
    let mut x0: libc::c_int = 0;
    let mut x28: libc::c_int = 0;
    v.x >>= 1 as libc::c_int;
    if v.x & 0x1 as libc::c_int != 0 {
        x0 = 1 as libc::c_int;
    } else {
        x0 = -(1 as libc::c_int);
    }
    if v.x & 0x10000000 as libc::c_int != 0 {
        x28 = 1 as libc::c_int;
    } else {
        x28 = -(1 as libc::c_int);
    }
    if x0 + x28 != 0 {
        v.x &= 0x7fffffff as libc::c_int;
    } else {
        v.x = (v.x as libc::c_uint | 0x80000000 as libc::c_uint) as libc::c_int;
    }
    x0
}
unsafe fn HTS_mc2b(
    mut mc: *mut libc::c_double,
    mut b: *mut libc::c_double,
    mut m: libc::c_int,
    a: libc::c_double,
) {
    if mc != b {
        if a != 0.0f64 {
            *b.offset(m as isize) = *mc.offset(m as isize);
            m -= 1;
            while m >= 0 as libc::c_int {
                *b.offset(m as isize) =
                    *mc.offset(m as isize) - a * *b.offset((m + 1 as libc::c_int) as isize);
                m -= 1;
            }
        } else {
            HTS_movem(mc, b, m + 1 as libc::c_int);
        }
    } else if a != 0.0f64 {
        m -= 1;
        while m >= 0 as libc::c_int {
            *b.offset(m as isize) -= a * *b.offset((m + 1 as libc::c_int) as isize);
            m -= 1;
        }
    }
}
unsafe fn HTS_b2mc(
    mut b: *const libc::c_double,
    mut mc: *mut libc::c_double,
    mut m: libc::c_int,
    a: libc::c_double,
) {
    let mut d: libc::c_double = 0.;
    let mut o: libc::c_double = 0.;
    let fresh4 = &mut (*mc.offset(m as isize));
    *fresh4 = *b.offset(m as isize);
    d = *fresh4;
    m -= 1;
    while m >= 0 as libc::c_int {
        o = *b.offset(m as isize) + a * d;
        d = *b.offset(m as isize);
        *mc.offset(m as isize) = o;
        m -= 1;
    }
}
unsafe fn HTS_freqt(
    v: &mut HTS_Vocoder,
    mut c1: *const libc::c_double,
    m1: libc::c_int,
    mut c2: *mut libc::c_double,
    m2: libc::c_int,
    a: libc::c_double,
) {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let b: libc::c_double = 1 as libc::c_int as libc::c_double - a * a;
    let mut g: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    if m2 as size_t > v.freqt_size {
        if !(v.freqt_buff).is_null() {
            HTS_free(v.freqt_buff as *mut libc::c_void);
        }
        v.freqt_buff = HTS_calloc(
            (m2 + m2 + 2 as libc::c_int) as size_t,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.freqt_size = m2 as size_t;
    }
    g = (v.freqt_buff)
        .offset(v.freqt_size as isize)
        .offset(1 as libc::c_int as isize);
    i = 0 as libc::c_int;
    while i < m2 + 1 as libc::c_int {
        *g.offset(i as isize) = 0.0f64;
        i += 1;
    }
    i = -m1;
    while i <= 0 as libc::c_int {
        if 0 as libc::c_int <= m2 {
            let fresh5 = &mut (*(v.freqt_buff).offset(0 as libc::c_int as isize));
            *fresh5 = *g.offset(0 as libc::c_int as isize);
            *g.offset(0 as libc::c_int as isize) = *c1.offset(-i as isize) + a * *fresh5;
        }
        if 1 as libc::c_int <= m2 {
            let fresh6 = &mut (*(v.freqt_buff).offset(1 as libc::c_int as isize));
            *fresh6 = *g.offset(1 as libc::c_int as isize);
            *g.offset(1 as libc::c_int as isize) =
                b * *(v.freqt_buff).offset(0 as libc::c_int as isize) + a * *fresh6;
        }
        j = 2 as libc::c_int;
        while j <= m2 {
            let fresh7 = &mut (*(v.freqt_buff).offset(j as isize));
            *fresh7 = *g.offset(j as isize);
            *g.offset(j as isize) = *(v.freqt_buff).offset((j - 1 as libc::c_int) as isize)
                + a * (*fresh7 - *g.offset((j - 1 as libc::c_int) as isize));
            j += 1;
        }
        i += 1;
    }
    HTS_movem(g, c2, m2 + 1 as libc::c_int);
}
unsafe fn HTS_c2ir(
    mut c: *const libc::c_double,
    nc: libc::c_int,
    mut h: *mut libc::c_double,
    leng: libc::c_int,
) {
    let mut n: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut upl: libc::c_int = 0;
    let mut d: libc::c_double = 0.;
    *h.offset(0 as libc::c_int as isize) = exp(*c.offset(0 as libc::c_int as isize));
    n = 1 as libc::c_int;
    while n < leng {
        d = 0 as libc::c_int as libc::c_double;
        upl = if n >= nc { nc - 1 as libc::c_int } else { n };
        k = 1 as libc::c_int;
        while k <= upl {
            d += k as libc::c_double * *c.offset(k as isize) * *h.offset((n - k) as isize);
            k += 1;
        }
        *h.offset(n as isize) = d / n as libc::c_double;
        n += 1;
    }
}
unsafe fn HTS_b2en(
    v: &mut HTS_Vocoder,
    mut b: *const libc::c_double,
    m: libc::c_int,
    a: libc::c_double,
) -> libc::c_double {
    let mut i: libc::c_int = 0;
    let mut en: libc::c_double = 0.0f64;
    let mut cep: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut ir: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    if v.spectrum2en_size < m as size_t {
        if !(v.spectrum2en_buff).is_null() {
            HTS_free(v.spectrum2en_buff as *mut libc::c_void);
        }
        v.spectrum2en_buff = HTS_calloc(
            (m + 1 as libc::c_int + 2 as libc::c_int * 576 as libc::c_int) as size_t,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.spectrum2en_size = m as size_t;
    }
    cep = (v.spectrum2en_buff)
        .offset(m as isize)
        .offset(1 as libc::c_int as isize);
    ir = cep.offset(576 as libc::c_int as isize);
    HTS_b2mc(b, v.spectrum2en_buff, m, a);
    HTS_freqt(
        v,
        v.spectrum2en_buff,
        m,
        cep,
        576 as libc::c_int - 1 as libc::c_int,
        -a,
    );
    HTS_c2ir(cep, 576 as libc::c_int, ir, 576 as libc::c_int);
    i = 0 as libc::c_int;
    while i < 576 as libc::c_int {
        en += *ir.offset(i as isize) * *ir.offset(i as isize);
        i += 1;
    }
    en
}
unsafe fn HTS_ignorm(
    mut c1: *mut libc::c_double,
    mut c2: *mut libc::c_double,
    mut m: libc::c_int,
    g: libc::c_double,
) {
    let mut k: libc::c_double = 0.;
    if g != 0.0f64 {
        k = pow(*c1.offset(0 as libc::c_int as isize), g);
        while m >= 1 as libc::c_int {
            *c2.offset(m as isize) = k * *c1.offset(m as isize);
            m -= 1;
        }
        *c2.offset(0 as libc::c_int as isize) = (k - 1.0f64) / g;
    } else {
        HTS_movem(
            &mut *c1.offset(1 as libc::c_int as isize),
            &mut *c2.offset(1 as libc::c_int as isize),
            m,
        );
        *c2.offset(0 as libc::c_int as isize) = log(*c1.offset(0 as libc::c_int as isize));
    };
}
unsafe fn HTS_gnorm(
    mut c1: *mut libc::c_double,
    mut c2: *mut libc::c_double,
    mut m: libc::c_int,
    g: libc::c_double,
) {
    let mut k: libc::c_double = 0.;
    if g != 0.0f64 {
        k = 1.0f64 + g * *c1.offset(0 as libc::c_int as isize);
        while m >= 1 as libc::c_int {
            *c2.offset(m as isize) = *c1.offset(m as isize) / k;
            m -= 1;
        }
        *c2.offset(0 as libc::c_int as isize) = pow(k, 1.0f64 / g);
    } else {
        HTS_movem(
            &mut *c1.offset(1 as libc::c_int as isize),
            &mut *c2.offset(1 as libc::c_int as isize),
            m,
        );
        *c2.offset(0 as libc::c_int as isize) = exp(*c1.offset(0 as libc::c_int as isize));
    };
}
unsafe fn HTS_lsp2lpc(
    v: &mut HTS_Vocoder,
    mut lsp: *mut libc::c_double,
    mut a: *mut libc::c_double,
    m: libc::c_int,
) {
    let mut i: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut mh1: libc::c_int = 0;
    let mut mh2: libc::c_int = 0;
    let mut flag_odd: libc::c_int = 0;
    let mut xx: libc::c_double = 0.;
    let mut xf: libc::c_double = 0.;
    let mut xff: libc::c_double = 0.;
    let mut p: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut q: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut a0: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut a1: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut a2: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut b0: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut b1: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    let mut b2: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    flag_odd = 0 as libc::c_int;
    if m % 2 as libc::c_int == 0 as libc::c_int {
        mh2 = m / 2 as libc::c_int;
        mh1 = mh2;
    } else {
        mh1 = (m + 1 as libc::c_int) / 2 as libc::c_int;
        mh2 = (m - 1 as libc::c_int) / 2 as libc::c_int;
        flag_odd = 1 as libc::c_int;
    }
    if m as size_t > v.lsp2lpc_size {
        if !(v.lsp2lpc_buff).is_null() {
            HTS_free(v.lsp2lpc_buff as *mut libc::c_void);
        }
        v.lsp2lpc_buff = HTS_calloc(
            (5 as libc::c_int * m + 6 as libc::c_int) as size_t,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.lsp2lpc_size = m as size_t;
    }
    p = (v.lsp2lpc_buff).offset(m as isize);
    q = p.offset(mh1 as isize);
    a0 = q.offset(mh2 as isize);
    a1 = a0.offset((mh1 + 1 as libc::c_int) as isize);
    a2 = a1.offset((mh1 + 1 as libc::c_int) as isize);
    b0 = a2.offset((mh1 + 1 as libc::c_int) as isize);
    b1 = b0.offset((mh2 + 1 as libc::c_int) as isize);
    b2 = b1.offset((mh2 + 1 as libc::c_int) as isize);
    HTS_movem(lsp, v.lsp2lpc_buff, m);
    i = 0 as libc::c_int;
    while i < mh1 + 1 as libc::c_int {
        *a0.offset(i as isize) = 0.0f64;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < mh1 + 1 as libc::c_int {
        *a1.offset(i as isize) = 0.0f64;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < mh1 + 1 as libc::c_int {
        *a2.offset(i as isize) = 0.0f64;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < mh2 + 1 as libc::c_int {
        *b0.offset(i as isize) = 0.0f64;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < mh2 + 1 as libc::c_int {
        *b1.offset(i as isize) = 0.0f64;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < mh2 + 1 as libc::c_int {
        *b2.offset(i as isize) = 0.0f64;
        i += 1;
    }
    k = 0 as libc::c_int;
    i = k;
    while i < mh1 {
        *p.offset(i as isize) = -2.0f64 * cos(*(v.lsp2lpc_buff).offset(k as isize));
        i += 1;
        k += 2 as libc::c_int;
    }
    k = 0 as libc::c_int;
    i = k;
    while i < mh2 {
        *q.offset(i as isize) =
            -2.0f64 * cos(*(v.lsp2lpc_buff).offset((k + 1 as libc::c_int) as isize));
        i += 1;
        k += 2 as libc::c_int;
    }
    xx = 1.0f64;
    xff = 0.0f64;
    xf = xff;
    k = 0 as libc::c_int;
    while k <= m {
        if flag_odd != 0 {
            *a0.offset(0 as libc::c_int as isize) = xx;
            *b0.offset(0 as libc::c_int as isize) = xx - xff;
            xff = xf;
            xf = xx;
        } else {
            *a0.offset(0 as libc::c_int as isize) = xx + xf;
            *b0.offset(0 as libc::c_int as isize) = xx - xf;
            xf = xx;
        }
        i = 0 as libc::c_int;
        while i < mh1 {
            *a0.offset((i + 1 as libc::c_int) as isize) = *a0.offset(i as isize)
                + *p.offset(i as isize) * *a1.offset(i as isize)
                + *a2.offset(i as isize);
            *a2.offset(i as isize) = *a1.offset(i as isize);
            *a1.offset(i as isize) = *a0.offset(i as isize);
            i += 1;
        }
        i = 0 as libc::c_int;
        while i < mh2 {
            *b0.offset((i + 1 as libc::c_int) as isize) = *b0.offset(i as isize)
                + *q.offset(i as isize) * *b1.offset(i as isize)
                + *b2.offset(i as isize);
            *b2.offset(i as isize) = *b1.offset(i as isize);
            *b1.offset(i as isize) = *b0.offset(i as isize);
            i += 1;
        }
        if k != 0 as libc::c_int {
            *a.offset((k - 1 as libc::c_int) as isize) =
                -0.5f64 * (*a0.offset(mh1 as isize) + *b0.offset(mh2 as isize));
        }
        xx = 0.0f64;
        k += 1;
    }
    i = m - 1 as libc::c_int;
    while i >= 0 as libc::c_int {
        *a.offset((i + 1 as libc::c_int) as isize) = -*a.offset(i as isize);
        i -= 1;
    }
    *a.offset(0 as libc::c_int as isize) = 1.0f64;
}
unsafe fn HTS_gc2gc(
    v: &mut HTS_Vocoder,
    mut c1: *mut libc::c_double,
    m1: libc::c_int,
    g1: libc::c_double,
    mut c2: *mut libc::c_double,
    m2: libc::c_int,
    g2: libc::c_double,
) {
    let mut i: libc::c_int = 0;
    let mut min: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut mk: libc::c_int = 0;
    let mut ss1: libc::c_double = 0.;
    let mut ss2: libc::c_double = 0.;
    let mut cc: libc::c_double = 0.;
    if m1 as size_t > v.gc2gc_size {
        if !(v.gc2gc_buff).is_null() {
            HTS_free(v.gc2gc_buff as *mut libc::c_void);
        }
        v.gc2gc_buff = HTS_calloc(
            (m1 + 1 as libc::c_int) as size_t,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.gc2gc_size = m1 as size_t;
    }
    HTS_movem(c1, v.gc2gc_buff, m1 + 1 as libc::c_int);
    *c2.offset(0 as libc::c_int as isize) = *(v.gc2gc_buff).offset(0 as libc::c_int as isize);
    i = 1 as libc::c_int;
    while i <= m2 {
        ss2 = 0.0f64;
        ss1 = ss2;
        min = if m1 < i { m1 } else { i - 1 as libc::c_int };
        k = 1 as libc::c_int;
        while k <= min {
            mk = i - k;
            cc = *(v.gc2gc_buff).offset(k as isize) * *c2.offset(mk as isize);
            ss2 += k as libc::c_double * cc;
            ss1 += mk as libc::c_double * cc;
            k += 1;
        }
        if i <= m1 {
            *c2.offset(i as isize) =
                *(v.gc2gc_buff).offset(i as isize) + (g2 * ss2 - g1 * ss1) / i as libc::c_double;
        } else {
            *c2.offset(i as isize) = (g2 * ss2 - g1 * ss1) / i as libc::c_double;
        }
        i += 1;
    }
}
unsafe fn HTS_mgc2mgc(
    v: &mut HTS_Vocoder,
    mut c1: *mut libc::c_double,
    m1: libc::c_int,
    a1: libc::c_double,
    g1: libc::c_double,
    mut c2: *mut libc::c_double,
    m2: libc::c_int,
    a2: libc::c_double,
    g2: libc::c_double,
) {
    let mut a: libc::c_double = 0.;
    if a1 == a2 {
        HTS_gnorm(c1, c1, m1, g1);
        HTS_gc2gc(v, c1, m1, g1, c2, m2, g2);
        HTS_ignorm(c2, c2, m2, g2);
    } else {
        a = (a2 - a1) / (1 as libc::c_int as libc::c_double - a1 * a2);
        HTS_freqt(v, c1, m1, c2, m2, a);
        HTS_gnorm(c2, c2, m2, g1);
        HTS_gc2gc(v, c2, m2, g1, c2, m2, g2);
        HTS_ignorm(c2, c2, m2, g2);
    };
}
unsafe fn HTS_lsp2mgc(
    v: &mut HTS_Vocoder,
    mut lsp: *mut libc::c_double,
    mut mgc: *mut libc::c_double,
    m: libc::c_int,
    alpha: libc::c_double,
) {
    let mut i: libc::c_int = 0;
    HTS_lsp2lpc(v, lsp.offset(1 as libc::c_int as isize), mgc, m);
    if v.use_log_gain != 0 {
        *mgc.offset(0 as libc::c_int as isize) = exp(*lsp.offset(0 as libc::c_int as isize));
    } else {
        *mgc.offset(0 as libc::c_int as isize) = *lsp.offset(0 as libc::c_int as isize);
    }
    HTS_ignorm(mgc, mgc, m, v.gamma);
    i = m;
    while i >= 1 as libc::c_int {
        *mgc.offset(i as isize) *= -(v.stage as libc::c_double);
        i -= 1;
    }
    HTS_mgc2mgc(v, mgc, m, alpha, v.gamma, mgc, m, alpha, v.gamma);
}
unsafe fn HTS_mglsadff(
    mut x: libc::c_double,
    mut b: *const libc::c_double,
    m: libc::c_int,
    a: libc::c_double,
    mut d: *mut libc::c_double,
) -> libc::c_double {
    let mut i: libc::c_int = 0;
    let mut y: libc::c_double = 0.;
    y = *d.offset(0 as libc::c_int as isize) * *b.offset(1 as libc::c_int as isize);
    i = 1 as libc::c_int;
    while i < m {
        *d.offset(i as isize) += a
            * (*d.offset((i + 1 as libc::c_int) as isize)
                - *d.offset((i - 1 as libc::c_int) as isize));
        y += *d.offset(i as isize) * *b.offset((i + 1 as libc::c_int) as isize);
        i += 1;
    }
    x -= y;
    i = m;
    while i > 0 as libc::c_int {
        *d.offset(i as isize) = *d.offset((i - 1 as libc::c_int) as isize);
        i -= 1;
    }
    *d.offset(0 as libc::c_int as isize) =
        a * *d.offset(0 as libc::c_int as isize) + (1 as libc::c_int as libc::c_double - a * a) * x;
    x
}
unsafe fn HTS_mglsadf(
    mut x: libc::c_double,
    mut b: *const libc::c_double,
    m: libc::c_int,
    a: libc::c_double,
    n: libc::c_int,
    mut d: *mut libc::c_double,
) -> libc::c_double {
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < n {
        x = HTS_mglsadff(
            x,
            b,
            m,
            a,
            &mut *d.offset((i * (m + 1 as libc::c_int)) as isize),
        );
        i += 1;
    }
    x
}
unsafe fn HTS_check_lsp_stability(mut lsp: *mut libc::c_double, mut m: size_t) {
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    let mut tmp: libc::c_double = 0.;
    let mut min: libc::c_double = 0.25f64 * 3.141_592_653_589_793_f64
        / m.wrapping_add(1 as libc::c_int as size_t) as libc::c_double;
    let mut find: HTS_Boolean = 0;
    i = 0 as libc::c_int as size_t;
    while i < 4 as libc::c_int as size_t {
        find = 0 as libc::c_int as HTS_Boolean;
        j = 1 as libc::c_int as size_t;
        while j < m {
            tmp = *lsp.offset(j.wrapping_add(1 as libc::c_int as size_t) as isize)
                - *lsp.offset(j as isize);
            if tmp < min {
                *lsp.offset(j as isize) -= 0.5f64 * (min - tmp);
                *lsp.offset(j.wrapping_add(1 as libc::c_int as size_t) as isize) +=
                    0.5f64 * (min - tmp);
                find = 1 as libc::c_int as HTS_Boolean;
            }
            j = j.wrapping_add(1);
        }
        if *lsp.offset(1 as libc::c_int as isize) < min {
            *lsp.offset(1 as libc::c_int as isize) = min;
            find = 1 as libc::c_int as HTS_Boolean;
        }
        if *lsp.offset(m as isize) > 3.141_592_653_589_793_f64 - min {
            *lsp.offset(m as isize) = 3.141_592_653_589_793_f64 - min;
            find = 1 as libc::c_int as HTS_Boolean;
        }
        if find as libc::c_int == 0 as libc::c_int {
            break;
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn HTS_lsp2en(
    v: &mut HTS_Vocoder,
    mut lsp: *mut libc::c_double,
    mut m: size_t,
    mut alpha: libc::c_double,
) -> libc::c_double {
    let mut i: size_t = 0;
    let mut en: libc::c_double = 0.0f64;
    let mut buff: *mut libc::c_double = std::ptr::null_mut::<libc::c_double>();
    if v.spectrum2en_size < m {
        if !(v.spectrum2en_buff).is_null() {
            HTS_free(v.spectrum2en_buff as *mut libc::c_void);
        }
        v.spectrum2en_buff = HTS_calloc(
            m.wrapping_add(1 as libc::c_int as size_t)
                .wrapping_add(576 as libc::c_int as size_t),
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.spectrum2en_size = m;
    }
    buff = (v.spectrum2en_buff)
        .offset(m as isize)
        .offset(1 as libc::c_int as isize);
    HTS_lsp2lpc(
        v,
        lsp.offset(1 as libc::c_int as isize),
        v.spectrum2en_buff,
        m as libc::c_int,
    );
    if v.use_log_gain != 0 {
        *(v.spectrum2en_buff).offset(0 as libc::c_int as isize) =
            exp(*lsp.offset(0 as libc::c_int as isize));
    } else {
        *(v.spectrum2en_buff).offset(0 as libc::c_int as isize) =
            *lsp.offset(0 as libc::c_int as isize);
    }
    HTS_ignorm(
        v.spectrum2en_buff,
        v.spectrum2en_buff,
        m as libc::c_int,
        v.gamma,
    );
    i = 1 as libc::c_int as size_t;
    while i <= m {
        *(v.spectrum2en_buff).offset(i as isize) *= -(v.stage as libc::c_double);
        i = i.wrapping_add(1);
    }
    HTS_mgc2mgc(
        v,
        v.spectrum2en_buff,
        m as libc::c_int,
        alpha,
        v.gamma,
        buff,
        576 as libc::c_int - 1 as libc::c_int,
        0.0f64,
        1 as libc::c_int as libc::c_double,
    );
    i = 0 as libc::c_int as size_t;
    while i < 576 as libc::c_int as size_t {
        en += *buff.offset(i as isize) * *buff.offset(i as isize);
        i = i.wrapping_add(1);
    }
    en
}
unsafe fn HTS_white_noise(v: &mut HTS_Vocoder) -> libc::c_double {
    if v.gauss != 0 {
        HTS_nrandom(v)
    } else {
        HTS_mseq(v) as libc::c_double
    }
}
unsafe fn HTS_Vocoder_initialize_excitation(
    v: &mut HTS_Vocoder,
    mut pitch: libc::c_double,
    mut nlpf: size_t,
) {
    let mut i: size_t = 0;
    v.pitch_of_curr_point = pitch;
    v.pitch_counter = pitch;
    v.pitch_inc_per_point = 0.0f64;
    if nlpf > 0 as libc::c_int as size_t {
        v.excite_buff_size = nlpf;
        v.excite_ring_buff = HTS_calloc(
            v.excite_buff_size,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        i = 0 as libc::c_int as size_t;
        while i < v.excite_buff_size {
            *(v.excite_ring_buff).offset(i as isize) = 0.0f64;
            i = i.wrapping_add(1);
        }
        v.excite_buff_index = 0 as libc::c_int as size_t;
    } else {
        v.excite_buff_size = 0 as libc::c_int as size_t;
        v.excite_ring_buff = std::ptr::null_mut::<libc::c_double>();
        v.excite_buff_index = 0 as libc::c_int as size_t;
    };
}
unsafe fn HTS_Vocoder_start_excitation(v: &mut HTS_Vocoder, mut pitch: libc::c_double) {
    if v.pitch_of_curr_point != 0.0f64 && pitch != 0.0f64 {
        v.pitch_inc_per_point = (pitch - v.pitch_of_curr_point) / v.fprd as libc::c_double;
    } else {
        v.pitch_inc_per_point = 0.0f64;
        v.pitch_of_curr_point = pitch;
        v.pitch_counter = pitch;
    };
}
unsafe fn HTS_Vocoder_excite_unvoiced_frame(v: &mut HTS_Vocoder, mut noise: libc::c_double) {
    let mut center: size_t = (v.excite_buff_size).wrapping_sub(1 as libc::c_int as size_t)
        / 2 as libc::c_int as size_t;
    *(v.excite_ring_buff).offset(
        ((v.excite_buff_index).wrapping_add(center) % v.excite_buff_size) as isize,
    ) += noise;
}
unsafe fn HTS_Vocoder_excite_voiced_frame(
    v: &mut HTS_Vocoder,
    mut noise: libc::c_double,
    mut pulse: libc::c_double,
    mut lpf: *const libc::c_double,
) {
    let mut i: size_t = 0;
    let mut center: size_t = (v.excite_buff_size).wrapping_sub(1 as libc::c_int as size_t)
        / 2 as libc::c_int as size_t;
    if noise != 0.0f64 {
        i = 0 as libc::c_int as size_t;
        while i < v.excite_buff_size {
            if i == center {
                *(v.excite_ring_buff).offset(
                    ((v.excite_buff_index).wrapping_add(i) % v.excite_buff_size) as isize,
                ) += noise * (1.0f64 - *lpf.offset(i as isize));
            } else {
                *(v.excite_ring_buff).offset(
                    ((v.excite_buff_index).wrapping_add(i) % v.excite_buff_size) as isize,
                ) += noise * (0.0f64 - *lpf.offset(i as isize));
            }
            i = i.wrapping_add(1);
        }
    }
    if pulse != 0.0f64 {
        i = 0 as libc::c_int as size_t;
        while i < v.excite_buff_size {
            *(v.excite_ring_buff).offset(
                ((v.excite_buff_index).wrapping_add(i) % v.excite_buff_size) as isize,
            ) += pulse * *lpf.offset(i as isize);
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn HTS_Vocoder_get_excitation(
    v: &mut HTS_Vocoder,
    mut lpf: *const libc::c_double,
) -> libc::c_double {
    let mut x: libc::c_double = 0.;
    let mut noise: libc::c_double = 0.;
    let mut pulse: libc::c_double = 0.0f64;
    if v.excite_buff_size > 0 as libc::c_int as size_t {
        noise = HTS_white_noise(v);
        pulse = 0.0f64;
        if v.pitch_of_curr_point == 0.0f64 {
            HTS_Vocoder_excite_unvoiced_frame(v, noise);
        } else {
            v.pitch_counter += 1.0f64;
            if v.pitch_counter >= v.pitch_of_curr_point {
                pulse = sqrt(v.pitch_of_curr_point);
                v.pitch_counter -= v.pitch_of_curr_point;
            }
            HTS_Vocoder_excite_voiced_frame(v, noise, pulse, lpf);
            v.pitch_of_curr_point += v.pitch_inc_per_point;
        }
        x = *(v.excite_ring_buff).offset(v.excite_buff_index as isize);
        *(v.excite_ring_buff).offset(v.excite_buff_index as isize) = 0.0f64;
        v.excite_buff_index = (v.excite_buff_index).wrapping_add(1);
        v.excite_buff_index;
        if v.excite_buff_index >= v.excite_buff_size {
            v.excite_buff_index = 0 as libc::c_int as size_t;
        }
    } else if v.pitch_of_curr_point == 0.0f64 {
        x = HTS_white_noise(v);
    } else {
        v.pitch_counter += 1.0f64;
        if v.pitch_counter >= v.pitch_of_curr_point {
            x = sqrt(v.pitch_of_curr_point);
            v.pitch_counter -= v.pitch_of_curr_point;
        } else {
            x = 0.0f64;
        }
        v.pitch_of_curr_point += v.pitch_inc_per_point;
    }
    x
}
unsafe fn HTS_Vocoder_end_excitation(v: &mut HTS_Vocoder, mut pitch: libc::c_double) {
    v.pitch_of_curr_point = pitch;
}
unsafe fn HTS_Vocoder_postfilter_mcp(
    v: &mut HTS_Vocoder,
    mut mcp: *mut libc::c_double,
    m: libc::c_int,
    mut alpha: libc::c_double,
    mut beta: libc::c_double,
) {
    let mut e1: libc::c_double = 0.;
    let mut e2: libc::c_double = 0.;
    let mut k: libc::c_int = 0;
    if beta > 0.0f64 && m > 1 as libc::c_int {
        if v.postfilter_size < m as size_t {
            if !(v.postfilter_buff).is_null() {
                HTS_free(v.postfilter_buff as *mut libc::c_void);
            }
            v.postfilter_buff = HTS_calloc(
                (m + 1 as libc::c_int) as size_t,
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            v.postfilter_size = m as size_t;
        }
        HTS_mc2b(mcp, v.postfilter_buff, m, alpha);
        e1 = HTS_b2en(v, v.postfilter_buff, m, alpha);
        *(v.postfilter_buff).offset(1 as libc::c_int as isize) -=
            beta * alpha * *(v.postfilter_buff).offset(2 as libc::c_int as isize);
        k = 2 as libc::c_int;
        while k <= m {
            *(v.postfilter_buff).offset(k as isize) *= 1.0f64 + beta;
            k += 1;
        }
        e2 = HTS_b2en(v, v.postfilter_buff, m, alpha);
        *(v.postfilter_buff).offset(0 as libc::c_int as isize) +=
            log(e1 / e2) / 2 as libc::c_int as libc::c_double;
        HTS_b2mc(v.postfilter_buff, mcp, m, alpha);
    }
}
unsafe fn HTS_Vocoder_postfilter_lsp(
    v: &mut HTS_Vocoder,
    mut lsp: *mut libc::c_double,
    mut m: size_t,
    mut alpha: libc::c_double,
    mut beta: libc::c_double,
) {
    let mut e1: libc::c_double = 0.;
    let mut e2: libc::c_double = 0.;
    let mut i: size_t = 0;
    let mut d1: libc::c_double = 0.;
    let mut d2: libc::c_double = 0.;
    if beta > 0.0f64 && m > 1 as libc::c_int as size_t {
        if v.postfilter_size < m {
            if !(v.postfilter_buff).is_null() {
                HTS_free(v.postfilter_buff as *mut libc::c_void);
            }
            v.postfilter_buff = HTS_calloc(
                m.wrapping_add(1 as libc::c_int as size_t),
                ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
            ) as *mut libc::c_double;
            v.postfilter_size = m;
        }
        e1 = HTS_lsp2en(v, lsp, m, alpha);
        i = 0 as libc::c_int as size_t;
        while i <= m {
            if i > 1 as libc::c_int as size_t && i < m {
                d1 = beta
                    * (*lsp.offset(i.wrapping_add(1 as libc::c_int as size_t) as isize)
                        - *lsp.offset(i as isize));
                d2 = beta
                    * (*lsp.offset(i as isize)
                        - *lsp.offset(i.wrapping_sub(1 as libc::c_int as size_t) as isize));
                *(v.postfilter_buff).offset(i as isize) = *lsp
                    .offset(i.wrapping_sub(1 as libc::c_int as size_t) as isize)
                    + d2
                    + d2 * d2
                        * (*lsp.offset(i.wrapping_add(1 as libc::c_int as size_t) as isize)
                            - *lsp.offset(i.wrapping_sub(1 as libc::c_int as size_t) as isize)
                            - (d1 + d2))
                        / (d2 * d2 + d1 * d1);
            } else {
                *(v.postfilter_buff).offset(i as isize) = *lsp.offset(i as isize);
            }
            i = i.wrapping_add(1);
        }
        HTS_movem(
            v.postfilter_buff,
            lsp,
            m.wrapping_add(1 as libc::c_int as size_t) as libc::c_int,
        );
        e2 = HTS_lsp2en(v, lsp, m, alpha);
        if e1 != e2 {
            if v.use_log_gain != 0 {
                *lsp.offset(0 as libc::c_int as isize) += 0.5f64 * log(e1 / e2);
            } else {
                *lsp.offset(0 as libc::c_int as isize) *= sqrt(e1 / e2);
            }
        }
    }
}

pub unsafe fn HTS_Vocoder_initialize(
    v: &mut HTS_Vocoder,
    mut m: size_t,
    mut stage: size_t,
    mut use_log_gain: HTS_Boolean,
    mut rate: size_t,
    mut fperiod: size_t,
) {
    v.is_first = 1 as libc::c_int as HTS_Boolean;
    v.stage = stage;
    if stage != 0 as libc::c_int as size_t {
        v.gamma = -1.0f64 / v.stage as libc::c_double;
    } else {
        v.gamma = 0.0f64;
    }
    v.use_log_gain = use_log_gain;
    v.fprd = fperiod;
    v.next = 1 as libc::c_int as libc::c_ulong;
    v.gauss = 1 as libc::c_int as HTS_Boolean;
    v.rate = rate as libc::c_double;
    v.pitch_of_curr_point = 0.0f64;
    v.pitch_counter = 0.0f64;
    v.pitch_inc_per_point = 0.0f64;
    v.excite_ring_buff = std::ptr::null_mut::<libc::c_double>();
    v.excite_buff_size = 0 as libc::c_int as size_t;
    v.excite_buff_index = 0 as libc::c_int as size_t;
    v.sw = 0 as libc::c_int as libc::c_uchar;
    v.x = 0x55555555 as libc::c_int;
    v.freqt_buff = std::ptr::null_mut::<libc::c_double>();
    v.freqt_size = 0 as libc::c_int as size_t;
    v.gc2gc_buff = std::ptr::null_mut::<libc::c_double>();
    v.gc2gc_size = 0 as libc::c_int as size_t;
    v.lsp2lpc_buff = std::ptr::null_mut::<libc::c_double>();
    v.lsp2lpc_size = 0 as libc::c_int as size_t;
    v.postfilter_buff = std::ptr::null_mut::<libc::c_double>();
    v.postfilter_size = 0 as libc::c_int as size_t;
    v.spectrum2en_buff = std::ptr::null_mut::<libc::c_double>();
    v.spectrum2en_size = 0 as libc::c_int as size_t;
    if v.stage == 0 as libc::c_int as size_t {
        v.c = HTS_calloc(
            (m * (3 as libc::c_int + 5 as libc::c_int) as size_t)
                .wrapping_add((5 as libc::c_int * 5 as libc::c_int) as size_t)
                .wrapping_add(6 as libc::c_int as size_t),
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.cc = (v.c)
            .offset(m as isize)
            .offset(1 as libc::c_int as isize);
        v.cinc = (v.cc)
            .offset(m as isize)
            .offset(1 as libc::c_int as isize);
        v.d1 = (v.cinc)
            .offset(m as isize)
            .offset(1 as libc::c_int as isize);
    } else {
        v.c = HTS_calloc(
            m.wrapping_add(1 as libc::c_int as size_t)
                * (v.stage).wrapping_add(3 as libc::c_int as size_t),
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        ) as *mut libc::c_double;
        v.cc = (v.c)
            .offset(m as isize)
            .offset(1 as libc::c_int as isize);
        v.cinc = (v.cc)
            .offset(m as isize)
            .offset(1 as libc::c_int as isize);
        v.d1 = (v.cinc)
            .offset(m as isize)
            .offset(1 as libc::c_int as isize);
    };
}

pub unsafe fn HTS_Vocoder_synthesize(
    v: &mut HTS_Vocoder,
    mut m: size_t,
    mut lf0: libc::c_double,
    mut spectrum: *mut libc::c_double,
    mut nlpf: size_t,
    mut lpf: *mut libc::c_double,
    mut alpha: libc::c_double,
    mut beta: libc::c_double,
    mut volume: libc::c_double,
    mut rawdata: *mut libc::c_double,
) {
    let mut x: libc::c_double = 0.;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut _xs: libc::c_short = 0;
    let mut rawidx: libc::c_int = 0 as libc::c_int;
    let mut p: libc::c_double = 0.;
    if lf0 == -1.0e+10f64 {
        p = 0.0f64;
    } else if lf0 <= MIN_LF0 {
        p = v.rate / MIN_F0;
    } else if lf0 >= MAX_LF0 {
        p = v.rate / MAX_F0;
    } else {
        p = v.rate / exp(lf0);
    }
    if v.is_first as libc::c_int == 1 as libc::c_int {
        HTS_Vocoder_initialize_excitation(v, p, nlpf);
        if v.stage == 0 as libc::c_int as size_t {
            HTS_mc2b(spectrum, v.c, m as libc::c_int, alpha);
        } else {
            HTS_movem(
                spectrum,
                v.c,
                m.wrapping_add(1 as libc::c_int as size_t) as libc::c_int,
            );
            HTS_lsp2mgc(v, v.c, v.c, m as libc::c_int, alpha);
            HTS_mc2b(v.c, v.c, m as libc::c_int, alpha);
            HTS_gnorm(v.c, v.c, m as libc::c_int, v.gamma);
            i = 1 as libc::c_int;
            while i as size_t <= m {
                *(v.c).offset(i as isize) *= v.gamma;
                i += 1;
            }
        }
        v.is_first = 0 as libc::c_int as HTS_Boolean;
    }
    HTS_Vocoder_start_excitation(v, p);
    if v.stage == 0 as libc::c_int as size_t {
        HTS_Vocoder_postfilter_mcp(v, spectrum, m as libc::c_int, alpha, beta);
        HTS_mc2b(spectrum, v.cc, m as libc::c_int, alpha);
        i = 0 as libc::c_int;
        while i as size_t <= m {
            *(v.cinc).offset(i as isize) = (*(v.cc).offset(i as isize)
                - *(v.c).offset(i as isize))
                / v.fprd as libc::c_double;
            i += 1;
        }
    } else {
        HTS_Vocoder_postfilter_lsp(v, spectrum, m, alpha, beta);
        HTS_check_lsp_stability(spectrum, m);
        HTS_lsp2mgc(v, spectrum, v.cc, m as libc::c_int, alpha);
        HTS_mc2b(v.cc, v.cc, m as libc::c_int, alpha);
        HTS_gnorm(v.cc, v.cc, m as libc::c_int, v.gamma);
        i = 1 as libc::c_int;
        while i as size_t <= m {
            *(v.cc).offset(i as isize) *= v.gamma;
            i += 1;
        }
        i = 0 as libc::c_int;
        while i as size_t <= m {
            *(v.cinc).offset(i as isize) = (*(v.cc).offset(i as isize)
                - *(v.c).offset(i as isize))
                / v.fprd as libc::c_double;
            i += 1;
        }
    }
    j = 0 as libc::c_int;
    while (j as size_t) < v.fprd {
        x = HTS_Vocoder_get_excitation(v, lpf);
        if v.stage == 0 as libc::c_int as size_t {
            if x != 0.0f64 {
                x *= exp(*(v.c).offset(0 as libc::c_int as isize));
            }
            x = HTS_mlsadf(
                x,
                v.c,
                m as libc::c_int,
                alpha,
                5 as libc::c_int,
                v.d1,
            );
        } else {
            if 0 as libc::c_int == 0 {
                x *= *(v.c).offset(0 as libc::c_int as isize);
            }
            x = HTS_mglsadf(
                x,
                v.c,
                m as libc::c_int,
                alpha,
                v.stage as libc::c_int,
                v.d1,
            );
        }
        x *= volume;
        if !rawdata.is_null() {
            let fresh8 = rawidx;
            rawidx += 1;
            *rawdata.offset(fresh8 as isize) = x;
        }
        i = 0 as libc::c_int;
        while i as size_t <= m {
            *(v.c).offset(i as isize) += *(v.cinc).offset(i as isize);
            i += 1;
        }
        j += 1;
    }
    HTS_Vocoder_end_excitation(v, p);
    HTS_movem(
        v.cc,
        v.c,
        m.wrapping_add(1 as libc::c_int as size_t) as libc::c_int,
    );
}

pub unsafe fn HTS_Vocoder_clear(v: *mut HTS_Vocoder) {
    if !v.is_null() {
        if !((*v).freqt_buff).is_null() {
            HTS_free((*v).freqt_buff as *mut libc::c_void);
            (*v).freqt_buff = std::ptr::null_mut::<libc::c_double>();
        }
        (*v).freqt_size = 0 as libc::c_int as size_t;
        if !((*v).gc2gc_buff).is_null() {
            HTS_free((*v).gc2gc_buff as *mut libc::c_void);
            (*v).gc2gc_buff = std::ptr::null_mut::<libc::c_double>();
        }
        (*v).gc2gc_size = 0 as libc::c_int as size_t;
        if !((*v).lsp2lpc_buff).is_null() {
            HTS_free((*v).lsp2lpc_buff as *mut libc::c_void);
            (*v).lsp2lpc_buff = std::ptr::null_mut::<libc::c_double>();
        }
        (*v).lsp2lpc_size = 0 as libc::c_int as size_t;
        if !((*v).postfilter_buff).is_null() {
            HTS_free((*v).postfilter_buff as *mut libc::c_void);
            (*v).postfilter_buff = std::ptr::null_mut::<libc::c_double>();
        }
        (*v).postfilter_size = 0 as libc::c_int as size_t;
        if !((*v).spectrum2en_buff).is_null() {
            HTS_free((*v).spectrum2en_buff as *mut libc::c_void);
            (*v).spectrum2en_buff = std::ptr::null_mut::<libc::c_double>();
        }
        (*v).spectrum2en_size = 0 as libc::c_int as size_t;
        if !((*v).c).is_null() {
            HTS_free((*v).c as *mut libc::c_void);
            (*v).c = std::ptr::null_mut::<libc::c_double>();
        }
        (*v).excite_buff_size = 0 as libc::c_int as size_t;
        (*v).excite_buff_index = 0 as libc::c_int as size_t;
        if !((*v).excite_ring_buff).is_null() {
            HTS_free((*v).excite_ring_buff as *mut libc::c_void);
            (*v).excite_ring_buff = std::ptr::null_mut::<libc::c_double>();
        }
    }
}
