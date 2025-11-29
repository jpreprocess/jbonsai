use super::{
    buffer::*,
    coefficients::{Coefficients, CoefficientsT, GeneralizedCoefficients},
    generalized::Generalized,
};

#[derive(Debug, Clone)]
pub struct MelCepstrum {
    pub(super) buffer: Box<[f64]>,
    pub(super) alpha: f64,
}

deref_buffer!(MelCepstrum);

impl MelCepstrum {
    pub fn new(c: &[f64], alpha: f64) -> Self {
        Self {
            buffer: c.into(),
            alpha,
        }
    }

    pub fn postfilter_mcp(&mut self, beta: f64) {
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
        Self::Coef::new(self)
    }

    fn clone_with_size(&self, size: usize) -> Self {
        Self {
            buffer: boxed_slice![0.0; size],
            alpha: self.alpha,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MelGeneralizedCepstrum {
    pub(super) buffer: Box<[f64]>,
    pub(super) alpha: f64,
    pub(super) gamma: f64,
}

deref_buffer!(MelGeneralizedCepstrum);

impl MelGeneralizedCepstrum {
    fn gc2gc(&self, m2: usize, gamma: f64) -> Self {
        let mut cepstrum = Self {
            buffer: boxed_slice![0.0; m2 + 1],
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

    pub fn mgc2mgc(&self, m2: usize, alpha: f64, gamma: f64) -> Self {
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
        Self::Coef::new(self, self.gamma)
    }

    fn clone_with_size(&self, size: usize) -> Self {
        Self {
            buffer: boxed_slice![0.0; size],
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

pub trait CepstrumT: Buffer + Sized {
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
        let mut f = boxed_slice![0.0; cepstrum.len()];

        for i in 0..self.len() {
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

    fn c2ir(&self, len: usize) -> Box<[f64]> {
        let mut ir = boxed_slice![0.0; len];
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
