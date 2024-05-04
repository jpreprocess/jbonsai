use crate::{
    constants::NODATA,
    model::{Models, StreamParameter},
};

mod mask;
mod mlpg;

use self::{
    mask::Mask,
    mlpg::{MlpgGlobalVariance, MlpgMatrix},
};

pub struct MlpgAdjust {
    stream_index: usize,
    gv_weight: f64,
    msd_threshold: f64,
}

impl MlpgAdjust {
    pub fn new(stream_index: usize, gv_weight: f64, msd_threshold: f64) -> Self {
        Self {
            stream_index,
            gv_weight,
            msd_threshold,
        }
    }
    /// Parameter generation using GV weight
    pub fn create(
        &self,
        stream: StreamParameter,
        models: &Models,
        durations: &[usize],
    ) -> Vec<Vec<f64>> {
        let vector_length = models.vector_length(self.stream_index);

        let msd_flag: Mask = stream
            .iter()
            .zip(durations)
            .flat_map(|((_, msd), duration)| {
                let flag = *msd > self.msd_threshold;
                [flag].repeat(*duration)
            })
            .collect();

        let msd_boundaries = msd_flag.boundary_distances();

        let mut pars = vec![vec![0.0; vector_length]; msd_flag.mask().len()];
        for vector_index in 0..vector_length {
            let parameters: Vec<Vec<(f64, f64)>> = models
                .windows(self.stream_index)
                .iter()
                .enumerate()
                .map(|(window_index, window)| {
                    let m = vector_length * window_index + vector_index;

                    let mut iter = msd_flag.mask().iter();
                    stream
                        .iter()
                        .zip(durations)
                        // get mean and ivar, and spread it to its duration
                        .flat_map(|((curr_stream, _), duration)| {
                            let (mean, vari) = curr_stream[m];
                            let ivar = {
                                if vari.abs() > 1e19 {
                                    0.0
                                } else if vari.abs() < 1e-19 {
                                    1e38
                                } else {
                                    1.0 / vari
                                }
                            };
                            [(mean, ivar)].repeat(*duration)
                        })
                        .zip(&msd_boundaries)
                        .map(|((mean, ivar), (left, right))| {
                            let is_left_msd_boundary = *left < window.left_width();
                            let is_right_msd_boundary = *right < window.right_width();

                            // If the window includes non-msd frames, set the ivar to 0.0
                            if (is_left_msd_boundary || is_right_msd_boundary) && window_index != 0
                            {
                                (mean, 0.0)
                            } else {
                                (mean, ivar)
                            }
                        })
                        .filter(|_| iter.next() == Some(&true))
                        .collect()
                })
                .collect();

            let mut mtx = MlpgMatrix::new();
            mtx.calc_wuw_and_wum(models.windows(self.stream_index), parameters);

            let par = if let Some((gv_param, gv_switch)) = models.gv(self.stream_index) {
                let mtx_before = mtx.clone();
                let par = mtx.solve();

                let gv_mean = gv_param[vector_index].0 * self.gv_weight;
                let gv_vari = gv_param[vector_index].1;

                let mut iter = msd_flag.mask().iter();
                let gv_switch: Vec<bool> = gv_switch
                    .iter()
                    .zip(durations)
                    .flat_map(|(switch, duration)| [*switch].repeat(*duration))
                    .filter(|_| iter.next() == Some(&true))
                    .collect();

                MlpgGlobalVariance::new(mtx_before, par, &gv_switch).apply_gv(gv_mean, gv_vari)
            } else {
                mtx.solve()
            };

            pars.iter_mut()
                .zip(msd_flag.fill(par.into_iter(), NODATA))
                .for_each(|(par, value)| {
                    par[vector_index] = value;
                });
        }

        pars
    }
}
