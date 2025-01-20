use super::MelLogSpectrumApproximation;

#[derive(Debug, Clone)]
pub(super) struct Df2(Vec<f64>);

impl Df2 {
    pub(super) fn new(len: usize) -> Self {
        Self(vec![0.0; len])
    }

    // Code optimization was done in
    // [#60](https://github.com/jpreprocess/jbonsai/pull/60)
    #[inline(always)]
    pub(super) fn fir(&mut self, x: f64, alpha: f64, coefficients: &[f64]) -> f64 {
        let d = &mut self.0[..];

        d[0] = x;

        let iaa = 1.0 - alpha * alpha;
        let mut rem = 0.0;
        for di in &mut d[..] {
            (*di, rem) = (alpha * *di + rem, iaa * *di - alpha * rem);
        }

        let mut y = 0.0;
        for i in 2..d.len() {
            y += d[i] * coefficients[i];
        }
        y
    }
}
