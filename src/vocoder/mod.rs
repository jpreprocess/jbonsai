use std::{
    f64::consts::PI,
    ops::{Index, IndexMut, RangeFrom, RangeFull},
    slice::SliceIndex,
};

use crate::constants::{MAX_F0, MAX_LF0, MIN_F0, MIN_LF0};

use self::excitation::Excitation;

mod excitation;

const PADE: [f64; 21] = [
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
    stage: Stage,

    use_log_gain: bool,
    fperiod: usize,
    rate: usize,
    /// is_first := excitation.is_none()
    excitation: Option<Excitation>,

    d1: Vec<f64>,
}

impl Vocoder {
    pub fn new(m: usize, stage: usize, use_log_gain: bool, rate: usize, fperiod: usize) -> Self {
        let stage = Stage::new(stage);
        let d1_len = stage.d1_len(m + 1);

        Self {
            stage,

            use_log_gain,
            fperiod,
            rate,
            excitation: None,

            d1: vec![0.0; d1_len],
        }
    }

    /// rawdata.len() >= self.fperiod
    pub fn synthesize(
        &mut self,
        lf0: f64,
        spectrum: &[f64],
        nlpf: usize,
        lpf: &[f64],
        alpha: f64,
        beta: f64,
        volume: f64,
        rawdata: &mut [f64],
    ) {
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
            match self.stage {
                Stage::Zero {
                    ref mut coefficients,
                } => {
                    let cepstrum = MelCepstrum::new(spectrum, alpha);
                    *coefficients = cepstrum.mc2b();
                }
                Stage::NonZero {
                    stage,
                    gamma,
                    ref mut coefficients,
                } => {
                    let lsp =
                        LineSpectralPairs::new(spectrum, alpha, self.use_log_gain, stage, gamma);
                    *coefficients = lsp.lsp2mgc().mc2b().gnorm();
                    for i in 1..coefficients.len() {
                        coefficients[i] *= gamma;
                    }
                }
            }
        }

