use std::{
    iter::Sum,
    ops::{Add, Mul},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MeanVari(pub f64, pub f64);

impl MeanVari {
    pub fn with_ivar(&self) -> Self {
        let Self(mean, vari) = self;
        let ivar = if vari.abs() > 1e19 {
            0.0
        } else if vari.abs() < 1e-19 {
            1e38
        } else {
            1.0 / vari
        };
        Self(*mean, ivar)
    }

    pub fn with_0(&self) -> Self {
        let Self(mean, _) = self;
        Self(*mean, 0.0)
    }

    pub fn weighted(&self, weight: f64) -> Self {
        let Self(mean, vari) = self;
        Self(mean * weight, vari * weight)
    }
}

impl Add for &MeanVari {
    type Output = MeanVari;
    fn add(self, rhs: Self) -> Self::Output {
        MeanVari(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Add for MeanVari {
    type Output = MeanVari;
    #[allow(clippy::op_ref)]
    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl<'a> Sum<&'a Self> for MeanVari {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(MeanVari(0.0, 0.0), |a, b| a + *b)
    }
}

impl Mul<f64> for &MeanVari {
    type Output = MeanVari;
    fn mul(self, rhs: f64) -> Self::Output {
        MeanVari(self.0 * rhs, self.1 * rhs)
    }
}

impl Mul<f64> for MeanVari {
    type Output = MeanVari;
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: f64) -> Self::Output {
        &self * rhs
    }
}
