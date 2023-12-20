use std::{f64::consts::PI, iter};

use crate::util::{MAX_F0, MAX_LF0, MIN_F0, MIN_LF0};

const HTS_PADE: [f64; 21] = [
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

#[derive(Debug, Clone)]
pub struct Vocoder {
    stage: usize,
    gamma: f64,
    use_log_gain: bool,
    fperiod: usize,
    rate: usize,
    /// is_first := excitation.is_none()
    excitation: Option<Excitation>,

    c: Vec<f64>,
    d1: Vec<f64>,
}

impl Vocoder {
    fn d1_len(stage: usize, c_len: usize) -> usize {
        if stage == 0 {
            (c_len + 4) * 5 + 3
        } else {
            c_len * stage
        }
    }

    pub fn new(m: usize, stage: usize, use_log_gain: bool, rate: usize, fperiod: usize) -> Self {
        let gamma = if stage != 0 { -1.0 / stage as f64 } else { 0.0 };
        let c_len = m + 1;
        let d1_len = Self::d1_len(stage, c_len);

        Self {
            stage,
            gamma,
            use_log_gain,
            fperiod,
            rate,
            excitation: None,

            c: vec![0.0; c_len],
            d1: vec![0.0; d1_len],
        }
    }

    /// spectrum.len() == _m + 1
    /// rawdata.len() == self.fperiod
    pub fn synthesize(
        &mut self,
        _m: usize,
        lf0: f64,
        spectrum: &mut [f64],
        nlpf: usize,
        lpf: &[f64],
        alpha: f64,
        beta: f64,
        volume: f64,
        rawdata: &mut [f64],
    ) {
        debug_assert_eq!(self.c.len(), spectrum.len());
        debug_assert_eq!(self.d1.len(), Self::d1_len(self.stage, spectrum.len()));

        let p = if lf0 == -1.0e+10 {
            0.0
        } else if lf0 <= MIN_LF0 {
            self.rate as f64 / MIN_F0
        } else if lf0 >= MAX_LF0 {
            self.rate as f64 / MAX_F0
        } else {
            self.rate as f64 / lf0.exp()
        };
        if self.excitation.is_none() {
            if self.stage == 0 {
                self.c = mc2b(spectrum, alpha);
            } else {
                let mgc = lsp2mgc(spectrum, alpha, self.use_log_gain, self.stage, self.gamma);
                let b = mc2b(&mgc, alpha);
                self.c = gnorm(&b, self.gamma);
                for i in 1..self.c.len() {
                    self.c[i] *= self.gamma;
                }
            }
        }
        let excitation = self
            .excitation
            .get_or_insert_with(|| Excitation::new(p, nlpf));
        excitation.start(p, self.fperiod);

        let cc = if self.stage == 0 {
            postfilter_mcp(spectrum, alpha, beta);
            mc2b(spectrum, alpha)
        } else {
            postfilter_lsp(
                spectrum,
                alpha,
                beta,
                self.use_log_gain,
                self.stage,
                self.gamma,
            );
            check_lsp_stability(spectrum);
            let mgc = lsp2mgc(spectrum, alpha, self.use_log_gain, self.stage, self.gamma);
            let b = mc2b(&mgc, alpha);
            let cc = gnorm(&b, self.gamma);
            iter::once(cc[0])
                .chain(cc[1..].iter().map(|x| x * self.gamma))
                .collect()
        };
        let cinc: Vec<_> = cc
            .iter()
            .zip(&self.c)
            .map(|(cc, c)| (cc - c) / self.fperiod as f64)
            .collect();
        for j in 0..self.fperiod {
            let mut x = excitation.get(lpf);
            if self.stage == 0 {
                if x != 0.0 {
                    x *= self.c[0].exp();
                }
                mlsadf(&mut x, &self.c, alpha, 5, &mut self.d1);
            } else {
                x *= self.c[0];
                mglsadf(&mut x, &self.c, alpha, self.stage, &mut self.d1);
            }
            x += volume;

            rawdata[j] = x;
            for i in 0..self.c.len() {
                self.c[i] += cinc[i];
            }
        }

        excitation.end(p);
        self.c.copy_from_slice(&cc);
    }
}

#[derive(Debug, Clone)]
pub struct Excitation {
    pitch_of_curr_point: f64,
    pitch_counter: f64,
    pitch_inc_per_point: f64,
    ring_buffer: RingBuffer<f64>,
    gauss: bool,
    mseq: Mseq,
    random: Random,
}

impl Excitation {
    pub fn new(pitch: f64, nlpf: usize) -> Self {
        Self {
            pitch_of_curr_point: pitch,
            pitch_counter: pitch,
            pitch_inc_per_point: 0.0,
            ring_buffer: RingBuffer::new(nlpf),
            gauss: true,
            mseq: Mseq::new(),
            random: Random::new(),
        }
    }

    fn start(&mut self, pitch: f64, fperiod: usize) {
        if self.pitch_of_curr_point != 0.0 && pitch != 0.0 {
            self.pitch_inc_per_point = (pitch - self.pitch_of_curr_point) / fperiod as f64;
        } else {
            self.pitch_inc_per_point = 0.0;
            self.pitch_of_curr_point = pitch;
            self.pitch_counter = pitch;
        }
    }

    fn white_noise(&mut self) -> f64 {
        if self.gauss {
            self.random.nrandom()
        } else {
            self.mseq.next() as f64
        }
    }

    fn unvoiced_frame(&mut self, noise: f64) {
        let center = (self.ring_buffer.len() - 1) / 2;
        *self.ring_buffer.get_mut_with_offset(center) += noise;
    }

    /// lpf.len() == nlpf
    fn voiced_frame(&mut self, noise: f64, pulse: f64, lpf: &[f64]) {
        let center = (self.ring_buffer.len() - 1) / 2;
        if noise != 0.0 {
            for i in 0..self.ring_buffer.len() {
                if i == center {
                    *self.ring_buffer.get_mut_with_offset(i) += noise * (1.0 - lpf[i]);
                } else {
                    *self.ring_buffer.get_mut_with_offset(i) += noise * (0.0 - lpf[i]);
                }
            }
        }
        if pulse != 0.0 {
            for i in 0..self.ring_buffer.len() {
                *self.ring_buffer.get_mut_with_offset(i) += pulse * lpf[i];
            }
        }
    }

    /// lpf.len() == nlpf
    fn get(&mut self, lpf: &[f64]) -> f64 {
        if self.ring_buffer.len() > 0 {
            let noise = self.white_noise();
            if self.pitch_of_curr_point == 0.0 {
                self.unvoiced_frame(noise);
            } else {
                self.pitch_counter += 1.0;
                let pulse = if self.pitch_counter >= self.pitch_of_curr_point {
                    self.pitch_counter -= self.pitch_of_curr_point;
                    self.pitch_of_curr_point.sqrt()
                } else {
                    0.0
                };
                self.voiced_frame(noise, pulse, lpf);
                self.pitch_of_curr_point += self.pitch_inc_per_point;
            }
            let x = *self.ring_buffer.get();
            *self.ring_buffer.get_mut() = 0.0;
            self.ring_buffer.advance();
            x
        } else if self.pitch_of_curr_point == 0.0 {
            self.white_noise()
        } else {
            self.pitch_counter += 1.0;
            let x = if self.pitch_counter >= self.pitch_of_curr_point {
                self.pitch_counter -= self.pitch_of_curr_point;
                self.pitch_of_curr_point.sqrt()
            } else {
                0.0
            };
            self.pitch_of_curr_point += self.pitch_inc_per_point;
            x
        }
    }

    fn end(&mut self, pitch: f64) {
        self.pitch_of_curr_point = pitch;
    }
}

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> RingBuffer<T> {
    fn new(size: usize) -> Self
    where
        T: Default + Clone,
    {
        Self {
            buffer: vec![Default::default(); size],
            index: 0,
        }
    }

    fn get(&self) -> &T {
        &self.buffer[self.index]
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.buffer[self.index]
    }

    fn get_mut_with_offset(&mut self, i: usize) -> &mut T {
        let index = (self.index + i) % self.buffer.len();
        &mut self.buffer[index]
    }

    fn advance(&mut self) {
        self.index += 1;
        if self.index >= self.buffer.len() {
            self.index = 0;
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Debug, Clone)]
pub struct Mseq {
    x: u32,
}

impl Mseq {
    pub fn new() -> Self {
        Self { x: 0x55555555 }
    }

    fn next(&mut self) -> i32 {
        self.x >>= 1;
        let x0 = if self.x & 0x00000001 != 0 { 1 } else { -1 };
        let x28 = if self.x & 0x10000000 != 0 { 1 } else { -1 };
        if x0 + x28 != 0 {
            self.x &= 0x7fffffff;
        } else {
            self.x |= 0x80000000;
        }
        x0
    }
}

#[derive(Debug, Clone)]
pub struct Random {
    sw: bool,
    r1: f64,
    r2: f64,
    s: f64,
    next: usize,
}

impl Random {
    pub fn new() -> Self {
        Self {
            sw: false,
            r1: 0.0,
            r2: 0.0,
            s: 0.0,
            next: 1,
        }
    }

    fn nrandom(&mut self) -> f64 {
        if self.sw {
            self.sw = false;
            self.r2 * self.s
        } else {
            self.sw = true;
            loop {
                self.r1 = 2.0 * self.rnd() - 1.0;
                self.r2 = 2.0 * self.rnd() - 1.0;
                self.s = self.r1 * self.r1 + self.r2 * self.r2;
                if !(self.s > 1.0 || self.s == 0.0) {
                    break;
                }
            }
            self.s = (-2.0 * self.s.ln() / self.s).sqrt();
            self.r1 * self.s
        }
    }

    fn rnd(&mut self) -> f64 {
        self.next = self.next.wrapping_mul(1103515245).wrapping_add(12345);
        let r = self.next.wrapping_div(65536).wrapping_rem(32768);
        r as f64 / 32767.0
    }
}

/// a went to return value
/// lpc.len() == lsp.len() + 1
fn lsp2lpc(lsp: &[f64]) -> Vec<f64> {
    let m = lsp.len();
    let (mh1, mh2) = if m % 2 == 0 {
        (m / 2, m / 2)
    } else {
        ((m + 1) / 2, (m - 1) / 2)
    };

    let p: Vec<_> = lsp.iter().step_by(2).map(|x| -2.0 * x.cos()).collect();
    let q: Vec<_> = lsp
        .iter()
        .skip(1)
        .step_by(2)
        .map(|x| -2.0 * x.cos())
        .collect();
    let mut a0 = vec![0.0; mh1 + 1];
    let mut a1 = vec![0.0; mh1 + 1];
    let mut a2 = vec![0.0; mh1 + 1];
    let mut b0 = vec![0.0; mh2 + 1];
    let mut b1 = vec![0.0; mh2 + 1];
    let mut b2 = vec![0.0; mh2 + 1];

    let mut xff = 0.0;
    let mut xf = 0.0;

    let mut lpc = vec![0.0; m + 1];
    for k in 0..=m {
        let xx = if k == 0 { 1.0 } else { 0.0 };
        if m % 2 == 1 {
            a0[0] = xx;
            b0[0] = xx - xff;
            xff = xf;
            xf = xx;
        } else {
            a0[0] = xx + xf;
            b0[0] = xx - xf;
            xf = xx;
        }
        for i in 0..mh1 {
            a0[i + 1] = a0[i] + p[i] * a1[i] + a2[i];
            a2[i] = a1[i];
            a1[i] = a0[i];
        }
        for i in 0..mh2 {
            b0[i + 1] = b0[i] + q[i] * b1[i] + b2[i];
            b2[i] = b1[i];
            b1[i] = b0[i];
        }
        if k > 0 {
            lpc[k - 1] = -0.5 * (a0[mh1] + b0[mh2]);
        }
    }

    for i in (0..m).rev() {
        lpc[i + 1] = -lpc[i];
    }
    lpc[0] = 1.0;

    lpc
}

fn lsp2en(lsp: &[f64], alpha: f64, use_log_gain: bool, stage: usize, gamma: f64) -> f64 {
    let mut lpc = lsp2lpc(&lsp[1..]);
    if use_log_gain {
        lpc[0] = lsp[0].exp();
    } else {
        lpc[0] = lsp[0];
    }
    let mut c2 = ignorm(&lpc, gamma);
    for i in 1..lsp.len() {
        c2[i] *= -(stage as f64);
    }

    let c2 = mgc2mgc(&c2, alpha, gamma, 576 - 1, 0.0, 1.0);
    c2.iter().map(|x| x * x).sum()
}

/// mgc went to return value
/// mgc.len() == lsp.len()
fn lsp2mgc(lsp: &[f64], alpha: f64, use_log_gain: bool, stage: usize, gamma: f64) -> Vec<f64> {
    let mut a = lsp2lpc(&lsp[1..]);
    if use_log_gain {
        a[0] = lsp[0].exp();
    } else {
        a[0] = lsp[0];
    }
    let mut c2 = ignorm(&a, gamma);
    for c2 in &mut c2[1..] {
        *c2 *= -(stage as f64);
    }
    mgc2mgc(&c2, alpha, gamma, lsp.len() - 1, alpha, gamma)
}

fn postfilter_lsp(
    lsp: &mut [f64],
    alpha: f64,
    beta: f64,
    use_log_gain: bool,
    stage: usize,
    gamma: f64,
) {
    if beta > 0.0 && lsp.len() > 2 {
        let mut buf = vec![0.0; lsp.len()];
        let en1 = lsp2en(&lsp, alpha, use_log_gain, stage, gamma);
        for i in 0..lsp.len() {
            if i > 1 && i < lsp.len() - 1 {
                let d1 = beta * (lsp[i + 1] - lsp[i]);
                let d2 = beta * (lsp[i] - lsp[i - 1]);
                buf[i] = lsp[i - 1]
                    + d2
                    + (d2 * d2 * ((lsp[i + 1] - lsp[i - 1]) - (d1 + d2))) / ((d2 * d2) + (d1 * d1));
            } else {
                buf[i] = lsp[i];
            }
        }
        lsp.copy_from_slice(&buf);

        let en2 = lsp2en(&lsp, alpha, use_log_gain, stage, gamma);
        if en1 != en2 {
            if use_log_gain {
                lsp[0] += 0.5 * (en1 / en2).ln();
            } else {
                lsp[0] *= (en1 / en2).sqrt();
            }
        }
    }
}

fn check_lsp_stability(lsp: &mut [f64]) {
    let min = 0.25 * PI / lsp.len() as f64;
    let last = lsp.len() - 1;
    for _ in 0..4 {
        let mut find = false;
        for j in 1..last {
            let tmp = lsp[j + 1] - lsp[j];
            if tmp < min {
                lsp[j] -= 0.5 * (min - tmp);
                lsp[j + 1] += 0.5 * (min - tmp);
                find = true;
            }
        }
        if lsp[1] < min {
            lsp[1] = min;
            find = true;
        }
        if lsp[last] > PI - min {
            lsp[last] = PI - min;
            find = true;
        }
        if !find {
            break;
        }
    }
}

/// c2 went to return value
/// c2.len() == m2 + 1
fn freqt(mc: &[f64], m2: isize, alpha: f64) -> Vec<f64> {
    assert!(m2 + 1 >= 0);
    let len = (m2 + 1) as usize;
    let mut f = vec![0.0; len];
    let mut c = vec![0.0; len];
    let b = 1.0 - alpha * alpha;

    for i in 0..=mc.len() {
        if 0 <= m2 {
            f[0] = c[0];
            c[0] = mc[i] + alpha * c[0];
        }
        if 1 <= m2 {
            f[1] = c[1];
            c[1] = b * f[0] + alpha * c[1];
        }
        for j in 2..len {
            f[j] = c[j];
            c[j] = f[j - 1] + alpha * (c[j] - c[j - 1]);
        }
    }

    c
}

/// c2 went to return value
/// c2.len() == m2 + 1
fn gc2gc(gc1: &[f64], gamma1: f64, m2: usize, gamma2: f64) -> Vec<f64> {
    let mut gc2 = vec![0.0; m2 + 1];
    gc2[0] = gc1[0];

    for i in 1..=m2 {
        let mut ss1 = 0.0;
        let mut ss2 = 0.0;
        for k in 1..gc1.len().min(i) {
            let mk = i - k;
            let cc = gc1[k] * gc2[mk];
            ss1 += mk as f64 * cc;
            ss2 += k as f64 * cc;
        }
        if i < gc1.len() {
            gc2[i] = gc1[i] + (gamma2 * ss2 - gamma1 * ss1) / (i as f64);
        } else {
            gc2[i] = (gamma2 * ss2 - gamma1 * ss1) / (i as f64);
        }
    }

    gc2
}

fn postfilter_mcp(mcp: &mut [f64], alpha: f64, beta: f64) {
    if beta > 0.0 && mcp.len() > 2 {
        let mut b = mc2b(&mcp, alpha);
        let e1 = b2en(&b, alpha);

        b[1] -= beta * alpha * b[2];
        for k in 2..mcp.len() {
            b[k] *= 1.0 + beta;
        }

        let e2 = b2en(&b, alpha);
        b[0] += (e1 / e2).ln() / 2.0;
        let mc = b2mc(&b, alpha);
        mcp.copy_from_slice(&mc);
    }
}

/// b went to return value
/// b.len() == mc.len()
fn mc2b(mc: &[f64], alpha: f64) -> Vec<f64> {
    if alpha != 0.0 {
        let mut b = vec![0.0; mc.len()];
        let last = mc.len() - 1;
        b[last] = mc[last];
        for i in (0..last).rev() {
            b[i] = mc[i] - alpha * b[i + 1];
        }
        b
    } else {
        mc.to_vec()
    }
}

fn b2en(b: &[f64], alpha: f64) -> f64 {
    let mc = b2mc(b, alpha);
    let c = freqt(&mc, 576 - 1, -alpha);
    let ir = c2ir(&c, 576);

    ir.iter().map(|x| x * x).sum()
}

/// c2 went to return value
/// c1.len() == _m1 + 1
/// c2.len() == m2 + 1
fn mgc2mgc(mgc: &[f64], alpha1: f64, gamma1: f64, m2: usize, alpha2: f64, gamma2: f64) -> Vec<f64> {
    if alpha1 == alpha2 {
        let gc1 = gnorm(mgc, gamma1);
        let gc2 = gc2gc(&gc1, gamma1, m2, gamma2);
        ignorm(&gc2, gamma2)
    } else {
        let alpha = (alpha2 - alpha1) / (1.0 - alpha1 * alpha2);
        let c1 = freqt(mgc, m2 as isize, alpha);
        let gc1 = gnorm(&c1, gamma1);
        let gc2 = gc2gc(&gc1, gamma1, m2, gamma2);
        ignorm(&gc2, gamma2)
    }
}

/// mc went to return value
/// mc.len() == b.len()
fn b2mc(b: &[f64], alpha: f64) -> Vec<f64> {
    let mut mc = vec![0.0; b.len()];
    let last = b.len() - 1;
    mc[last] = b[last];
    for i in (0..last).rev() {
        mc[i] = b[i] + alpha * b[i + 1];
    }
    mc
}

/// h went to return value
/// h.len() == leng
fn c2ir(c: &[f64], leng: usize) -> Vec<f64> {
    let mut ir = vec![0.0; leng];
    ir[0] = c[0].exp();
    for n in 1..leng {
        let mut d = 0.0;
        for k in 1..c.len().min(n + 1) {
            d += k as f64 * c[k] * ir[n - k];
        }
        ir[n] = d / n as f64;
    }
    ir
}

/// c2 went to return value
/// c2.len() == c1.len()
fn ignorm(c1: &[f64], gamma: f64) -> Vec<f64> {
    if gamma != 0.0 {
        let k = c1[0].powf(gamma);
        iter::once((k - 1.0) / gamma)
            .chain(c1[1..].iter().map(|x| x * k))
            .collect()
    } else {
        iter::once(c1[0].ln())
            .chain(c1[1..].iter().cloned())
            .collect()
    }
}

/// c2 went to return value
/// c2.len() == c1.len()
fn gnorm(c1: &[f64], gamma: f64) -> Vec<f64> {
    if gamma != 0.0 {
        let k = 1.0 + gamma * c1[0];
        iter::once(k.powf(1.0 / gamma))
            .chain(c1[1..].iter().map(|x| x / k))
            .collect()
    } else {
        iter::once(c1[0].exp())
            .chain(c1[1..].iter().cloned())
            .collect()
    }
}

/// d.len() == pd * b.len() + 4 * pd + 3
fn mlsadf(x: &mut f64, b: &[f64], alpha: f64, pd: usize, d: &mut [f64]) {
    let aa = 1.0 - alpha * alpha;
    let ppade = &HTS_PADE[(pd * (pd + 1) / 2)..];
    let (d1, d2) = d.split_at_mut(2 * (pd + 1));
    mlsadf1(x, b, alpha, aa, pd, d1, ppade);
    mlsadf2(x, b, alpha, aa, pd, d2, ppade);
}

/// d.len() == 2 * pd + 2
fn mlsadf1(x: &mut f64, b: &[f64], alpha: f64, aa: f64, pd: usize, d: &mut [f64], ppade: &[f64]) {
    let mut out = 0.0;
    let (d, pt) = d.split_at_mut(pd + 1);
    for i in (1..=pd).rev() {
        d[i] = aa * pt[i - 1] + alpha * d[i];
        pt[i] = d[i] * b[1];
        let v = pt[i] * ppade[i];
        *x += if i & 1 != 0 { v } else { -v };
        out += v;
    }
    pt[0] = *x;
    *x += out;
}

/// d.len() == pd * b.len() + 2 * pd + 1
fn mlsadf2(x: &mut f64, b: &[f64], alpha: f64, aa: f64, pd: usize, d: &mut [f64], ppade: &[f64]) {
    let mut out = 0.0;
    let (d, pt) = d.split_at_mut(pd * (b.len() + 1));
    for i in (1..=pd).rev() {
        pt[i] = mlsafir(
            pt[i - 1],
            b,
            alpha,
            aa,
            &mut d[(i - 1) * (b.len() + 1)..i * (b.len() + 1)],
        );
        let v = pt[i] * ppade[i];
        *x += if i & 1 != 0 { v } else { -v };
        out += v;
    }
    pt[0] = *x;
    *x += out;
}

/// d.len() == b.len() + 1
fn mlsafir(x: f64, b: &[f64], alpha: f64, aa: f64, d: &mut [f64]) -> f64 {
    d[0] = x;
    d[1] = aa * d[0] + alpha * d[1];
    for i in 2..b.len() {
        d[i] += alpha * (d[i + 1] - d[i - 1]);
    }
    let mut y = 0.0;
    for i in 2..b.len() {
        y += d[i] * b[i];
    }
    for i in (1..d.len()).rev() {
        d[i] = d[i - 1];
    }
    y
}

/// d.len() == n * b.len()
fn mglsadf(x: &mut f64, b: &[f64], alpha: f64, n: usize, d: &mut [f64]) {
    for i in 0..n {
        mglsadff(x, b, alpha, &mut d[i * b.len()..(i + 1) * b.len()]);
    }
}

/// d.len() == b.len()
fn mglsadff(x: &mut f64, b: &[f64], alpha: f64, d: &mut [f64]) {
    let mut y = d[0] * b[1];
    for i in 1..b.len() - 1 {
        d[i] += alpha * (d[i + 1] - d[i - 1]);
        y += d[i] * b[i + 1];
    }
    *x -= y;
    for i in (1..b.len()).rev() {
        d[i] = d[i - 1];
    }
    d[0] = alpha * d[0] + (1.0 - alpha * alpha) * *x;
}
