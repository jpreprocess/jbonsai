use super::{
    buffer::*,
    cepstrum::{CepstrumT, MelCepstrum, MelGeneralizedCepstrum},
    generalized::Generalized,
};

#[derive(Debug, Clone)]
pub struct Coefficients {
    pub(super) buffer: Vec<f64>,
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
pub struct GeneralizedCoefficients {
    pub(super) buffer: Vec<f64>,
    pub(super) gamma: f64,
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

pub trait CoefficientsT: Buffer {
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
