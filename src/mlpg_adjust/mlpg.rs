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
        let Self { length, width, .. } = *self;
        let wuw = &mut self.wuw[..width * length];

        for t in 0..length {
            for i in 1..width.min(t + 1) {
                wuw[width * t] -=
                    wuw[width * (t - i) + i] * wuw[width * (t - i) + i] * wuw[width * (t - i)];
            }
            for i in 1..width {
                for j in 1..(width - i).min(t + 1) {
                    wuw[width * t + i] -= wuw[width * (t - j) + j]
                        * wuw[width * (t - j) + i + j]
                        * wuw[width * (t - j)];
                }
                wuw[width * t + i] /= wuw[width * t];
            }
        }
    }

    /// Forward & backward substitution.
    fn substitutions(&self) -> Box<[f64]> {
        let Self { length, width, .. } = *self;
        let wuw = &self.wuw[..width * length];
        let wum = &self.wum[..length];
        let mut g = boxed_slice![0.0; length];
        // forward
        for t in 0..length {
            g[t] = wum[t];
            for i in 1..width.min(t + 1) {
                g[t] -= wuw[width * (t - i) + i] * g[t - i];
            }
        }

        let mut par = boxed_slice![0.0; length];
        // backward
        for t in (0..length).rev() {
            par[t] = g[t] / wuw[width * t];
            for i in 1..width.min(length - t) {
                par[t] -= wuw[width * t + i] * par[t + i];
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
        let mut sum = 0.0;
        let mut sum_quad = 0.0;

        for (par, sw) in std::iter::zip(&self.par, self.gv_switch) {
            if *sw {
                sum += *par;
                sum_quad += *par * *par;
            }
        }

        let mean = sum / self.gv_length as f64;
        let vari = (sum_quad / self.gv_length as f64) - (mean * mean);
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
        let MlpgMatrix {
            win_size,
            length,
            width,
            ..
        } = self.mtx;
        assert!(width >= 1); // required for `wuw[0]` access
        let wuw = self.mtx.wuw.chunks_exact(width);
        let wum = &self.mtx.wum[..length];
        let par = &self.par[..length];
        let mut g = boxed_slice![0.0; length];

        // .zip(0..length) to help optimizer recognize t < length
        for (wuw, t) in wuw.zip(0..length) {
            g[t] += wuw[0] * par[t];
            for i in 1..width {
                if t + i < length {
                    g[t] += wuw[i] * par[t + i];
                    g[t + i] += wuw[i] * par[t];
                }
            }
        }

        let w = 1.0 / ((win_size * length) as f64);
        let mut hmmobj = 0.0;

        for t in 0..length {
            hmmobj += W1 * w * par[t] * (wum[t] - 0.5 * g[t]);
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
        let MlpgMatrix {
            win_size,
            length,
            width,
            ..
        } = self.mtx;
        let wuw = &self.mtx.wuw[..width * length];
        let wum = &self.mtx.wum[..length];
        let par = &mut self.par[..length];
        let gv_switch = &self.gv_switch[..length];

        let w = 1.0 / ((win_size * length) as f64);
        let dv = -2.0 * gv_vari * (vari - gv_mean) / length as f64;

        for t in 0..length {
            let h = -W1 * w * wuw[width * t]
                - W2 * 2.0 / (length * length) as f64
                    * ((length - 1) as f64 * gv_vari * (vari - gv_mean)
                        + 2.0 * gv_vari * (par[t] - mean) * (par[t] - mean));
            let next_g = if gv_switch[t] {
                1.0 / h * (W1 * w * (-g[t] + wum[t]) + W2 * dv * (par[t] - mean))
            } else {
                1.0 / h * (W1 * w * (-g[t] + wum[t]))
            };

            par[t] += step * next_g;
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
