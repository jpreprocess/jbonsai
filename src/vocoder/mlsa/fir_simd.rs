use std::{
    ops::{Deref, DerefMut},
    simd::{LaneCount, Simd, StdFloat, SupportedLaneCount, num::SimdFloat},
};

#[derive(Debug, Clone)]
pub struct SimdVec<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    chunks: Vec<Simd<f64, N>>,
    len: usize,
}

impl<const N: usize> SimdVec<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new(len: usize) -> Self {
        Self {
            chunks: vec![Simd::splat(0.0); len.div_ceil(N)],
            len,
        }
    }
}

impl<const N: usize> Deref for SimdVec<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    type Target = [f64];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        let data = self.chunks.as_ptr() as *const f64;
        unsafe { std::slice::from_raw_parts(data, self.len) }
    }
}

impl<const N: usize> DerefMut for SimdVec<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let data = self.chunks.as_mut_ptr() as *mut f64;
        unsafe { std::slice::from_raw_parts_mut(data, self.len) }
    }
}

#[derive(Debug, Clone)]
pub struct AlphaMatrix<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    dd: [Simd<f64, N>; N],
    dr: Simd<f64, N>,
    rd: Simd<f64, N>,
    rr: f64,
}

// d[0] =                                        a d[0] +    1 rem(0)
// d[1] =                             iaa d[0] + a d[1] +   -a rem(0)
// d[2] =               -a iaa d[0] + iaa d[1] + a d[2] +  a^2 rem(0)
// d[3] = aa iaa d[0] + -a iaa d[1] + iaa d[2] + a d[3] + -a^3 rem(0)
//
// rem(4) = -a^3 iaa d[0] + a^2 iaa d[1] - a d[2] + d[3] + a^4 rem(0)
impl<const N: usize> AlphaMatrix<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new(alpha: f64) -> Self {
        let axn = Simd::splat(alpha);
        let iaaxn = Simd::splat(1.0 - alpha * alpha);

        let mut dd = iaaxn;
        let zero = [0.0; N];
        let dd = std::array::from_fn(|i| {
            if i == 0 {
                axn
            } else {
                let ret = Simd::load_or(&zero[..i], dd);
                dd *= -axn;
                ret
            }
        });

        let dr = Simd::from_array(std::array::from_fn(|i| (-alpha).powi(i as i32)));
        let rd = dr.reverse() * iaaxn;
        let rr = (-alpha).powi(N as i32);

        Self { dd, dr, rd, rr }
    }

    #[inline(always)]
    fn mul(&self, (mut d, r): (Simd<f64, N>, f64)) -> (Simd<f64, N>, f64) {
        let mut ret = self.dr * Simd::splat(r);
        for dd in self.dd {
            ret = dd.mul_add(d, ret);
            d = d.rotate_elements_right::<1>();
        }
        (ret, (self.rd * d).reduce_sum() + self.rr * r)
    }
}

#[derive(Debug, Clone)]
pub(super) struct Df2(SimdVec<4>);

impl Df2 {
    #[inline(always)]
    pub(super) fn new(len: usize) -> Self {
        Self(SimdVec::new(len))
    }

    #[inline(always)]
    pub(super) fn fir(&mut self, x: f64, alpha: f64, coefficients: &[f64]) -> f64 {
        let d = &mut self.0.chunks[..];

        d[0][0] = x;

        let matrix = AlphaMatrix::new(alpha);
        let mut rem = 0.0;
        for d in &mut d[..] {
            (*d, rem) = matrix.mul((*d, rem));
        }

        let (c, last) = coefficients.as_chunks();
        assert!(c.len() < d.len());
        let mut y = Simd::load_or(&[0.0; 2], d[0]) * Simd::from_array(c[0]);
        for i in 1..c.len() {
            y = d[i].mul_add(Simd::from_array(c[i]), y);
        }
        y = d[c.len()].mul_add(Simd::load_or_default(last), y);
        y.reduce_sum()
    }
}
