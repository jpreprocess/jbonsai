use crate::sstream::SStreamSet;

use self::mlpg::{MlpgGlobalVariance, MlpgMatrix};
mod mlpg;

pub struct PStreamSet {
    streams: Vec<PStream>,
}

pub struct PStream {
    par: Vec<Vec<f64>>,
    msd_flag: Option<Vec<bool>>,
}

impl PStreamSet {
    pub fn create(sss: &SStreamSet, msd_threshold: &Vec<f64>, gv_weight: &Vec<f64>) -> PStreamSet {
        let mut streams = Vec::with_capacity(sss.get_nstream());
        for i in 0..sss.get_nstream() {
            let msd_flag = if sss.is_msd(i) {
                let msd_flag: Vec<_> = (0..sss.get_total_state())
                    .flat_map(|state| {
                        let flag = sss.get_msd(i, state) > msd_threshold[i];
                        vec![flag].repeat(sss.get_duration(state))
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
                    .into_iter()
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
                    .into_iter()
                    .map(|window_index| {
                        let m = sss.get_duration(i) * window_index + vector_index;

                        (0..sss.get_total_state())
                            .into_iter()
                            // iterate over frames
                            .flat_map(|state| (0..sss.get_duration(state)).map(move |j| state))
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
                                        < sss
                                            .get_window_right_width(i, window_index)
                                            .try_into()
                                            .unwrap();

                                let mean = sss.get_mean(i, state, m);
                                let ivar = if !is_msd_boundary || window_index == 0 {
                                    let vari = sss.get_vari(i, state, m);
                                    if vari.abs() > 1e19 {
                                        0.0
                                    } else if vari.abs() < 1e-19 {
                                        1e38
                                    } else {
                                        vari
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
                mtx.calc_wuw_and_wum(&sss, i, parameters);

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

            streams.push(PStream {
                par: pars,
                msd_flag,
            });
        }

        PStreamSet { streams }
    }

    fn msd_boundary_distances(
        total_frame: usize,
        msd_flag: &Option<Vec<bool>>,
    ) -> (Vec<usize>, Vec<usize>) {
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
            result_right[frame] = result_right[frame].min(right - frame);

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
}
