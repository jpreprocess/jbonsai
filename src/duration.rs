use crate::model::Models;

pub struct DurationEstimator;

impl DurationEstimator {
    pub fn create(&self, models: &Models, speed: f64) -> Vec<usize> {
        let duration_params = models.duration();

        // determine frame length
        let mut duration = Self::estimate_duration(&duration_params, 0.0);
        if speed != 1.0 {
            let length: usize = duration.iter().sum();
            duration =
                Self::estimate_duration_with_frame_length(&duration_params, length as f64 / speed);
        }

        duration
    }

    pub fn create_with_alignment(&self, models: &Models, times: &[(f64, f64)]) -> Vec<usize> {
        let duration_params = models.duration();

        // determine state duration
        let mut duration = vec![];
        // use duration set by user
        let mut frame_count = 0;
        let mut next_state = 0;
        let mut state = 0;
        for (i, (_start_frame, end_frame)) in times.iter().enumerate() {
            if *end_frame >= 0.0 {
                let curr_duration = Self::estimate_duration_with_frame_length(
                    &duration_params[next_state..state + models.nstate()],
                    end_frame - frame_count as f64,
                );
                frame_count += curr_duration.iter().sum::<usize>();
                next_state = state + models.nstate();
                duration.extend_from_slice(&curr_duration);
            } else if i + 1 == times.len() {
                eprintln!("HTS_SStreamSet_create: The time of final label is not specified.");
                Self::estimate_duration(&duration_params[next_state..state + models.nstate()], 0.0);
            }
            state += models.nstate();
        }

        duration
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
}

#[cfg(test)]
mod tests {
    use crate::model::tests::load_models;

    use super::DurationEstimator;

    #[test]
    fn without_alignment() {
        let models = load_models();
        assert_eq!(
            DurationEstimator.create(&models, 1.0),
            [
                8, 17, 14, 25, 15, 3, 4, 2, 2, 2, 2, 3, 3, 3, 3, 4, 3, 2, 2, 2, 3, 3, 6, 3, 2, 3,
                3, 3, 3, 2, 2, 1, 3, 2, 14, 22, 14, 26, 38, 5
            ]
        );
        assert_eq!(
            DurationEstimator.create(&models, 1.2),
            [
                6, 12, 11, 19, 14, 3, 4, 2, 2, 2, 2, 3, 3, 3, 3, 4, 3, 2, 2, 2, 3, 3, 6, 3, 2, 3,
                3, 3, 3, 2, 2, 1, 3, 2, 14, 18, 11, 16, 27, 4
            ]
        );
    }

    #[test]
    fn with_alignment() {
        let models = load_models();
        assert_eq!(
            DurationEstimator.create_with_alignment(
                &models,
                &[
                    (0.0, 298.5),
                    (298.5, 334.5),
                    (334.5, 350.5),
                    (350.5, 362.5),
                    (362.5, 394.5),
                    (394.5, 416.5),
                    (416.5, 454.5),
                    (454.5, 606.5)
                ]
            ),
            [
                36, 86, 48, 102, 27, 7, 11, 6, 6, 6, 2, 4, 3, 4, 3, 3, 3, 2, 2, 2, 3, 6, 14, 6, 3,
                4, 5, 6, 4, 3, 3, 1, 4, 4, 26, 28, 19, 42, 55, 8
            ]
        );
    }
}
