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
    // sstreams: Vec<SStream>,
    // nstate: usize,
    duration: Vec<usize>,
    // total_state: usize,
    // total_frame: usize,
    ms: &'a ModelSet,
}

// pub struct SStream {
//     vector_length: usize,
//     params: ModelParameter,
//     win_coef: Vec<Vec<f32>>,
//     gv_params: Option<ModelParameter>,
//     gv_switch: bool,
// }

impl<'a> SStreamSet<'a> {
    pub fn create(
        ms: &'a ModelSet,
        label: Label,
        phoneme_alignment_flag: bool,
        speed: f64,
        duration_iw: &[f64],
        parameter_iw: &[f64],
        gv_iw: &[f64],
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
            duration = Self::estimate_duration(&duration_params, 0.);
            if speed != 1.0 {
                let length: usize = duration.iter().sum();
                duration = Self::estimate_duration_with_frame_length(
                    &duration_params,
                    length as f64 / speed,
                );
            }
        }

        Some(Self { duration, ms })
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
}
