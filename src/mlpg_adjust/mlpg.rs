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
    wuw: Vec<f64>,
    wum: Vec<f64>,
}

impl MlpgMatrix {
    /// Calculate W^T U^{-1} W and W^T U^{-1} \mu
    /// (preparation for calculation of dynamic feature)
    pub fn calc_wuw_and_wum(windows: &Windows, parameters: Vec<Vec<MeanVari>>) -> Self {
        let length = parameters[0].len();
        let width = windows.max_width() * 2 + 1;
        let mut wum = vec![0.0; length];
        let mut wuw = vec![0.0; length * width];

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

                        wuw[t * width + j] += wu * coef;
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
    pub fn solve(&mut self) -> Vec<f64> {
        self.ldl_factorization();
        self.substitutions()
    }

    /// Perform Cholesky decomposition.
    fn ldl_factorization(&mut self) {
        let Self { width, length, .. } = *self;
        let wuw = &mut self.wuw[..length * width];
        for t in 0..length {
            for i in 1..width.min(t + 1) {
                wuw[width * t] -=
                    wuw[(t - i) * width + i] * wuw[(t - i) * width + i] * wuw[(t - i) * width];
            }
            for i in 1..width {
                for j in 1..(width - i).min(t + 1) {
                    wuw[width * t + i] -= wuw[(t - j) * width + j]
                        * wuw[(t - j) * width + i + j]
                        * wuw[(t - j) * width];
                }
                wuw[width * t + i] /= wuw[width * t];
            }
        }
    }

    /// Forward & backward substitution.
    fn substitutions(&self) -> Vec<f64> {
        let Self { width, length, .. } = *self;
        let wum = &self.wum[..length];
        let wuw = &self.wuw[..length * width];
        let mut g = vec![0.0; self.length];
        // forward
        for t in 0..length {
            g[t] = wum[t];
            for i in 1..width.min(t + 1) {
                g[t] -= wuw[(t - i) * width + i] * g[t - i];
            }
        }

        let mut par = vec![0.0; self.length];
        // backward
        for t in (0..self.length).rev() {
            par[t] = g[t] / wuw[t * width];
            for i in 1..width.min(length - t) {
                par[t] -= wuw[t * width + i] * par[t + i];
            }
        }

        par
    }

    fn calculate_gv_switch(gv_switch: &[bool], durations: &[usize], mask: &[bool]) -> Vec<bool> {
        gv_switch
            .iter()
            .copied()
            .duration(durations)
            .filter_by(mask)
            .collect()
    }

    /// Solve the equasion, and if necessary, applies GV (global variance).
    pub fn par(
        &mut self,
        gv: &Option<GvParameter>,
        vector_index: usize,
        gv_weight: f64,
        durations: &[usize],
        msd_flag: &Mask,
    ) -> Vec<f64> {
        if let Some((gv_param, gv_switch)) = gv {
            let mtx_before = self.clone();
            let par = self.solve();
            let gv_switch: Vec<_> =
                Self::calculate_gv_switch(gv_switch, durations, msd_flag.mask());
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
    par: Vec<f64>,
    gv_switch: &'a [bool],
    gv_length: usize,

    mtx: MlpgMatrix,
}

impl<'a> MlpgGlobalVariance<'a> {
    /// Create a new GV structure.
    pub fn new(mtx: MlpgMatrix, par: Vec<f64>, gv_switch: &'a [bool]) -> Self {
        let gv_length = gv_switch.iter().filter(|b| **b).count();
        Self {
            par,
            gv_switch,
            gv_length,
            mtx,
        }
    }

    /// Apply GV to the current parameter and returns it.
    pub fn apply_gv(mut self, gv_mean: f64, gv_vari: f64) -> Vec<f64> {
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
    #[inline(never)]
    fn calc_hmmobj_derivative(&self) -> (f64, Vec<f64>) {
        let MlpgMatrix {
            width,
            length,
            win_size,
            ..
        } = self.mtx;
        let wuw = &self.mtx.wuw[..length * width];
        let wum = &self.mtx.wum[..length];
        let par = &self.par[..length];

        let mut g = vec![0.0; length];

        for t in 0..length {
            g[t] = wuw[t * width] * par[t];
            for i in 1..width {
                if i < length - t {
                    g[t] += wuw[t * width + i] * par[t + i];
                }
            }
        }

        for t in 0..length {
            for i in 1..width {
                if i < length - t {
                    g[t + i] += wuw[t * width + i] * par[t];
                }
            }
        }

        let w = 1.0 / ((win_size * length) as f64);
        let mut hmmobj = 0.0;

        #[allow(clippy::needless_range_loop)]
        for t in 0..length {
            hmmobj += W1 * w * par[t] * (wum[t] - 0.5 * g[t]);
        }

        (hmmobj, g)
    }
    fn next_step(
        &mut self,
        g: Vec<f64>,
        step: f64,
        mean: f64,
        vari: f64,
        gv_mean: f64,
        gv_vari: f64,
    ) {
        let MlpgMatrix { width, length, .. } = self.mtx;

        let w = 1.0 / ((self.mtx.win_size * length) as f64);
        let dv = -2.0 * gv_vari * (vari - gv_mean) / self.mtx.length as f64;

        let wum = &self.mtx.wum[..length];
        let wuw = &self.mtx.wuw[..length * width];

        for t in 0..length {
            let h = -W1 * w * wuw[t * width]
                - W2 * 2.0 / (length * length) as f64
                    * ((length - 1) as f64 * gv_vari * (vari - gv_mean)
                        + 2.0 * gv_vari * (self.par[t] - mean) * (self.par[t] - mean));
            let next_g = if self.gv_switch[t] {
                1.0 / h * (W1 * w * (-g[t] + wum[t]) + W2 * dv * (self.par[t] - mean))
            } else {
                1.0 / h * (W1 * w * (-g[t] + wum[t]))
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

            self.next_step(g, step, mean, vari, gv_mean, gv_vari);

            prev = obj;
        }
    }
}
