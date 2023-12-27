use crate::sstream::StateStreamSet;

use self::mlpg::{MlpgGlobalVariance, MlpgMatrix};
mod mlpg;

pub struct ParameterStreamSet {
    streams: Vec<ParameterStream>,
}

pub struct ParameterStream {
    par: Vec<Vec<f64>>,
    msd_flag: Vec<bool>,
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
                [true].repeat(sss.get_total_frame())
            };

            let msd_boundaries = Self::msd_boundary_distances(sss.get_total_frame(), &msd_flag);

            let mut pars = Vec::with_capacity(sss.get_vector_length(i));
            for vector_index in 0..sss.get_vector_length(i) {
                let parameters: Vec<Vec<(f64, f64)>> = (0..sss.get_window_size(i))
                    .map(|window_index| {
                        let m = sss.get_vector_length(i) * window_index + vector_index;

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
                            // add frame index
                            .enumerate()
                            // filter by msd
                            .filter(|(frame, _)| msd_flag[*frame])
                            // apply boundary condition
                            .map(|(frame, (mean, ivar))| {
                                let (left, right) = msd_boundaries[frame];

                                let is_left_msd_boundary =
                                    sss.get_window_left_width(i, window_index) < -(left as isize);
                                let is_right_msd_boundary =
                                    (right as isize) < sss.get_window_right_width(i, window_index);

                                // If the window includes non-msd frames, set the ivar to 0.0
                                if (is_left_msd_boundary || is_right_msd_boundary)
                                    && window_index != 0
                                {
                                    (mean, 0.0)
                                } else {
                                    (mean, ivar)
                                }
                            })
                            .collect()
                    })
                    .collect();

                let mut mtx = MlpgMatrix::new();
                mtx.calc_wuw_and_wum(sss, i, parameters);

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
                        .zip(&msd_flag)
                        .filter(|(_, msd)| **msd)
                        .map(|(data, _)| data)
                        .collect();

                    MlpgGlobalVariance::new(mtx_before, par, &gv_switch).apply_gv(gv_mean, gv_vari)
                } else {
                    mtx.solve()
                };

                pars.push(par);
            }

            streams.push(ParameterStream {
                par: pars,
                msd_flag,
            });
        }

        ParameterStreamSet { streams }
    }

    /// Calculate distance from the closest msd boundaries
    fn msd_boundary_distances(total_frame: usize, msd_flag: &[bool]) -> Vec<(usize, usize)> {
        if total_frame == 0 {
            return vec![];
        }

        let mut result = vec![(0, 0); total_frame];

        let mut left = 0;
        for frame in 0..total_frame {
            if msd_flag[frame] {
                result[frame].0 = frame - left;
            } else {
                // MSD is enabled and current position is non-MSD
                left = frame + 1;
            }
        }

        let mut right = total_frame - 1;
        for frame in (0..total_frame).rev() {
            if msd_flag[frame] {
                result[frame].1 = right - frame;
            } else {
                // MSD is enabled and current position is non-MSD
                if frame == 0 {
                    break;
                }
                right = frame - 1;
            }
        }

        result
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
    /// Get generated MSD flag per frame
    pub fn get_msd_flag(&self, stream_index: usize, frame_index: usize) -> bool {
        self.streams[stream_index].msd_flag[frame_index]
    }
}

#[cfg(test)]
mod tests {
    use super::ParameterStreamSet;

    #[test]
    fn msd_boundary_distances() {
        assert_eq!(
            ParameterStreamSet::msd_boundary_distances(
                10,
                &[true, true, true, true, true, true, true, true, true, true]
            ),
            vec![
                (0, 9),
                (1, 8),
                (2, 7),
                (3, 6),
                (4, 5),
                (5, 4),
                (6, 3),
                (7, 2),
                (8, 1),
                (9, 0)
            ],
        );
        assert_eq!(
            ParameterStreamSet::msd_boundary_distances(
                10,
                &[true, true, true, false, false, true, true, true, true, true]
            ),
            vec![
                (0, 2),
                (1, 1),
                (2, 0),
                (0, 0),
                (0, 0),
                (0, 4),
                (1, 3),
                (2, 2),
                (3, 1),
                (4, 0)
            ]
        );
        assert_eq!(
            ParameterStreamSet::msd_boundary_distances(
                10,
                &[true, true, true, false, true, false, false, false, false, false]
            ),
            vec![
                (0, 2),
                (1, 1),
                (2, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0)
            ]
        );
        assert_eq!(ParameterStreamSet::msd_boundary_distances(0, &[]), vec![]);
    }
}
