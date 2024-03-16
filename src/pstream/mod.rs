use crate::{constants::NODATA, sequence::Mask, sstream::StateStreamSet};

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
        sss: &StateStreamSet,
        msd_threshold: &[f64],
        gv_weight: &[f64],
    ) -> ParameterStreamSet {
        let mut streams = Vec::with_capacity(sss.get_nstream());
        for i in 0..sss.get_nstream() {
            let msd_flag = if sss.is_msd(i) {
                (0..sss.get_total_state())
                    .flat_map(|state| {
                        let flag = sss.get_msd(i, state) > msd_threshold[i];
                        [flag].repeat(sss.get_duration(state))
                    })
                    .collect()
            } else {
                Mask::new([true].repeat(sss.get_total_frame()))
            };

            let msd_boundaries = msd_flag.boundary_distances();

            let mut pars = Vec::with_capacity(sss.get_vector_length(i));
            for vector_index in 0..sss.get_vector_length(i) {
                let parameters: Vec<Vec<(f64, f64)>> = sss
                    .get_windows(i)
                    .iter()
                    .enumerate()
                    .map(|(window_index, window)| {
                        let m = sss.get_vector_length(i) * window_index + vector_index;

                        let mut iter = msd_flag.mask().iter();
                        (0..sss.get_total_state())
                            // get mean and ivar, and spread it to its duration
                            .flat_map(|state| {
                                let mean = sss.get_mean(i, state, m);
                                let ivar = {
                                    let vari = sss.get_vari(i, state, m);
                                    if vari.abs() > 1e19 {
                                        0.0
                                    } else if vari.abs() < 1e-19 {
                                        1e38
                                    } else {
                                        1.0 / vari
                                    }
                                };
                                [(mean, ivar)].repeat(sss.get_duration(state))
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
                mtx.calc_wuw_and_wum(sss.get_windows(i), parameters);

                let par = if sss.use_gv(i) {
                    let mtx_before = mtx.clone();
                    let par = mtx.solve();

                    let gv_mean = sss.get_gv_mean(i, vector_index) * gv_weight[i];
                    let gv_vari = sss.get_gv_vari(i, vector_index);

                    let gv_switch: Vec<bool> = (0..sss.get_total_state())
                        .flat_map(|state_index| {
                            [sss.get_gv_switch(i, state_index)]
                                .repeat(sss.get_duration(state_index))
                        })
                        .zip(msd_flag.mask())
                        .filter(|(_, msd)| **msd)
                        .map(|(data, _)| data)
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
