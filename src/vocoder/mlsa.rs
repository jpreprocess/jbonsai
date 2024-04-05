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

/// N == pd + 1
#[derive(Debug, Clone)]
pub struct MelLogSpectrumApproximation<const N: usize> {
    ppade: [f64; N],
    d11: [f64; N],
    d12: [f64; N],
    d21: [Vec<f64>; N],
    d22: [f64; N],
}

impl<const N: usize> MelLogSpectrumApproximation<N> {
    pub fn new(c_len: usize) -> Self {
        let pade_start = (N - 1) * N / 2;
        Self {
            ppade: std::array::from_fn(|i| PADE[pade_start + i]),
            d11: [0.0; N],
            d12: [0.0; N],
            d21: std::array::from_fn(|_| vec![0.0; c_len + 1]),
            d22: [0.0; N],
        }
    }

    #[inline(always)]
    pub fn df(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        self.df1(x, alpha, coefficients);
        self.df2(x, alpha, coefficients);
    }

    #[inline(always)]
    fn df1(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        let aa = 1.0 - alpha * alpha;
        let mut out = 0.0;
        for i in (1..N).rev() {
            self.d11[i] = aa * self.d12[i - 1] + alpha * self.d11[i];
            self.d12[i] = self.d11[i] * coefficients[1];
            let v = self.d12[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        self.d12[0] = *x;
        *x += out;
    }

    #[inline(always)]
    fn df2(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        let mut out = 0.0;
        for i in (1..N).rev() {
            self.d22[i] = Self::fir(&mut self.d21[i - 1], self.d22[i - 1], alpha, coefficients);
            let v = self.d22[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        self.d22[0] = *x;
        *x += out;
    }

    // Code optimization was done in
    // [#12](https://github.com/jpreprocess/jbonsai/pull/12)
    // [#16](https://github.com/jpreprocess/jbonsai/pull/16)
    #[inline(always)]
    fn fir(d: &mut [f64], x: f64, alpha: f64, coefficients: &'_ Coefficients) -> f64 {
        // This ensures the unsafe code will not cause undefined behavior
        assert_eq!(d.len(), coefficients.len() + 1);

        let aa = 1.0 - alpha * alpha;
        d[0] = x;
        d[1] = aa * d[0] + alpha * d[1];
        let mut y = 0.0;
        let mut prev = d[1];
        for i in 2..coefficients.len() {
            unsafe {
                let di = d.get_unchecked(i) + alpha * (d.get_unchecked(i + 1) - prev);
                y += di * coefficients.buffer.get_unchecked(i);
                *d.get_unchecked_mut(i) = std::mem::replace(&mut prev, di);
            }
        }
        d[coefficients.len()] = prev;

        y
    }
}
