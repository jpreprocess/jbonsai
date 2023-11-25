use crate::{
    label::Label,
    model::{model::ModelParameter, ModelSet},
};

#[derive(Debug, Default, Clone)]
pub struct ModelParameterSequence {
    pub parameters: Vec<(f64, f64)>,
    pub msd: Option<Vec<f64>>,
}

impl ModelParameterSequence {
    pub fn size(&self) -> usize {
        self.parameters.len()
    }
    pub fn get_rho(&self, target_length: f64) -> f64 {
        let (mean, vari) = self.parameters.iter().fold((0., 0.), |(mean, vari), curr| {
            (mean + curr.0, vari + curr.1)
        });
        (target_length - mean) / vari
    }
}

impl FromIterator<ModelParameter> for ModelParameterSequence {
    fn from_iter<T: IntoIterator<Item = ModelParameter>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let Some(ModelParameter {
            parameters: first_parameters,
            msd: first_msd,
        }) = iter.next()
        else {
            return Self::default();
        };
        let is_msd = first_msd.is_some();

        let mut result = Self {
            parameters: first_parameters,
            msd: if is_msd {
                Some(vec![first_msd.unwrap()])
            } else {
                None
            },
        };

        for elem in iter {
            result.parameters.extend_from_slice(&elem.parameters);
            if is_msd {
                let msd_value = elem.msd.unwrap();
                let msd = result.msd.as_mut().unwrap();
                msd.extend(vec![msd_value].repeat(elem.parameters.len()));
            }
        }

        result
    }
}

pub struct SStreamSet<'a> {
    sstreams: Vec<SStream>,
    // nstate: usize,
    duration: Vec<usize>,
    total_state: usize,
    total_frame: usize,
    ms: &'a ModelSet,
}

pub struct SStream {
    // vector_length: usize,
    params: ModelParameterSequence,
    // win_coef: Vec<Vec<f32>>,
    gv_params: Option<ModelParameter>,
    // gv_switch: bool,
}

impl<'a> SStreamSet<'a> {
    pub fn create(
        ms: &'a ModelSet,
        label: Label,
        phoneme_alignment_flag: bool,
        speed: f64,
        duration_iw: &[f64],
        parameter_iw: &Vec<Vec<f64>>,
        gv_iw: &Vec<Vec<f64>>,
    ) -> Option<Self> {
        // check interpolation weights
        let duration_iw_sum: f64 = duration_iw.iter().sum();
        if (duration_iw_sum - 1.0).abs() > f64::EPSILON {
            return None;
        }

        let duration_params: Vec<(f64, f64)> = (0..label.get_size())
            .flat_map(|i| {
                ms.get_duration(label.get_string(i), &duration_iw)
                    .parameters
            })
            .collect();

        let mut duration = vec![];
        if phoneme_alignment_flag {
            // use duration set by user
            let mut next_time = 0;
            let mut next_state = 0;
            let mut state = 0;
            for i in 0..label.get_size() {
                let end_frame = label.get_end_frame(i);
                if end_frame >= 0. {
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
                        0.,
                    );
                }
                state += ms.get_nstate();
            }
        } else {
            // determine frame length
            duration = Self::estimate_duration(&duration_params, 0.);
            if speed != 1.0 {
                let length: usize = duration.iter().sum();
                duration = Self::estimate_duration_with_frame_length(
                    &duration_params,
                    length as f64 / speed,
                );
            }
        }

        let sstreams: Vec<SStream> = (0..ms.get_nstream())
            .map(|stream_idx| {
                let params = (0..label.get_size())
                    .flat_map(|label_idx| {
                        (0..ms.get_nstate()).zip(std::iter::repeat(label_idx)).map(
                            |(state_idx, label_idx)| {
                                ms.get_parameter(
                                    stream_idx,
                                    state_idx,
                                    label.get_string(label_idx),
                                    parameter_iw,
                                )
                            },
                        )
                    })
                    .collect();
                let gv_params = if ms.use_gv(stream_idx) {
                    Some(ms.get_gv(stream_idx, label.get_string(0), gv_iw))
                } else {
                    None
                };
                SStream { params, gv_params }
            })
            .collect();

