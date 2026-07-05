//! Smoothes speech parameters by "maximum likelihood parameter generation (MLPG)."
//!
//! For details on MLPG, please refer to <https://doi.org/10.1109/ICASSP.2000.861820>.

use crate::{
    constants::NODATA,
    model::{GvParameter, MeanVari, ModelStream, StreamParameter, Windows},
};

mod mask;
mod mlpg;

use self::{mask::Mask, mlpg::MlpgMatrix};

/// Generate stream parameters.
///
/// Determines unvoiced frames and applies MLPG (maximum likelihood parameter generation) with GV (global variance) to a parameters.
pub struct MlpgAdjust<'a> {
    gv_weight: f64,
    msd_threshold: f64,
    vector_length: usize,
    stream: StreamParameter,
    gv: Option<GvParameter>,
    windows: &'a Windows,
}

impl<'a> MlpgAdjust<'a> {
    /// Create a new [`MlpgAdjust`].
    pub fn new(
        gv_weight: f64,
        msd_threshold: f64,
        ModelStream {
            vector_length,
            stream,
            gv,
            windows,
        }: ModelStream<'a>,
    ) -> Self {
        Self {
            gv_weight,
            msd_threshold,
            vector_length,
            stream,
            gv,
            windows,
        }
    }
    /// Parameter generation using GV weight
    pub fn create(&self, durations: &[usize]) -> Vec<Vec<f64>> {
        let mask = Mask::create(&self.stream, self.msd_threshold, durations);
        let mut pars = vec![vec![0.0; self.vector_length]; mask.len()];

        for vector_index in 0..self.vector_length {
            let parameters = self.create_parameters(vector_index, &mask);
            let mut mtx = MlpgMatrix::calc_wuw_and_wum(self.windows, parameters);
            let par = mtx.par(&self.gv, vector_index, self.gv_weight, &mask);

            for (par, value) in pars.iter_mut().zip(mask.fill(par, NODATA)) {
                par[vector_index] = value;
            }
        }

        pars
    }

    fn create_parameters(&self, vector_index: usize, mask: &Mask) -> Vec<Vec<MeanVari>> {
        self.windows
            .iter()
            .enumerate()
            .map(|(window_index, window)| {
                let m = self.vector_length * window_index + vector_index;

                let mut out = Vec::with_capacity(mask.voiced_len());
                for ((curr_stream, _), (range, voiced_range)) in self.stream.iter().zip(mask) {
                    let Some(voiced_range) = voiced_range else {
                        continue;
                    };

                    let mean_ivar = curr_stream[m].with_ivar();
                    for i in range {
                        if !window.contained_in(&voiced_range, i) && window_index != 0 {
                            out.push(mean_ivar.with_0());
                        } else {
                            out.push(mean_ivar);
                        }
                    }
                }

                out
            })
            .collect()
    }
}
