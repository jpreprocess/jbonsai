use std::iter;

use crate::{
    constants::NODATA,
    model::{GvParameter, MeanVari, ModelStream, StreamParameter, Windows},
};

mod mask;
mod mlpg;

use self::{mask::Mask, mlpg::MlpgMatrix};

pub struct MlpgAdjust<'a> {
    gv_weight: f64,
    msd_threshold: f64,
    vector_length: usize,
    stream: StreamParameter,
    gv: Option<GvParameter>,
    windows: &'a Windows,
}

impl<'a> MlpgAdjust<'a> {
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
        let msd_flag = Mask::create(&self.stream, self.msd_threshold, durations);
        let msd_boundaries = msd_flag.boundary_distances();
        let mut pars = vec![vec![0.0; self.vector_length]; msd_flag.mask().len()];

        for vector_index in 0..self.vector_length {
            let parameters: Vec<Vec<MeanVari>> = self
                .windows
                .iter()
                .enumerate()
                .map(|(window_index, window)| {
                    let m = self.vector_length * window_index + vector_index;

                    self.stream
                        .iter()
                        .map(|(curr_stream, _)| curr_stream[m].with_ivar())
                        .duration(durations)
                        .zip(&msd_boundaries)
                        .map(|(mean_ivar, (left, right))| {
                            let is_left_msd_boundary = *left < window.left_width();
                            let is_right_msd_boundary = *right < window.right_width();

                            // If the window includes non-msd frames, set the ivar to 0.0
                            if (is_left_msd_boundary || is_right_msd_boundary) && window_index != 0
                            {
                                mean_ivar.with_0()
                            } else {
                                mean_ivar
                            }
                        })
                        .filter_by(msd_flag.mask())
                        .collect()
                })
                .collect();

            let mut mtx = MlpgMatrix::calc_wuw_and_wum(self.windows, parameters);
            let par = mtx.par(&self.gv, vector_index, self.gv_weight, durations, &msd_flag);

            for (par, value) in pars.iter_mut().zip(msd_flag.fill(par, NODATA)) {
                par[vector_index] = value;
            }
        }

        pars
    }
}

trait IterExt: Iterator {
    fn duration<'a>(
        self,
        durations: impl IntoIterator<Item = &'a usize> + 'a,
    ) -> impl Iterator<Item = Self::Item>;

    fn filter_by<'a>(
        self,
        mask: impl IntoIterator<Item = &'a bool> + 'a,
    ) -> impl Iterator<Item = Self::Item>;
}

impl<T: Copy + 'static, I: Iterator<Item = T>> IterExt for I {
    fn duration<'a>(
        self,
        durations: impl IntoIterator<Item = &'a usize> + 'a,
    ) -> impl Iterator<Item = Self::Item> {
        self.zip(durations)
            .flat_map(move |(item, duration)| iter::repeat(item).take(*duration))
    }

    fn filter_by<'a>(
        self,
        mask: impl IntoIterator<Item = &'a bool> + 'a,
    ) -> impl Iterator<Item = Self::Item> {
        self.zip(mask)
            .filter_map(|(item, mask)| if *mask { Some(item) } else { None })
    }
}