        match self.stage {
            Stage::Zero {
                ref mut coefficients,
            } => {
                let mut cepstrum = MelCepstrum::new(spectrum, alpha);
                cepstrum.postfilter_mcp(beta);
                let cc = cepstrum.mc2b();
                let cinc: Vec<_> = cc
                    .iter()
                    .zip(&*coefficients)
                    .map(|(cc, c)| (cc - c) / self.fperiod as f64)
                    .collect();

                let excitation = self
                    .excitation
                    .get_or_insert_with(|| Excitation::new(p, nlpf));
                excitation.start(p, self.fperiod);

                for j in 0..self.fperiod {
                    let mut x = excitation.get(lpf);
                    if x != 0.0 {
                        x *= coefficients[0].exp();
                    }
                    let mlsa = MelLogSpectrumApproximation::new(&coefficients[..], alpha, 5);
                    mlsa.df(&mut x, &mut self.d1);
                    x *= volume;
                    rawdata[j] = x;
                    for i in 0..coefficients.len() {
                        coefficients[i] += cinc[i];
                    }
                }

                excitation.end(p);
                *coefficients = cc
            }
            Stage::NonZero {
                stage,
                gamma,
                ref mut coefficients,
            } => {
                let mut lsp =
                    LineSpectralPairs::new(spectrum, alpha, self.use_log_gain, stage, gamma);
                lsp.postfilter_lsp(beta);
                lsp.check_lsp_stability();
                let mut cc = lsp.lsp2mgc().mc2b().gnorm();
                for i in 1..cc.len() {
                    cc[i] *= gamma;
                }
                let cinc: Vec<_> = cc
                    .iter()
                    .zip(&*coefficients)
                    .map(|(cc, c)| (cc - c) / self.fperiod as f64)
                    .collect();

                let excitation = self
                    .excitation
                    .get_or_insert_with(|| Excitation::new(p, nlpf));
                excitation.start(p, self.fperiod);

                for j in 0..self.fperiod {
                    let mut x = excitation.get(lpf);
                    x *= coefficients[0];
                    let mglsa = MelGeneralizedLogSpectrumApproximation::new(
                        &coefficients[..],
                        alpha,
                        stage,
                    );
                    mglsa.df(&mut x, &mut self.d1);
                    x *= volume;
                    rawdata[j] = x;
                    for i in 0..coefficients.len() {
                        coefficients[i] += cinc[i];
                    }
                }

                excitation.end(p);
                *coefficients = cc
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Stage {
    NonZero {
        stage: usize,
        gamma: f64,
        coefficients: GeneralizedCoefficients,
    },
    Zero {
        coefficients: Coefficients,
    },
}

impl Stage {
    fn new(stage: usize) -> Self {
        if stage == 0 {
            Self::Zero {
                coefficients: Coefficients { buffer: Vec::new() },
            }
        } else {
            let gamma = -1.0 / stage as f64;
            Self::NonZero {
                stage,
                gamma,
                coefficients: GeneralizedCoefficients {
                    buffer: Vec::new(),
                    gamma,
                },
            }
        }
    }

    fn d1_len(&self, c_len: usize) -> usize {
        match self {
            Self::NonZero { stage, .. } => c_len * stage,
            Self::Zero { .. } => (c_len + 4) * 5 + 3,
        }
    }
}

macro_rules! buffer_index {
    ($t:ty) => {
        impl<I: SliceIndex<[f64]>> Index<I> for $t {
            type Output = I::Output;

            fn index(&self, index: I) -> &Self::Output {
                &self.buffer[index]
            }
        }

        impl<I: SliceIndex<[f64]>> IndexMut<I> for $t {
            fn index_mut(&mut self, index: I) -> &mut Self::Output {
                &mut self.buffer[index]
            }
        }

        impl<'a> IntoIterator for &'a $t {
            type Item = &'a f64;
            type IntoIter = <&'a Vec<f64> as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.buffer.iter()
            }
        }

        impl Buffer for $t {
            fn len(&self) -> usize {
                self.buffer.len()
            }

            #[allow(dead_code)]
            fn iter(&self) -> <&Vec<f64> as IntoIterator>::IntoIter {
                self.buffer.iter()
            }
        }
    };
}

trait Buffer:
    Index<usize, Output = f64>
    + IndexMut<usize, Output = f64>
    + Index<RangeFull, Output = [f64]>
    + Index<RangeFrom<usize>, Output = [f64]>
    + IndexMut<RangeFrom<usize>, Output = [f64]>
{
    fn len(&self) -> usize;
    fn iter(&self) -> <&Vec<f64> as IntoIterator>::IntoIter;
}

/// Line Spectral Pairs
#[derive(Debug, Clone)]
struct LineSpectralPairs {
    buffer: Vec<f64>,
    alpha: f64,
    use_log_gain: bool,
    stage: usize,
    gamma: f64,
}

buffer_index!(LineSpectralPairs);

impl LineSpectralPairs {
    fn new(lsp: &[f64], alpha: f64, use_log_gain: bool, stage: usize, gamma: f64) -> Self {
        Self {
            buffer: lsp.to_vec(),
            alpha,
            use_log_gain,
            stage,
            gamma,
        }
    }

    /// convert self to Linear Prediction Coding
    /// lpc.len() == lsp.len() + 1
    fn lsp2lpc(&self) -> MelGeneralizedCepstrum {
        let m = self.len();
        let (mh1, mh2) = if m % 2 == 0 {
            (m / 2, m / 2)
        } else {
            ((m + 1) / 2, (m - 1) / 2)
        };

        let p: Vec<_> = self.iter().step_by(2).map(|x| -2.0 * x.cos()).collect();
        let q: Vec<_> = self
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

        let mut cepstrum = MelGeneralizedCepstrum {
            buffer: vec![0.0; m + 1],
            alpha: self.alpha,
            gamma: self.gamma,
        };
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
                cepstrum[k - 1] = -0.5 * (a0[mh1] + b0[mh2]);
            }
        }

        for i in (0..m).rev() {
            cepstrum[i + 1] = -cepstrum[i];
        }
        cepstrum[0] = 1.0;

        cepstrum
    }

    // mgc.len() == lsp.len()
    fn lsp2mgc(&self) -> MelGeneralizedCepstrum {
        let mut lpc = self.lsp2lpc();
        if self.use_log_gain {
            lpc[0] = self[0].exp();
        } else {
            lpc[0] = self[0];
        }
        let mut lpc = lpc.ignorm();
        for i in 1..lpc.len() {
            lpc[i] *= -(self.stage as f64);
        }
        lpc.mgc2mgc(self.len() - 1, self.alpha, self.gamma)
    }

    /// calculate frame energy
    fn lsp2en(&self) -> f64 {
        self.lsp2mgc().iter().map(|x| x * x).sum()
    }

