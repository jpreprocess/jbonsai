use std::sync::Arc;

use crate::{
    label::Label,
    model::{
        interporation_weight::InterporationWeight, stream::ModelParameter, window::Windows,
        ModelSet,
    },
};

pub struct StateStreamSet {
    sstreams: Vec<StateStream>,
    duration: Vec<usize>,
    total_state: usize,
    total_frame: usize,
    ms: Arc<ModelSet>,
}

pub struct StateStream {
    params: Vec<ModelParameter>,
    gv_params: Option<ModelParameter>,
    gv_switch: Vec<bool>,
}

impl StateStreamSet {
    /// Parse label and determine state duration
    pub fn create(
        ms: Arc<ModelSet>,
        label: &Label,
        phoneme_alignment_flag: bool,
        speed: f64,
        iw: &InterporationWeight,
    ) -> Self {
        let duration_params: Vec<(f64, f64)> = (0..label.get_size())
            .flat_map(|i| {
                ms.get_duration(label.get_label(i), iw.get_duration())
                    .parameters
            })
            .collect();

        // determine state duration
        let mut duration = vec![];
        if phoneme_alignment_flag {
            // use duration set by user
            let mut next_time = 0;
            let mut next_state = 0;
            let mut state = 0;
            for i in 0..label.get_size() {
                let end_frame = label.get_end_frame(i);
                if end_frame >= 0.0 {
                    let curr_duration = Self::estimate_duration_with_frame_length(
                        &duration_params[next_state..state + ms.get_nstate()],
                        end_frame - next_time as f64,
                    );
                    next_time += curr_duration.len();
                    next_state = state + ms.get_nstate();
                    duration.extend_from_slice(&curr_duration);
                } else if i + 1 == label.get_size() {
                    eprintln!("HTS_SStreamSet_create: The time of final label is not specified.");
                    Self::estimate_duration(
                        &duration_params[next_state..state + ms.get_nstate()],
                        0.0,
                    );
                }
                state += ms.get_nstate();
            }
        } else {
            // determine frame length
            duration = Self::estimate_duration(&duration_params, 0.0);
            if speed != 1.0 {
                let length: usize = duration.iter().sum();
                duration = Self::estimate_duration_with_frame_length(
                    &duration_params,
                    length as f64 / speed,
                );
            }
        }

        let sstreams: Vec<StateStream> = (0..ms.get_nstream())
            .map(|stream_idx| {
                // get parameter
                let params = (0..label.get_size())
                    .zip(std::iter::repeat(iw))
                    .flat_map(|(label_idx, iw)| {
                        (2..2 + ms.get_nstate())
                            .zip(std::iter::repeat((label_idx, iw)))
                            .map(|(state_idx, (label_idx, iw))| {
                                ms.get_parameter(
                                    stream_idx,
                                    state_idx,
                                    label.get_label(label_idx),
                                    iw.get_parameter(stream_idx),
                                )
                            })
                    })
                    .collect();

                // determine GV
                let gv_switch = (0..label.get_size())
                    .flat_map(|label_idx| {
                        let sw =
                            !ms.use_gv(stream_idx) || ms.get_gv_flag(label.get_label(label_idx));
                        [sw].repeat(ms.get_nstate())
                    })
                    .collect();
                let gv_params = if ms.use_gv(stream_idx) && label.get_size() > 0 {
                    Some(ms.get_gv(stream_idx, label.get_label(0), iw.get_gv(stream_idx)))
                } else {
                    None
                };

                StateStream {
                    params,
                    gv_params,
                    gv_switch,
                }
            })
            .collect();

        Self {
            total_state: label.get_size() * ms.get_nstate(),
            total_frame: duration.iter().sum(),
            duration,
            ms,
            sstreams,
        }
    }