        Some(Self {
            total_state: label.get_size() * ms.get_nstate(),
            total_frame: duration.iter().sum(),
            duration,
            ms,
            sstreams,
        })
    }

    fn estimate_duration(duration_params: &[(f64, f64)], rho: f64) -> Vec<usize> {
        duration_params
            .iter()
            .map(|(mean, vari)| (mean + rho * vari).round().min(1.0) as usize)
            .collect()
    }
    fn estimate_duration_with_frame_length(
        duration_params: &[(f64, f64)],
        frame_length: f64,
    ) -> Vec<usize> {
        let size = duration_params.len();

        // get the target frame length
        let target_length: usize = frame_length.round().min(1.0) as usize;

        // check the specified duration
        if target_length <= size {
            return vec![1; size];
        }

        // RHO calculation
        let (mean, vari) = duration_params.iter().fold((0., 0.), |(mean, vari), curr| {
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

    pub fn get_nstream(&self) -> u64 {
        self.ms.get_nstream() as u64
    }
    pub fn get_vector_length(&self, stream_index: u64) -> u64 {
        self.ms.get_vector_length(stream_index as usize) as u64
    }
    pub fn is_msd(&self, stream_index: u64) -> i8 {
        self.ms.is_msd(stream_index as usize) as i8
    }
    pub fn get_total_state(&self) -> u64 {
        self.total_state as u64
    }
    pub fn get_total_frame(&self) -> u64 {
        self.total_frame as u64
    }
    pub fn get_msd(&self, stream_index: u64, state_index: u64) -> f64 {
        self.sstreams[stream_index as usize]
            .params
            .msd
            .as_ref()
            .unwrap()[state_index as usize]
    }
    pub fn get_window_size(&self, stream_index: u64) -> u64 {
        self.ms.get_window_size(stream_index as usize) as u64
    }
    pub fn get_window_left_width(&self, stream_index: u64, window_index: u64) -> i32 {
        self.ms
            .get_window_left_width(stream_index as usize, window_index as usize) as i32
    }
    pub fn get_window_right_width(&self, stream_index: u64, window_index: u64) -> i32 {
        self.ms
            .get_window_right_width(stream_index as usize, window_index as usize) as i32
    }
    pub fn get_window_coefficient(
        &self,
        stream_index: u64,
        window_index: u64,
        coefficient_index: i32,
    ) -> f64 {
        self.ms.get_window_coefficient(
            stream_index as usize,
            window_index as usize,
            coefficient_index as usize,
        )
    }
    pub fn get_window_max_width(&self, stream_index: u64) -> u64 {
        self.ms.get_window_max_width(stream_index as usize) as u64
    }
    pub fn use_gv(&self, stream_index: u64) -> i8 {
        self.sstreams[stream_index as usize].gv_params.is_some() as i8
    }
    pub fn get_duration(&self, state_index: u64) -> u64 {
        self.duration[state_index as usize] as u64
    }
    pub fn get_mean(&self, stream_index: u64, state_index: u64, vector_index: u64) -> f64 {
        self.sstreams[stream_index as usize].params.parameters
            [self.ms.get_nstate() * (state_index as usize) + vector_index as usize]
            .0
    }
    pub fn get_vari(&self, stream_index: u64, state_index: u64, vector_index: u64) -> f64 {
        self.sstreams[stream_index as usize].params.parameters
            [self.ms.get_nstate() * (state_index as usize) + vector_index as usize]
            .1
    }
    pub fn get_gv_mean(&self, stream_index: u64, vector_index: u64) -> f64 {
        self.sstreams[stream_index as usize]
            .gv_params
            .as_ref()
            .unwrap()
            .parameters[vector_index as usize]
            .0
    }
    pub fn get_gv_vari(&self, stream_index: u64, vector_index: u64) -> f64 {
        self.sstreams[stream_index as usize]
            .gv_params
            .as_ref()
            .unwrap()
            .parameters[vector_index as usize]
            .1
    }
    pub fn get_gv_switch(&self, _stream_index: u64, _state_index: u64) -> i8 {
        0
    }
}
