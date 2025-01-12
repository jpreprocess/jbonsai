use std::ops::IndexMut;

use super::coefficients::Coefficients;

const PADE: [f64; 21] = [
    1.00000000000f64,
    1.00000000000f64,
    0.00000000000f64,
    1.00000000000f64,
    0.00000000000f64,
    0.00000000000f64,
    1.00000000000f64,
    0.00000000000f64,
    0.00000000000f64,
    0.00000000000f64,
    1.00000000000f64,
    0.49992730000f64,
    0.10670050000f64,
    0.01170221000f64,
    0.00056562790f64,
    1.00000000000f64,
    0.49993910000f64,
    0.11070980000f64,
    0.01369984000f64,
    0.00095648530f64,
    0.00003041721f64,
];
const PADE_OFFSET: [usize; 6] = [0, 1, 3, 6, 10, 15];

/// N == pd + 1
#[derive(Debug, Clone)]
pub struct MelLogSpectrumApproximation<const N: usize> {
    d1: [[f64; 2]; N],
    d2: [Vec<f64>; N],
}

impl<const N: usize> MelLogSpectrumApproximation<N> {
    pub fn new(nmcp: usize) -> Self {
        Self {
            d1: [[0.0; 2]; N],
            d2: std::array::from_fn(|_| vec![0.0; nmcp]),
        }
    }

    #[inline(always)]
    pub fn df(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        self.df1(x, alpha, coefficients);
        self.df2(x, alpha, coefficients);
    }

    #[inline(always)]
    fn df1(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        for i in (1..N).rev() {
            Self::df1_step1(&mut self.d1[i - 1], alpha);
            self.d1[i][0] = Self::df1_step2(&self.d1[i - 1], coefficients);
        }
        self.d1[0][0] = Self::df_apply(&self.d1, x);
    }

    #[inline(always)]
    fn df1_step1(d: &mut [f64], alpha: f64) {
        let aa = 1.0 - alpha * alpha;
        d[1] = alpha * d[1] + aa * d[0];
    }

    #[inline(always)]
    fn df1_step2(d: &[f64], coefficients: &'_ Coefficients) -> f64 {
        d[1] * coefficients[1]
    }

    // Code optimization was done in
    // [#12](https://github.com/jpreprocess/jbonsai/pull/12)
    // [#16](https://github.com/jpreprocess/jbonsai/pull/16)
    // [#57](https://github.com/jpreprocess/jbonsai/pull/57)
    #[inline(always)]
    fn df2(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        for i in (1..N).rev() {
            Self::df2_step1(&mut self.d2[i - 1], alpha);
            self.d2[i][0] = Self::df2_step2(&self.d2[i - 1], coefficients);
        }
        self.d2[0][0] = Self::df_apply(&self.d2, x);
    }

    #[inline(always)]
    fn df2_step1(d: &mut [f64], alpha: f64) {
        let mut prev = alpha * d[0];
        for i in 0..d.len() - 1 {
            d[i] += alpha * (d[i + 1] - prev);
            (d[i], prev) = (prev, d[i]);
        }
        d[d.len() - 1] = prev;
    }

    #[inline(always)]
    fn df2_step2(d: &[f64], coefficients: &'_ Coefficients) -> f64 {
        let mut y = 0.0;
        for i in 2..d.len() {
            y += d[i] * coefficients[i];
        }
        y
    }

    #[inline(always)]
    fn df_apply(d: &[impl IndexMut<usize, Output = f64>; N], x: &mut f64) -> f64 {
        let mut d00 = *x;
        for i in (1..N).rev() {
            let v = d[i][0] * PADE[PADE_OFFSET[N - 1] + i];
            if i & 1 != 0 {
                d00 += v;
                *x += v + v;
            } else {
                d00 -= v;
            }
        }
        d00
    }
}