    fn postfilter_lsp(&mut self, beta: f64) {
        if beta > 0.0 && self.len() > 2 {
            let mut buf = vec![0.0; self.len()];
            let en1 = self.lsp2en();
            for i in 0..self.len() {
                if i > 1 && i < self.len() - 1 {
                    let d1 = beta * (self[i + 1] - self[i]);
                    let d2 = beta * (self[i] - self[i - 1]);
                    buf[i] = self[i - 1]
                        + d2
                        + (d2 * d2 * ((self[i + 1] - self[i - 1]) - (d1 + d2)))
                            / ((d2 * d2) + (d1 * d1));
                } else {
                    buf[i] = self[i];
                }
            }
            self[..].copy_from_slice(&buf);

            let en2 = self.lsp2en();
            if en1 != en2 {
                if self.use_log_gain {
                    self[0] += 0.5 * (en1 / en2).ln();
                } else {
                    self[0] *= (en1 / en2).sqrt();
                }
            }
        }
    }

    fn check_lsp_stability(&mut self) {
        let min = 0.25 * PI / self.len() as f64;
        let last = self.len() - 1;
        for _ in 0..4 {
            let mut find = false;
            for j in 1..last {
                let tmp = self[j + 1] - self[j];
                if tmp < min {
                    self[j] -= 0.5 * (min - tmp);
                    self[j + 1] += 0.5 * (min - tmp);
                    find = true;
                }
            }
            if self[1] < min {
                self[1] = min;
                find = true;
            }
            if self[last] > PI - min {
                self[last] = PI - min;
                find = true;
            }
            if !find {
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct MelCepstrum {
    buffer: Vec<f64>,
    alpha: f64,
}

buffer_index!(MelCepstrum);

impl MelCepstrum {
    fn new(c: &[f64], alpha: f64) -> Self {
        Self {
            buffer: c.to_vec(),
            alpha,
        }
    }

    fn postfilter_mcp(&mut self, beta: f64) {
        if beta > 0.0 && self.len() > 2 {
            let mut coefficients = self.mc2b();
            let e1 = coefficients.b2en(self.alpha);

            coefficients[1] -= beta * self.alpha * coefficients[2];
            for k in 2..self.len() {
                coefficients[k] *= 1.0 + beta;
            }

            let e2 = coefficients.b2en(self.alpha);
            coefficients[0] += (e1 / e2).ln() / 2.0;
            *self = coefficients.b2mc(self.alpha);
        }
    }
}

impl CepstrumT for MelCepstrum {
    type Coef = Coefficients;

    fn alpha(&self) -> f64 {
        self.alpha
    }

    fn to_coef(&self) -> Self::Coef {
        Self::Coef {
            buffer: self.buffer.clone(),
        }
    }

    fn clone_with_size(&self, size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            alpha: self.alpha,
        }
    }
}

#[derive(Debug, Clone)]
struct MelGeneralizedCepstrum {
    buffer: Vec<f64>,
    alpha: f64,
    gamma: f64,
}

buffer_index!(MelGeneralizedCepstrum);

impl MelGeneralizedCepstrum {
    fn gc2gc(&self, m2: usize, gamma: f64) -> Self {
        let mut cepstrum = Self {
            buffer: vec![0.0; m2 + 1],
            alpha: self.alpha,
            gamma,
        };
        cepstrum[0] = self[0];

        for i in 1..=m2 {
            let mut ss1 = 0.0;
            let mut ss2 = 0.0;
            for k in 1..self.len().min(i) {
                let mk = i - k;
                let cc = self[k] * cepstrum[mk];
                ss1 += mk as f64 * cc;
                ss2 += k as f64 * cc;
            }
            if i < self.len() {
                cepstrum[i] = self[i] + (cepstrum.gamma * ss2 - self.gamma * ss1) / (i as f64);
            } else {
                cepstrum[i] = (cepstrum.gamma * ss2 - self.gamma * ss1) / (i as f64);
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
}

impl CepstrumT for MelGeneralizedCepstrum {
    type Coef = GeneralizedCoefficients;

    fn alpha(&self) -> f64 {
        self.alpha
    }

    fn to_coef(&self) -> Self::Coef {
        Self::Coef {
            buffer: self.buffer.clone(),
            gamma: self.gamma,
        }
    }

    fn clone_with_size(&self, size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            alpha: self.alpha,
            gamma: self.gamma,
        }
    }
}

impl Generalized for MelGeneralizedCepstrum {
    fn gamma(&self) -> f64 {
        self.gamma
    }
}

trait CepstrumT: Buffer + Sized {
    type Coef: CoefficientsT;

    fn alpha(&self) -> f64;

    fn to_coef(&self) -> Self::Coef;

    fn mc2b(&self) -> Self::Coef {
        let mut coefficients = self.to_coef();
        if self.alpha() != 0.0 {
            let last = self.len() - 1;
            coefficients[last] = self[last];
            for i in (0..last).rev() {
                coefficients[i] = self[i] - self.alpha() * coefficients[i + 1];
            }
        }
        coefficients
    }

    fn clone_with_size(&self, size: usize) -> Self;

    fn freqt(&self, m2: usize, alpha: f64) -> Self {
        let aa = 1.0 - alpha * alpha;

        let mut cepstrum = self.clone_with_size(m2 + 1);
        let mut f = vec![0.0; cepstrum.len()];

        for i in 0..=self.len() {
            f[0] = cepstrum[0];
            cepstrum[0] = self[i] + alpha * cepstrum[0];
            if 1 <= m2 {
                f[1] = cepstrum[1];
                cepstrum[1] = aa * f[0] + alpha * cepstrum[1];
            }
            for j in 2..cepstrum.len() {
                f[j] = cepstrum[j];
                cepstrum[j] = f[j - 1] + alpha * (cepstrum[j] - cepstrum[j - 1]);
            }
        }

        cepstrum
    }

    fn c2ir(&self, len: usize) -> Vec<f64> {
        let mut ir = vec![0.0; len];
        ir[0] = self[0].exp();
        for n in 1..len {
            let mut d = 0.0;
            for k in 1..self.len().min(n + 1) {
                d += k as f64 * self[k] * ir[n - k];
            }
            ir[n] = d / n as f64;
        }
        ir
    }
}

#[derive(Debug, Clone)]
struct Coefficients {
    buffer: Vec<f64>,
}

buffer_index!(Coefficients);

impl CoefficientsT for Coefficients {
    type Cep = MelCepstrum;

    fn to_cep(&self, alpha: f64) -> Self::Cep {
        Self::Cep {
            buffer: vec![0.0; self.len()],
            alpha,
        }
    }
}

#[derive(Debug, Clone)]
struct GeneralizedCoefficients {
    buffer: Vec<f64>,
    gamma: f64,
}

buffer_index!(GeneralizedCoefficients);

impl CoefficientsT for GeneralizedCoefficients {
    type Cep = MelGeneralizedCepstrum;

    fn to_cep(&self, alpha: f64) -> Self::Cep {
        Self::Cep {
            buffer: vec![0.0; self.len()],
            alpha,
            gamma: self.gamma,
        }
    }
}

trait CoefficientsT: Buffer {
    type Cep: CepstrumT;

    fn to_cep(&self, alpha: f64) -> Self::Cep;

    fn b2mc(&self, alpha: f64) -> Self::Cep {
        let mut cepstrum = self.to_cep(alpha);
        let last = self.len() - 1;
        cepstrum[last] = self[last];
        for i in (0..last).rev() {
            cepstrum[i] = self[i] + alpha * self[i + 1];
        }
        cepstrum
    }

    fn b2en(&self, alpha: f64) -> f64 {
        let ir = self.b2mc(alpha).freqt(576 - 1, -alpha).c2ir(576);
        ir.iter().map(|x| x * x).sum()
    }
}

impl Generalized for GeneralizedCoefficients {
    fn gamma(&self) -> f64 {
        self.gamma
    }
}

trait Generalized: Clone + Buffer {
    fn gamma(&self) -> f64;

    fn gnorm(&self) -> Self {
        let mut target = self.clone();

        if self.gamma() != 0.0 {
            let k = 1.0 + self.gamma() * self[0];
            target[0] = k.powf(1.0 / self.gamma());
            for i in 1..self.len() {
                target[i] = self[i] / k;
            }
        } else {
            target[0] = self[0].exp();
            target[1..].copy_from_slice(&self[1..]);
        };

        target
    }

    fn ignorm(&self) -> Self {
        let mut target = self.clone();

        if self.gamma() != 0.0 {
            let k = self[0].powf(self.gamma());
            target[0] = (k - 1.0) / self.gamma();
            for i in 1..self.len() {
                target[i] = self[i] * k;
            }
        } else {
            target[0] = self[0].ln();
            target[1..].copy_from_slice(&self[1..]);
        };

        target
    }
}

#[derive(Debug)]
struct MelLogSpectrumApproximation<'a> {
    b: &'a [f64],
    alpha: f64,
    pd: usize,
    aa: f64,
    ppade: &'a [f64],
}

impl<'a> MelLogSpectrumApproximation<'a> {
    fn new(b: &'a [f64], alpha: f64, pd: usize) -> Self {
        Self {
            b,
            alpha,
            pd,
            aa: 1.0 - alpha * alpha,
            ppade: &PADE[(pd * (pd + 1) / 2)..],
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
struct MelGeneralizedLogSpectrumApproximation<'a> {
    b: &'a [f64],
    alpha: f64,
    n: usize,
    aa: f64,
}

impl<'a> MelGeneralizedLogSpectrumApproximation<'a> {
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
