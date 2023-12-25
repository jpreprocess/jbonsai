use crate::sstream::StateStreamSet;

use self::mlpg::{MlpgGlobalVariance, MlpgMatrix};
mod mlpg;

pub struct ParameterStreamSet {
    streams: Vec<ParameterStream>,
}

pub struct ParameterStream {
    par: Vec<Vec<f64>>,
    msd_flag: Option<Vec<bool>>,
}

impl ParameterStreamSet {
    /// Parameter generation using GV weight
    pub fn create(sss: &StateStreamSet, msd_threshold: &[f64], gv_weight: &[f64]) -> ParameterStreamSet {
        let mut streams = Vec::with_capacity(sss.get_nstream());
        for i in 0..sss.get_nstream() {
            let msd_flag = if sss.is_msd(i) {
                let msd_flag: Vec<_> = (0..sss.get_total_state())
                    .flat_map(|state| {
                        let flag = sss.get_msd(i, state) > msd_threshold[i];
                        [flag].repeat(sss.get_duration(state))
                    })
                    .collect();
                Some(msd_flag)
            } else {
                None
            };

            let (msd_left_boundaries, msd_right_boundaries) =
                Self::msd_boundary_distances(sss.get_total_frame(), &msd_flag);

            let gv_switch = if sss.use_gv(i) {
                let gv_switch: Vec<bool> = (0..sss.get_total_state())
                    .flat_map(|state| (0..sss.get_duration(state)).map(move |j| (state, j)))
                    .enumerate()
                    .filter(|(frame, _)| msd_flag.is_none() || msd_flag.as_ref().unwrap()[*frame])
                    .map(|(_, (state, _))| sss.get_gv_switch(i, state))
                    .collect();
                Some(gv_switch)
            } else {
                None
            };

            let mut pars = Vec::with_capacity(sss.get_vector_length(i));
            for vector_index in 0..sss.get_vector_length(i) {
                let parameters: Vec<Vec<(f64, f64)>> = (0..sss.get_window_size(i))
                    .map(|window_index| {
                        let m = sss.get_vector_length(i) * window_index + vector_index;

                        (0..sss.get_total_state())
                            // iterate over frames
                            .flat_map(|state| [state].repeat(sss.get_duration(state)))
                            // add frame index
                            .enumerate()
                            // filter by msd
                            .filter(|(frame, _)| {
                                msd_flag.is_none() || msd_flag.as_ref().unwrap()[*frame]
                            })
                            .zip(std::iter::repeat((m, window_index)))
                            .map(|((frame, state), (m, window_index))| {
                                let is_msd_boundary = sss.get_window_left_width(i, window_index)
                                    < -(msd_left_boundaries[frame] as isize)
                                    || (msd_right_boundaries[frame] as isize)
                                        < sss.get_window_right_width(i, window_index);

                                let mean = sss.get_mean(i, state, m);
                                let ivar = if !is_msd_boundary || window_index == 0 {
                                    let vari = sss.get_vari(i, state, m);
                                    if vari.abs() > 1e19 {
                                        0.0
                                    } else if vari.abs() < 1e-19 {
                                        1e38
                                    } else {
                                        1.0 / vari
                                    }
                                } else {
                                    0.0
                                };

                                (mean, ivar)
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

                    MlpgGlobalVariance::new(mtx_before, par, gv_switch.as_ref().unwrap())
                        .apply_gv(gv_mean, gv_vari)
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
    fn msd_boundary_distances(
        total_frame: usize,
        msd_flag: &Option<Vec<bool>>,
    ) -> (Vec<usize>, Vec<usize>) {
        if total_frame == 0 {
            return (vec![], vec![]);
        }

        let mut result_left = vec![0; total_frame];
        let mut left = 0;
        for frame in 0..total_frame {
            result_left[frame] = frame - left;

            if matches!(msd_flag, Some(ref msd_flag) if !msd_flag[frame]) {
                // MSD is enabled and current position is non-MSD
                left = frame + 1;
            }
        }

        let mut result_right = vec![0; total_frame];
        let mut right = total_frame - 1;
        for frame in (0..total_frame).rev() {
            result_right[frame] = right - frame;

            if matches!(msd_flag, Some(ref msd_flag) if !msd_flag[frame]) {
                // MSD is enabled and current position is non-MSD
                if frame == 0 {
                    break;
                }
                right = frame - 1;
            }
        }

        (result_left, result_right)
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
        self.streams[stream_index].msd_flag.as_ref().unwrap()[frame_index]
    }
    /// Get MSD flag
    pub fn is_msd(&self, stream_index: usize) -> bool {
        self.streams[stream_index].msd_flag.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::ParameterStreamSet;

    #[test]
    fn msd_boundary_distances() {
        assert_eq!(
            ParameterStreamSet::msd_boundary_distances(10, &None),
            (
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
                vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
            )
        );
        assert_eq!(
            ParameterStreamSet::msd_boundary_distances(
                10,
                &Some(vec![
                    true, true, true, false, false, true, true, true, true, true
                ])
            ),
            (
                vec![0, 1, 2, 3, 0, 0, 1, 2, 3, 4],
                vec![2, 1, 0, 0, 5, 4, 3, 2, 1, 0]
            )
        );
        assert_eq!(
            ParameterStreamSet::msd_boundary_distances(0, &None),
            (vec![], vec![])
        );
    }
}
