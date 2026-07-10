use super::{
    buffer::*,
    cepstrum::{CepstrumT, MelCepstrum, MelGeneralizedCepstrum},
    generalized::Generalized,
};

#[derive(Debug, Clone)]
pub struct Coefficients {
    buffer: Vec<f64>,
}

deref_buffer!(Coefficients);

impl Coefficients {
    pub fn new(c: &[f64]) -> Self {
        Self { buffer: c.to_vec() }
    }
}

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
pub struct GeneralizedCoefficients {
    buffer: Vec<f64>,
    gamma: f64,
}

deref_buffer!(GeneralizedCoefficients);

impl GeneralizedCoefficients {
    pub fn new(c: &[f64], gamma: f64) -> Self {
        Self {
            buffer: c.to_vec(),
            gamma,
        }
    }
}

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

impl Generalized for GeneralizedCoefficients {
    fn gamma(&self) -> f64 {
        self.gamma
    }
}

pub trait CoefficientsT: Buffer + Sized {
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

    fn start(&mut self, target: Self, fperiod: usize) -> CoefficientsSession<'_, Self> {
        let increment = self
            .iter()
            .zip(&target[..self.len()])
            .map(|(current, target)| (target - current) / fperiod as f64)
            .collect();
        CoefficientsSession {
            current: self,
            increment,
            target,
        }
    }
}

pub struct CoefficientsSession<'a, T: CoefficientsT> {
    current: &'a mut T,
    increment: Vec<f64>,
    target: T,
}

impl<'a, T: CoefficientsT> CoefficientsSession<'a, T> {
    pub fn advance(&mut self) {
        for (current, inc) in self.current.iter_mut().zip(&self.increment) {
            *current += inc;
        }
    }
}

impl<T: CoefficientsT> Deref for CoefficientsSession<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.current
    }
}

impl<T: CoefficientsT> Drop for CoefficientsSession<'_, T> {
    fn drop(&mut self) {
        self.current.copy_from_slice(&self.target);
    }
}
