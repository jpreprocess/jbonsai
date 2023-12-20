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
                let cepstrum = Cepstrum::new(spectrum, alpha, self.gamma);
                self.c = cepstrum.mc2b();
            } else {
                let lsp = Lsp::new(spectrum, alpha, self);
                let b = lsp.lsp2mgc().mc2b();
                self.c = gnorm(&b, self.gamma);
                for i in 1..self.c.len() {
                    self.c[i] *= self.gamma;
                }
            }
        }

        let cc = if self.stage == 0 {
            let mut cepstrum = Cepstrum::new(spectrum, alpha, self.gamma);
            cepstrum.postfilter_mcp(beta);
            cepstrum.mc2b()
        } else {
            let mut lsp = Lsp::new(spectrum, alpha, self);
            lsp.postfilter_lsp(beta);
            lsp.check_lsp_stability();
            let b = lsp.lsp2mgc().mc2b();
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

        let excitation = self
            .excitation
            .get_or_insert_with(|| Excitation::new(p, nlpf));
        excitation.start(p, self.fperiod);
        for j in 0..self.fperiod {
            let mut x = excitation.get(lpf);
            if self.stage == 0 {
                if x != 0.0 {
                    x *= self.c[0].exp();
                }
                let mlsa = Mlsa::new(&self.c, alpha, 5);
                mlsa.df(&mut x, &mut self.d1)
            } else {
                x *= self.c[0];
                let mglsa = Mglsa::new(&self.c, alpha, self.stage);
                mglsa.df(&mut x, &mut self.d1)
            }
            x *= volume;

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

/// Line Spectral Pairs
#[derive(Debug, Clone)]
struct Lsp {
    lsp: Vec<f64>,
    alpha: f64,
    use_log_gain: bool,
    stage: usize,
    gamma: f64,
}

impl Lsp {
    fn new(lsp: &[f64], alpha: f64, vocoder: &Vocoder) -> Self {
        Self {
            lsp: lsp.to_vec(),
            alpha,
            use_log_gain: vocoder.use_log_gain,
            stage: vocoder.stage,
            gamma: vocoder.gamma,
        }
    }

    /// convert self to Linear Prediction Coding
    /// lpc.len() == lsp.len() + 1
    fn lsp2lpc(&self) -> Vec<f64> {
        let m = self.lsp.len();
        let (mh1, mh2) = if m % 2 == 0 {
            (m / 2, m / 2)
        } else {
            ((m + 1) / 2, (m - 1) / 2)
        };

        let p: Vec<_> = self.lsp.iter().step_by(2).map(|x| -2.0 * x.cos()).collect();
        let q: Vec<_> = self
            .lsp
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

    /// calculate frame energy
    fn lsp2en(&self) -> f64 {
        let mut lpc = self.lsp2lpc();
        if self.use_log_gain {
            lpc[0] = self.lsp[0].exp();
        } else {
            lpc[0] = self.lsp[0];
        }
        let mut cepstrum = Cepstrum::new(&lpc, self.alpha, self.gamma).ignorm();
        for i in 1..self.lsp.len() {
            cepstrum.c[i] *= -(self.stage as f64);
        }

        let cepstrum = cepstrum.mgc2mgc(576 - 1, 0.0, 1.0);
        cepstrum.c.iter().map(|x| x * x).sum()
    }

    /// mgc.len() == lsp.len()
    fn lsp2mgc(&self) -> Cepstrum {
        let mut a = self.lsp2lpc();
        if self.use_log_gain {
            a[0] = self.lsp[0].exp();
        } else {
            a[0] = self.lsp[0];
        }
        let mut cepstrum = Cepstrum::new(&a, self.alpha, self.gamma).ignorm();
        for i in 1..cepstrum.c.len() {
            cepstrum.c[i] *= -(self.stage as f64);
        }
        cepstrum.mgc2mgc(self.lsp.len() - 1, self.alpha, self.gamma)
    }

    fn postfilter_lsp(&mut self, beta: f64) {
        if beta > 0.0 && self.lsp.len() > 2 {
            let mut buf = vec![0.0; self.lsp.len()];
            let en1 = self.lsp2en();
            for i in 0..self.lsp.len() {
                if i > 1 && i < self.lsp.len() - 1 {
                    let d1 = beta * (self.lsp[i + 1] - self.lsp[i]);
                    let d2 = beta * (self.lsp[i] - self.lsp[i - 1]);
                    buf[i] = self.lsp[i - 1]
                        + d2
                        + (d2 * d2 * ((self.lsp[i + 1] - self.lsp[i - 1]) - (d1 + d2)))
                            / ((d2 * d2) + (d1 * d1));
                } else {
                    buf[i] = self.lsp[i];
                }
            }
            self.lsp.copy_from_slice(&buf);

            let en2 = self.lsp2en();
            if en1 != en2 {
                if self.use_log_gain {
                    self.lsp[0] += 0.5 * (en1 / en2).ln();
                } else {
                    self.lsp[0] *= (en1 / en2).sqrt();
                }
            }
        }
    }

    fn check_lsp_stability(&mut self) {
        let min = 0.25 * PI / self.lsp.len() as f64;
        let last = self.lsp.len() - 1;
        for _ in 0..4 {
            let mut find = false;
            for j in 1..last {
                let tmp = self.lsp[j + 1] - self.lsp[j];
                if tmp < min {
                    self.lsp[j] -= 0.5 * (min - tmp);
                    self.lsp[j + 1] += 0.5 * (min - tmp);
                    find = true;
                }
            }
            if self.lsp[1] < min {
                self.lsp[1] = min;
                find = true;
            }
            if self.lsp[last] > PI - min {
                self.lsp[last] = PI - min;
                find = true;
            }
            if !find {
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Cepstrum {
    c: Vec<f64>,
    alpha: f64,
    gamma: f64,
}

impl Cepstrum {
    fn new(c: &[f64], alpha: f64, gamma: f64) -> Self {
        Self {
            c: c.to_vec(),
            alpha,
            gamma,
        }
    }

    fn mc2b(&self) -> Vec<f64> {
        if self.alpha != 0.0 {
            let mut b = vec![0.0; self.c.len()];
            let last = self.c.len() - 1;
            b[last] = self.c[last];
            for i in (0..last).rev() {
                b[i] = self.c[i] - self.alpha * b[i + 1];
            }
            b
        } else {
            self.c.to_vec()
        }
    }

    fn gnorm(&self) -> Self {
        let c = if self.gamma != 0.0 {
            let k = 1.0 + self.gamma * self.c[0];
            iter::once(k.powf(1.0 / self.gamma))
                .chain(self.c[1..].iter().map(|x| x / k))
                .collect()
        } else {
            iter::once(self.c[0].exp())
                .chain(self.c[1..].iter().cloned())
                .collect()
        };
        Self {
            c,
            alpha: self.alpha,
            gamma: self.gamma,
        }
    }

    fn ignorm(&self) -> Self {
        let c = if self.gamma != 0.0 {
            let k = self.c[0].powf(self.gamma);
            iter::once((k - 1.0) / self.gamma)
                .chain(self.c[1..].iter().map(|x| x * k))
                .collect()
        } else {
            iter::once(self.c[0].ln())
                .chain(self.c[1..].iter().cloned())
                .collect()
        };
        Self {
            c,
            alpha: self.alpha,
            gamma: self.gamma,
        }
    }

    fn postfilter_mcp(&mut self, beta: f64) {
        if beta > 0.0 && self.c.len() > 2 {
            let mut b = self.mc2b();
            let e1 = b2en(&b, self.alpha);

            b[1] -= beta * self.alpha * b[2];
            for k in 2..self.c.len() {
                b[k] *= 1.0 + beta;
            }

            let e2 = b2en(&b, self.alpha);
            b[0] += (e1 / e2).ln() / 2.0;
            let mc = b2mc(&b, self.alpha);
            self.c.copy_from_slice(&mc);
        }
    }

    fn freqt(&self, m2: usize, alpha: f64) -> Self {
        let aa = 1.0 - alpha * alpha;

        let mut cepstrum = Self {
            c: vec![0.0; m2 + 1],
            alpha: self.alpha,
            gamma: self.gamma,
        };
        let mut f = vec![0.0; cepstrum.c.len()];

        for i in 0..=self.c.len() {
            f[0] = cepstrum.c[0];
            cepstrum.c[0] = self.c[i] + alpha * cepstrum.c[0];
            if 1 <= m2 {
                f[1] = cepstrum.c[1];
                cepstrum.c[1] = aa * f[0] + alpha * cepstrum.c[1];
            }
            for j in 2..cepstrum.c.len() {
                f[j] = cepstrum.c[j];
                cepstrum.c[j] = f[j - 1] + alpha * (cepstrum.c[j] - cepstrum.c[j - 1]);
            }
        }

        cepstrum
    }

    fn gc2gc(&self, m2: usize, gamma: f64) -> Self {
        let mut cepstrum = Self {
            c: vec![0.0; m2 + 1],
            alpha: self.alpha,
            gamma,
        };
        cepstrum.c[0] = self.c[0];

        for i in 1..=m2 {
            let mut ss1 = 0.0;
            let mut ss2 = 0.0;
            for k in 1..self.c.len().min(i) {
                let mk = i - k;
                let cc = self.c[k] * cepstrum.c[mk];
                ss1 += mk as f64 * cc;
                ss2 += k as f64 * cc;
            }
            if i < self.c.len() {
                cepstrum.c[i] = self.c[i] + (cepstrum.gamma * ss2 - self.gamma * ss1) / (i as f64);
            } else {
                cepstrum.c[i] = (cepstrum.gamma * ss2 - self.gamma * ss1) / (i as f64);
            }
        }

        cepstrum
    }

    fn mgc2mgc(&self, m2: usize, alpha: f64, gamma: f64) -> Self {
        if self.alpha == alpha {
            self.gnorm().gc2gc(m2, gamma).ignorm()
        } else {
            let alpha = (alpha - self.alpha) / (1.0 - self.alpha * alpha);
            self.freqt(m2, alpha).gnorm().gc2gc(m2, gamma).ignorm()
        }
    }

    fn c2ir(&self, len: usize) -> Vec<f64> {
        let mut ir = vec![0.0; len];
        ir[0] = self.c[0].exp();
        for n in 1..len {
            let mut d = 0.0;
            for k in 1..self.c.len().min(n + 1) {
                d += k as f64 * self.c[k] * ir[n - k];
            }
            ir[n] = d / n as f64;
        }
        ir
    }
}

impl Gain for Cepstrum {
    fn gamma(&self) -> f64 {
        self.gamma
    }

    fn slice(&self) -> &[f64] {
        &self.c
    }

    fn slice_mut(&mut self) -> &mut [f64] {
        &mut self.c
    }
}

fn b2en(b: &[f64], alpha: f64) -> f64 {
    let c = b2mc(b, alpha);
    let c = Cepstrum {
        c,
        alpha,
        gamma: 0.0,
    };
    let ir = c.freqt(576 - 1, -alpha).c2ir(576);
    ir.iter().map(|x| x * x).sum()
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

trait Gain: Clone {
    fn gamma(&self) -> f64;
    fn slice(&self) -> &[f64];
    fn slice_mut(&mut self) -> &mut [f64];

    fn gnorm(&self) -> Self {
        let mut cloned = self.clone();
        let source = self.slice();
        let target = cloned.slice_mut();

        if self.gamma() != 0.0 {
            let k = 1.0 + self.gamma() * source[0];
            target[0] = k.powf(1.0 / self.gamma());
            for i in 1..source.len() {
                target[i] = source[i] / k;
            }
        } else {
            target[0] = source[0].exp();
            for i in 1..source.len() {
                target[i] = source[i];
            }
        };

        cloned
    }

    fn ignorm(&self) -> Self {
        let mut cloned = self.clone();
        let source = self.slice();
        let target = cloned.slice_mut();

        if self.gamma() != 0.0 {
            let k = source[0].powf(self.gamma());
            target[0] = (k - 1.0) / self.gamma();
            for i in 1..source.len() {
                target[i] = source[i] * k;
            }
        } else {
            target[0] = source[0].ln();
            for i in 1..source.len() {
                target[i] = source[i];
            }
        };

        cloned
    }
}

// temporary
fn gnorm(b: &[f64], gamma: f64) -> Vec<f64> {
    #[derive(Debug, Clone)]
    struct B {
        b: Vec<f64>,
        gamma: f64,
    }

    impl Gain for B {
        fn gamma(&self) -> f64 {
            self.gamma
        }

        fn slice(&self) -> &[f64] {
            &self.b
        }

        fn slice_mut(&mut self) -> &mut [f64] {
            &mut self.b
        }
    }

    B {
        b: b.to_vec(),
        gamma,
    }
    .gnorm()
    .b
}

#[derive(Debug)]
struct Mlsa<'a> {
    b: &'a [f64],
    alpha: f64,
    pd: usize,
    aa: f64,
    ppade: &'a [f64],
}

impl<'a> Mlsa<'a> {
    fn new(b: &'a [f64], alpha: f64, pd: usize) -> Self {
        Self {
            b,
            alpha,
            pd,
            aa: 1.0 - alpha * alpha,
            ppade: &HTS_PADE[(pd * (pd + 1) / 2)..],
        }
    }

    /// d.len() == pd * b.len() + 4 * pd + 3
    fn df(&self, x: &mut f64, d: &mut [f64]) {
        let (d1, d2) = d.split_at_mut(2 * (self.pd + 1));
        self.df1(x, d1);
        self.df2(x, d2);
    }

    /// d.len() == 2 * self.pd + 2
    fn df1(&self, x: &mut f64, d: &mut [f64]) {
        let mut out = 0.0;
        let (d, pt) = d.split_at_mut(self.pd + 1);
        for i in (1..=self.pd).rev() {
            d[i] = self.aa * pt[i - 1] + self.alpha * d[i];
            pt[i] = d[i] * self.b[1];
            let v = pt[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        pt[0] = *x;
        *x += out;
    }

    /// d.len() == self.pd * self.b.len() + 2 * self.pd + 1
    fn df2(&self, x: &mut f64, d: &mut [f64]) {
        let mut out = 0.0;
        let (d, pt) = d.split_at_mut(self.pd * (self.b.len() + 1));
        for i in (1..=self.pd).rev() {
            pt[i] = self.fir(
                pt[i - 1],
                &mut d[(i - 1) * (self.b.len() + 1)..i * (self.b.len() + 1)],
            );
            let v = pt[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        pt[0] = *x;
        *x += out;
    }

    fn fir(&self, x: f64, d: &mut [f64]) -> f64 {
        d[0] = x;
        d[1] = self.aa * d[0] + self.alpha * d[1];
        for i in 2..self.b.len() {
            d[i] += self.alpha * (d[i + 1] - d[i - 1]);
        }
        let mut y = 0.0;
        for i in 2..self.b.len() {
            y += d[i] * self.b[i];
        }
        for i in (2..d.len()).rev() {
            d[i] = d[i - 1];
        }
        y
    }
}

#[derive(Debug)]
struct Mglsa<'a> {
    b: &'a [f64],
    alpha: f64,
    n: usize,
    aa: f64,
}

impl<'a> Mglsa<'a> {
    fn new(b: &'a [f64], alpha: f64, n: usize) -> Self {
        Self {
            b,
            alpha,
            n,
            aa: 1.0 - alpha * alpha,
        }
    }

    fn df(&self, x: &mut f64, d: &mut [f64]) {
        for i in 0..self.n {
            self.dff(x, &mut d[i * self.b.len()..(i + 1) * self.b.len()]);
        }
    }

    fn dff(&self, x: &mut f64, d: &mut [f64]) {
        let mut y = d[0] * self.b[1];
        for i in 1..self.b.len() - 1 {
            d[i] += self.alpha * (d[i + 1] - d[i - 1]);
            y += d[i] * self.b[i + 1];
        }
        *x -= y;
        for i in (1..self.b.len()).rev() {
            d[i] = d[i - 1];
        }
        d[0] = self.alpha * d[0] + self.aa * *x;
    }
}
