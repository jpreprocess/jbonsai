//! MLPG speech parameter generation algorithm.
//!
//! For details, please refer to <https://doi.org/10.1109/ICASSP.2000.861820>.

use crate::model::{GvParameter, MeanVari, Windows};

use super::{IterExt, mask::Mask};

const W1: f64 = 1.0;
const W2: f64 = 1.0;

/// MLPG matrices.
#[derive(Debug, Clone)]
pub struct MlpgMatrix {
    win_size: usize,
    length: usize,
    width: usize,
    wuw: Box<[f64]>,
    wum: Box<[f64]>,
}

impl MlpgMatrix {
    /// Calculate W^T U^{-1} W and W^T U^{-1} \mu
    /// (preparation for calculation of dynamic feature)
    pub fn calc_wuw_and_wum(windows: &Windows, parameters: Vec<Vec<MeanVari>>) -> Self {
        let length = parameters[0].len();
        let width = windows.max_width() * 2 + 1;
        let mut wum = boxed_slice!(0.0; length);
        let mut wuw = boxed_slice!(0.0; width * length);

        for t in 0..length {
            for (i, window) in windows.iter().enumerate() {
                for (index, coef) in window.iter_rev(0) {
                    if coef == 0.0 {
                        continue;
                    }

                    let idx = (t as isize) - index.position();
                    if idx < 0 || idx >= length as isize {
                        continue;
                    }
                    let wu = coef * parameters[i][idx as usize].1;
                    wum[t] += wu * parameters[i][idx as usize].0;

                    for (inner_index, coef) in window.iter_rev(index.index()) {
                        if coef == 0.0 {
                            continue;
                        }
                        let j = inner_index.index() - index.index();
                        if t + j >= length {
                            break;
                        }

                        wuw[width * t + j] += wu * coef;
                    }
                }
            }
        }

        Self {
            win_size: windows.size(),
            length,
            width,
            wuw,
            wum,
        }
    }

    /// Solve equation $W^T U^{-1} W c = W^T U^{-1} \mu$ and return the vector $c$.
    pub fn solve(&mut self) -> Box<[f64]> {
        self.ldl_factorization();
        self.substitutions()
    }

    /// Perform Cholesky decomposition.
    fn ldl_factorization(&mut self) {
        for t in 0..self.length {
            for i in 1..self.width.min(t + 1) {
                self.wuw[self.width * t] -= self.wuw[self.width * (t - i) + i]
                    * self.wuw[self.width * (t - i) + i]
                    * self.wuw[self.width * (t - i)];
            }
            for i in 1..self.width {
                for j in 1..(self.width - i).min(t + 1) {
                    self.wuw[self.width * t + i] -= self.wuw[self.width * (t - j) + j]
                        * self.wuw[self.width * (t - j) + i + j]
                        * self.wuw[self.width * (t - j)];
                }
                self.wuw[self.width * t + i] /= self.wuw[self.width * t];
            }
        }
    }

    /// Forward & backward substitution.
    fn substitutions(&self) -> Box<[f64]> {
        let mut g = boxed_slice![0.0; self.length];
        // forward
        for t in 0..self.length {
            g[t] = self.wum[t];
            for i in 1..self.width.min(t + 1) {
                g[t] -= self.wuw[self.width * (t - i) + i] * g[t - i];
            }
        }

        let mut par = boxed_slice![0.0; self.length];
        // backward
        for t in (0..self.length).rev() {
            par[t] = g[t] / self.wuw[self.width * t];
            for i in 1..self.width.min(self.length - t) {
                par[t] -= self.wuw[self.width * t + i] * par[t + i];
            }
        }

        par
    }

    /// Solve the equasion, and if necessary, applies GV (global variance).
    pub fn par(
        &mut self,
        gv: &Option<GvParameter>,
        vector_index: usize,
        gv_weight: f64,
        durations: &[usize],
        msd_flag: &Mask,
    ) -> Box<[f64]> {
        if let Some((gv_param, gv_switch)) = gv {
            let mtx_before = self.clone();
            let par = self.solve();
            let gv_switch: Vec<_> = gv_switch
                .iter()
                .copied()
                .duration(durations)
                .filter_by(msd_flag.mask())
                .collect();
            let mgv = MlpgGlobalVariance::new(mtx_before, par, &gv_switch);

            let MeanVari(gv_mean, gv_vari) = gv_param[vector_index];
            mgv.apply_gv(gv_mean * gv_weight, gv_vari)
        } else {
            self.solve()
        }
    }
}

/// MLPG global variance (GV) calculator.
#[derive(Debug, Clone)]
pub struct MlpgGlobalVariance<'a> {
    par: Box<[f64]>,
    gv_switch: &'a [bool],
    gv_length: usize,

    mtx: MlpgMatrix,
}

