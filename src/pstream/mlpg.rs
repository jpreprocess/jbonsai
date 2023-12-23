use crate::{model::ModelSet, sstream::SStreamSet};

const W1: f64 = 1.0;
const W2: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct MlpgMatrix {
    length: usize,
    width: usize,
    wuw: Vec<Vec<f64>>,
    wum: Vec<f64>,
}

impl MlpgMatrix {
    pub fn new() -> Self {
        Self {
            length: 0,
            width: 0,
            wuw: Vec::new(),
            wum: Vec::new(),
        }
    }

    pub fn calc_wuw_and_wum(
        &mut self,
        sss: &SStreamSet,
        stream_index: usize,
        parameters: Vec<Vec<(f64, f64)>>,
    ) {
        self.length = parameters.len();
        self.width = parameters[0].len();

        self.wuw = Vec::new();
        self.wum = Vec::new();

        for t in 0..self.length {
            self.wuw.push(vec![0.; self.width]);
            self.wum.push(0.);

            for i in 0..sss.get_window_size(stream_index) {
                for shift in sss.get_window_left_width(stream_index, i)
                    ..sss.get_window_right_width(stream_index, i) + 1
                {
                    let idx = (t as isize) + (shift as isize);
                    if idx < 0 || idx >= self.length as isize {
                        continue;
                    }
                    let coef = sss.get_window_coefficient(stream_index, i, -shift);
                    if coef == 0. {
                        continue;
                    }

                    let wu = coef * parameters[i][idx as usize].1;
                    self.wum[t] += wu * parameters[i][idx as usize].0;

                    for j in 0..self.width {
                        if t + j >= self.length
                            || j as isize > sss.get_window_right_width(stream_index, i) + shift
                        {
                            break;
                        }
                        let coef = sss.get_window_coefficient(stream_index, i, j as isize - shift);
                        if coef == 0. {
                            continue;
                        }

                        self.wuw[t][j] += wu * coef;
                    }
                }
            }
        }
    }

    pub fn solve(&mut self) -> Vec<f64> {
        self.ldl_factorization();
        self.substitutions()
    }

    fn ldl_factorization(&mut self) {
        for t in 0..self.length {
            for i in 1..self.width.min(t + 1) {
                self.wuw[t][0] -= self.wuw[t - i][i] * self.wuw[t - i][i] * self.wuw[t - i][0];
            }
            for i in 1..self.width {
                for j in 1..self.width.min(t + 1) {
                    self.wuw[t][i] -=
                        self.wuw[t - j][j] * self.wuw[t - j][i + j] * self.wuw[t - j][0];
                }
                self.wuw[t][i] /= self.wuw[t][0];
            }
        }
    }

    fn substitutions(&self) -> Vec<f64> {
        let mut g = vec![0.; self.length];
        // forward
        for t in 0..self.length {
            g[t] = self.wum[t];
            for i in 1..self.width.min(t + 1) {
                g[t] -= self.wuw[t - i][i] * g[t - i];
            }
        }

        let mut par = vec![0.; self.length];
        // backward
        for rev in 0..self.length {
            let t = self.length - 1 - rev;
            par[t] = g[t] / self.wuw[t][0];
            for i in 1..self.width.min(self.length - t) {
                par[t] -= self.wuw[t][i] * par[t + 1];
            }
        }

        par
    }
}

#[derive(Debug, Clone)]
pub struct MlpgGlobalVariance<'a> {
    par: Vec<f64>,
    gv_switch: &'a [bool],
    gv_length: usize,

    mtx: MlpgMatrix,
}

impl<'a> MlpgGlobalVariance<'a> {
    pub fn new(mtx: MlpgMatrix, par: Vec<f64>, gv_switch: &'a [bool]) -> Self {
        let gv_length = gv_switch.iter().map(|b| *b as usize).sum::<usize>();
        Self {
            par,
            gv_switch,
            gv_length,
            mtx,
        }
    }

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
    fn conv_gv(&mut self, gv_mean: f64) {
        let (mean, vari) = self.calc_gv();
        let ratio = (gv_mean / vari).sqrt();
        self.par
            .iter_mut()
            .zip(self.gv_switch.iter())
            .filter(|(_, sw)| **sw)
            .for_each(|(p, _)| *p = ratio * (*p - mean) + mean);
    }
    fn calc_hmmobj_derivative(&self) -> (f64, Vec<f64>) {
        let mut g = vec![0.; self.gv_switch.len()];
        for t in 0..self.gv_switch.len() {
            g[t] = self.mtx.wuw[t][0] * self.par[t];
            for i in 1..self.mtx.width {
                if t + i < self.gv_switch.len() {
                    g[t] += self.mtx.wuw[t][i] * self.par[t + i];
                }
                if t + 1 < i {
                    g[t] += self.mtx.wuw[t - i][i] * self.par[t - i];
                }
            }
        }

        let w = 1.0 / ((self.mtx.width * self.gv_switch.len()) as f64);
        let mut hmmobj = 0.;
        for t in 0..self.gv_switch.len() {
            hmmobj += W1 * w * self.par[t] * (self.mtx.wum[t] - 0.5 * g[t]);
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
        let length = self.gv_switch.len();

        let w = 1.0 / ((self.mtx.width * length) as f64);
        let dv = -2.0 * gv_vari * (vari - gv_mean) / self.gv_switch.len() as f64;
        for t in 0..length {
            let h = -W1 * w * self.mtx.wuw[t][0]
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
        for i in 1..GV_MAX_ITERATION {
            let (mean, vari) = self.calc_gv();

            let gvobj = -0.5 * W2 * vari * gv_vari * (vari - 2.0 * gv_mean);
            let (hmmobj, g) = self.calc_hmmobj_derivative();
            let obj = -(hmmobj + gvobj);

            if i > 1 {
                if obj < prev {
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
