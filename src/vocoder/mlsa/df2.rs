use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

#[derive(Debug, Clone)]
pub struct D2 {
    data: Vec<f64>,
}

impl D2 {
    pub fn new(len: usize) -> Self {
        Self {
            data: vec![0.0; len],
        }
    }
}

impl std::ops::Deref for D2 {
    type Target = [f64];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for D2 {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<I: SliceIndex<[f64]>> Index<I> for D2 {
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        self.data.index(index)
    }
}

impl<I: SliceIndex<[f64]>> IndexMut<I> for D2 {
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

pub struct AlphaMatrix {
    dd: f64,
    // dr: f64,
    rd: f64,
    rr: f64,
}

impl AlphaMatrix {
    pub fn new(alpha: f64) -> Self {
        Self {
            dd: alpha,
            // dr: 1.0,
            rd: 1.0 - alpha * alpha,
            rr: -alpha,
        }
    }
}

// calculate d[t] from d[t-1] such that
// - d[t][0] = alpha d[t-1][0]
// - d[t][i] = d[t-1][i-1] + alpha (d[t-1][i] - dr[t][i-1])
//
// using rem(i) := d[t-1][i-1] - alpha d[t][i-1], we have
// - d[t][i]  = alpha d[t-1][i] + rem(i)
// - rem(1)   = d[t-1][0] - alpha d[t][0]
//            = (1 - alpha^2) d[t-1][0]
// - rem(i+1) = d[t-1][i] - alpha d[t][i]
//            = (1 - alpha^2) d[t-1][i] - alpha rem(i)
//
// needless_range_loop for better understanding along with the explanation
#[inline(always)]
#[allow(clippy::needless_range_loop)]
pub fn df2_step1(d: &mut [f64], matrix: &AlphaMatrix) {
    // skip d[t][0] as it is never used
    // `rem` value used in loop of i = rem(i)
    let mut rem = 0.0; // rem(0)
    for i in 0..d.len() {
        // calculate d[t][i] and rem(i+1)
        (d[i], rem) = (
            matrix.dd * d[i] + /* matrix.dr * */ rem,
            matrix.rd * d[i] + matrix.rr * rem,
        );
    }
}

#[inline(always)]
pub fn df2_step2(d: &[f64], coefficients: &[f64]) -> f64 {
    let mut y = 0.0;
    for i in 2..d.len() {
        y += d[i] * coefficients[i];
    }
    y
}