impl<'a> MlpgGlobalVariance<'a> {
    /// Create a new GV structure.
    pub fn new(mtx: MlpgMatrix, par: Box<[f64]>, gv_switch: &'a [bool]) -> Self {
        let gv_length = gv_switch.iter().filter(|b| **b).count();
        Self {
            par,
            gv_switch,
            gv_length,
            mtx,
        }
    }

    /// Apply GV to the current parameter and returns it.
    pub fn apply_gv(mut self, gv_mean: f64, gv_vari: f64) -> Box<[f64]> {
        self.parmgen(gv_mean, gv_vari);
        self.par
    }

    fn calc_gv(&self) -> (f64, f64) {
        let mean = self
            .par
            .iter()
            .zip(self.gv_switch.iter())
            .filter(|(_, sw)| **sw)
            .map(|(p, _)| *p)
            .sum::<f64>()
            / self.gv_length as f64;
        let vari = self
            .par
            .iter()
            .zip(self.gv_switch.iter())
            .filter(|(_, sw)| **sw)
            .map(|(p, _)| (*p - mean) * (*p - mean))
            .sum::<f64>()
            / self.gv_length as f64;

        (mean, vari)
    }

    /// Adjust parameter's deviation from mean value using gv_mean
    fn conv_gv(&mut self, gv_mean: f64) {
        let (mean, vari) = self.calc_gv();
        let ratio = (gv_mean / vari).sqrt();
        self.par
            .iter_mut()
            .zip(self.gv_switch.iter())
            .filter(|(_, sw)| **sw)
            .for_each(|(p, _)| *p = ratio * (*p - mean) + mean);
    }
    fn calc_hmmobj_derivative(&self) -> (f64, Box<[f64]>) {
        let mut g = boxed_slice![0.0; self.mtx.length];

        #[allow(clippy::needless_range_loop)]
        for t in 0..self.mtx.length {
            g[t] = self.mtx.wuw[self.mtx.width * t] * self.par[t];
            for i in 1..self.mtx.width {
                if t + i < self.mtx.length {
                    g[t] += self.mtx.wuw[self.mtx.width * t + i] * self.par[t + i];
                }
                if t + 1 > i {
                    g[t] += self.mtx.wuw[self.mtx.width * (t - i) + i] * self.par[t - i];
                }
            }
        }

        let w = 1.0 / ((self.mtx.win_size * self.mtx.length) as f64);
        let mut hmmobj = 0.0;

        #[allow(clippy::needless_range_loop)]
        for t in 0..self.mtx.length {
            hmmobj += W1 * w * self.par[t] * (self.mtx.wum[t] - 0.5 * g[t]);
        }

        (hmmobj, g)
    }
    fn next_step(
        &mut self,
        g: &[f64],
        step: f64,
        mean: f64,
        vari: f64,
        gv_mean: f64,
        gv_vari: f64,
    ) {
        let length = self.mtx.length;

        let w = 1.0 / ((self.mtx.win_size * length) as f64);
        let dv = -2.0 * gv_vari * (vari - gv_mean) / self.mtx.length as f64;

        #[allow(clippy::needless_range_loop)]
        for t in 0..length {
            let h = -W1 * w * self.mtx.wuw[self.mtx.width * t]
                - W2 * 2.0 / (length * length) as f64
                    * ((length - 1) as f64 * gv_vari * (vari - gv_mean)
                        + 2.0 * gv_vari * (self.par[t] - mean) * (self.par[t] - mean));
            let next_g = if self.gv_switch[t] {
                1.0 / h * (W1 * w * (-g[t] + self.mtx.wum[t]) + W2 * dv * (self.par[t] - mean))
            } else {
                1.0 / h * (W1 * w * (-g[t] + self.mtx.wum[t]))
            };

            self.par[t] += step * next_g;
        }
    }

    fn parmgen(&mut self, gv_mean: f64, gv_vari: f64) {
        const GV_MAX_ITERATION: usize = 5;
        const STEPINIT: f64 = 0.1;
        const STEPDEC: f64 = 0.5;
        const STEPINC: f64 = 1.2;

        if self.gv_length == 0 || GV_MAX_ITERATION == 0 {
            return;
        }

        let mut step = STEPINIT;
        let mut prev = 0.0;
        self.conv_gv(gv_mean);
        for i in 1..=GV_MAX_ITERATION {
            let (mean, vari) = self.calc_gv();

            let gvobj = -0.5 * W2 * vari * gv_vari * (vari - 2.0 * gv_mean);
            let (hmmobj, g) = self.calc_hmmobj_derivative();
            let obj = -(hmmobj + gvobj);

            if i > 1 {
                if obj > prev {
                    step *= STEPDEC;
                } else if obj < prev {
                    step *= STEPINC;
                }
            }

            self.next_step(&g, step, mean, vari, gv_mean, gv_vari);

            prev = obj;
        }
    }
}
