use std::f64::consts::PI;

use super::{buffer::*, cepstrum::MelGeneralizedCepstrum, generalized::Generalized};

#[derive(Debug, Clone)]
pub struct LineSpectralPairs {
    buffer: Box<[f64]>,
    alpha: f64,
    use_log_gain: bool,
    stage: usize,
    gamma: f64,
}

deref_buffer!(LineSpectralPairs);

impl LineSpectralPairs {
    pub fn new(lsp: &[f64], alpha: f64, use_log_gain: bool, stage: usize, gamma: f64) -> Self {
        Self {
            buffer: lsp.into(),
            alpha,
            use_log_gain,
            stage,
            gamma,
        }
    }

    fn lsp2lpc(&self) -> MelGeneralizedCepstrum {
        let m = self.len();
        let (mh1, mh2) = if m.is_multiple_of(2) {
            (m / 2, m / 2)
        } else {
            #[allow(clippy::manual_div_ceil)] // The semantics of this division is not div_ceil
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
            buffer: vec![0.0; m + 1].into(),
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

    pub fn lsp2mgc(&self) -> MelGeneralizedCepstrum {
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

    fn lsp2en(&self) -> f64 {
        self.lsp2mgc().iter().map(|x| x * x).sum()
    }

    pub fn postfilter_lsp(&mut self, beta: f64) {
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
            self.copy_from_slice(&buf);

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

    pub fn check_lsp_stability(&mut self) {
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
