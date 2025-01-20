use super::coefficients::Coefficients;

#[cfg_attr(
    any(target_feature = "avx", target_feature = "neon"),
    path = "fir_simd.rs"
)]
mod fir;
use fir::Df2;

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
    d21: [Df2; N],
    d22: [f64; N],
}

impl<const N: usize> MelLogSpectrumApproximation<N> {
    pub fn new(nmcp: usize) -> Self {
        let pade_start = (N - 1) * N / 2;
        Self {
            ppade: std::array::from_fn(|i| PADE[pade_start + i]),
            d11: [0.0; N],
            d12: [0.0; N],
            d21: std::array::from_fn(|_| Df2::new(nmcp)),
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
            self.d22[i] = self.d21[i - 1].fir(self.d22[i - 1], alpha, coefficients);
            let v = self.d22[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        self.d22[0] = *x;
        *x += out;
    }
}
