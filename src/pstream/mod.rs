use crate::{constants::NODATA, model::Models, sequence::Mask};

use self::mlpg::{MlpgGlobalVariance, MlpgMatrix};
mod mlpg;

pub struct ParameterStreamSet {
    streams: Vec<ParameterStream>,
}

pub struct ParameterStream {
    par: Vec<Vec<f64>>,
}

impl ParameterStreamSet {
    /// Parameter generation using GV weight
    pub fn create(
        models: &Models<'_>,
        durations: &[usize],
        msd_threshold: &[f64],
        gv_weight: &[f64],
    ) -> ParameterStreamSet {
        let mut streams = Vec::with_capacity(models.nstream());
        for i in 0..models.nstream() {
            let stream = models.stream(i);

            let msd_flag: Mask = stream
                .iter()
                .zip(durations)
                .flat_map(|((_, msd), duration)| {
                    let flag = *msd > msd_threshold[i];
                    [flag].repeat(*duration)
                })
                .collect();

            let msd_boundaries = msd_flag.boundary_distances();

            let mut pars = Vec::with_capacity(models.vector_length(i));
            for vector_index in 0..models.vector_length(i) {
                let parameters: Vec<Vec<(f64, f64)>> = models
                    .windows(i)
                    .iter()
                    .enumerate()
                    .map(|(window_index, window)| {
                        let m = models.vector_length(i) * window_index + vector_index;

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
                                if (is_left_msd_boundary || is_right_msd_boundary)
                                    && window_index != 0
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
                mtx.calc_wuw_and_wum(models.windows(i), parameters);

                let par = if let Some((gv_param, gv_switch)) = models.gv(i) {
                    let mtx_before = mtx.clone();
                    let par = mtx.solve();

                    let gv_mean = gv_param[vector_index].0 * gv_weight[i];
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

                pars.push(msd_flag.fill(par, NODATA));
            }

            streams.push(ParameterStream { par: pars });
        }

        ParameterStreamSet { streams }
    }

    /// Get number of stream
    pub fn get_nstream(&self) -> usize {
        self.streams.len()
    }
    /// Get feature length
    pub fn get_vector_length(&self, stream_index: usize) -> usize {
        self.streams[stream_index].par.len()
    }
    /// Get total number of frame
    pub fn get_total_frame(&self) -> usize {
        self.streams[0].par[0].len()
    }
    /// Get parameter
    pub fn get_parameter(
        &self,
        stream_index: usize,
        frame_index: usize,
        vector_index: usize,
    ) -> f64 {
        self.streams[stream_index].par[vector_index][frame_index]
    }
}