    /// Estimate state duration
    fn estimate_duration(duration_params: &[(f64, f64)], rho: f64) -> Vec<usize> {
        duration_params
            .iter()
            .map(|(mean, vari)| (mean + rho * vari).round().max(1.0) as usize)
            .collect()
    }
    /// Estimate duration from state duration probability distribution and specified frame length
    fn estimate_duration_with_frame_length(
        duration_params: &[(f64, f64)],
        frame_length: f64,
    ) -> Vec<usize> {
        let size = duration_params.len();

        // get the target frame length
        let target_length: usize = frame_length.round().max(1.0) as usize;

        // check the specified duration
        if target_length <= size {
            return vec![1; size];
        }

        // RHO calculation
        let (mean, vari) = duration_params
            .iter()
            .fold((0.0, 0.0), |(mean, vari), curr| {
                (mean + curr.0, vari + curr.1)
            });
        let rho = (target_length as f64 - mean) / vari;

        let mut duration = Self::estimate_duration(duration_params, rho);

        // loop estimation
        let mut sum: usize = duration.iter().sum();
        let calculate_cost =
            |d: usize, (mean, vari): (f64, f64)| (rho - (d as f64 - mean) / vari).abs();
        while target_length != sum {
            // search flexible state and modify its duration
            if target_length > sum {
                let (found_duration, _) = duration
                    .iter_mut()
                    .zip(duration_params.iter())
                    .min_by(|(ad, ap), (bd, bp)| {
                        calculate_cost(**ad + 1, **ap).total_cmp(&calculate_cost(**bd + 1, **bp))
                    })
                    .unwrap();
                *found_duration += 1;
                sum += 1;
            } else {
                let (found_duration, _) = duration
                    .iter_mut()
                    .zip(duration_params.iter())
                    .filter(|(duration, _)| **duration > 1)
                    .min_by(|(ad, ap), (bd, bp)| {
                        calculate_cost(**ad - 1, **ap).total_cmp(&calculate_cost(**bd - 1, **bp))
                    })
                    .unwrap();
                *found_duration -= 1;
                sum -= 1;
            }
        }

        duration
    }

    /// Get number of stream
    pub fn get_nstream(&self) -> usize {
        self.ms.get_nstream()
    }
    /// Get vector length
    pub fn get_vector_length(&self, stream_index: usize) -> usize {
        self.ms.get_vector_length(stream_index)
    }
    /// Get MSD flag
    pub fn is_msd(&self, stream_index: usize) -> bool {
        self.ms.is_msd(stream_index)
    }
    /// Get total number of state
    pub fn get_total_state(&self) -> usize {
        self.total_state
    }
    /// Get total number of frame
    pub fn get_total_frame(&self) -> usize {
        self.total_frame
    }
    /// Get MSD parameter
    pub fn get_msd(&self, stream_index: usize, state_index: usize) -> f64 {
        self.sstreams[stream_index].params[state_index].msd.unwrap()
    }
    /// TODO: remove this
    pub fn get_windows(&self, stream_index: usize) -> &Windows {
        self.ms.get_windows(stream_index)
    }
    /// Get GV flag
    pub fn use_gv(&self, stream_index: usize) -> bool {
        self.sstreams[stream_index].gv_params.is_some()
    }
    pub fn get_durations(&self) -> &[usize] {
        &self.duration
    }
    /// Get state duration
    pub fn get_duration(&self, state_index: usize) -> usize {
        self.duration[state_index]
    }
    /// Get mean parameter
    pub fn get_mean(&self, stream_index: usize, state_index: usize, vector_index: usize) -> f64 {
        self.sstreams[stream_index].params[state_index].parameters[vector_index].0
    }
    /// Get variance parameter
    pub fn get_vari(&self, stream_index: usize, state_index: usize, vector_index: usize) -> f64 {
        self.sstreams[stream_index].params[state_index].parameters[vector_index].1
    }
    /// Get GV mean parameter
    pub fn get_gv_mean(&self, stream_index: usize, vector_index: usize) -> f64 {
        self.sstreams[stream_index]
            .gv_params
            .as_ref()
            .unwrap()
            .parameters[vector_index]
            .0
    }
    /// Get GV variance parameter
    pub fn get_gv_vari(&self, stream_index: usize, vector_index: usize) -> f64 {
        self.sstreams[stream_index]
            .gv_params
            .as_ref()
            .unwrap()
            .parameters[vector_index]
            .1
    }
    /// Get GV switch
    pub fn get_gv_switch(&self, stream_index: usize, state_index: usize) -> bool {
        self.sstreams[stream_index].gv_switch[state_index]
    }

    /// Set mean parameter
    pub fn set_mean(
        &mut self,
        stream_index: usize,
        state_index: usize,
        vector_index: usize,
        value: f64,
    ) {
        self.sstreams[stream_index].params[state_index].parameters[vector_index].0 = value;
    }
}
