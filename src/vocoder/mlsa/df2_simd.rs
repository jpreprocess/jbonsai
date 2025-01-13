use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    simd::{num::SimdFloat, LaneCount, Simd, StdFloat, SupportedLaneCount},
    slice::SliceIndex,
};

#[derive(Debug, Clone)]
pub struct D2<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    chunks: Vec<Simd<f64, N>>,
    len: usize,
}

impl<const N: usize> D2<N>
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

impl<const N: usize> Deref for D2<N>
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

impl<const N: usize> DerefMut for D2<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let data = self.chunks.as_mut_ptr() as *mut f64;
        unsafe { std::slice::from_raw_parts_mut(data, self.len) }
    }
}

impl<const N: usize, I: SliceIndex<[f64]>> Index<I> for D2<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        self.deref().index(index)
    }
}

impl<const N: usize, I: SliceIndex<[f64]>> IndexMut<I> for D2<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.deref_mut().index_mut(index)
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

#[inline(always)]
pub fn df2_step1<const N: usize>(d: &mut D2<N>, matrix: &AlphaMatrix<N>)
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut rem = 0.0;
    for d in &mut d.chunks {
        (*d, rem) = matrix.mul((*d, rem));
    }
}

const SKIP: usize = 2;

#[inline(always)]
#[allow(clippy::needless_range_loop)]
pub fn df2_step2<const N: usize>(d: &D2<N>, coefficients: &[f64]) -> f64
where
    LaneCount<N>: SupportedLaneCount,
{
    let (chunks, last) = coefficients.as_chunks();
    assert!(SKIP.div_ceil(N) <= chunks.len());
    assert!(chunks.len() < d.chunks.len());

    let mut y = if N > SKIP {
        Simd::load_or(&[0.0; SKIP], d.chunks[0]) * Simd::from_array(chunks[0])
    } else {
        Simd::splat(0.0)
    };

    for i in SKIP.div_ceil(N)..chunks.len() {
        y = d.chunks[i].mul_add(Simd::from_array(chunks[i]), y);
    }
    y = d.chunks[chunks.len()].mul_add(Simd::load_or_default(last), y);

    y.reduce_sum()
}
