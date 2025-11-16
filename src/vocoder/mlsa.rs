use super::coefficients::Coefficients;

/// N == pd + 1
#[derive(Debug, Clone)]
pub struct MelLogSpectrumApproximation<const N: usize> {
    d11: [f64; N],
    d12: [f64; N],
    d21: [Vec<f64>; N],
    d22: [f64; N],
}

pub trait Pade<const N: usize> {
    const PPADE: [f64; N];
}
macro_rules! impl_pade {
    ($($i:literal: $ppade:expr),* $(,)?) => {
        $(
            impl Pade<$i> for MelLogSpectrumApproximation<$i> {
                const PPADE: [f64; $i] = $ppade;
            }
        )*
    };
}

impl_pade!(
    1: [1.00000000000f64],
    2: [1.00000000000f64, 0.00000000000f64],
    3: [1.00000000000f64, 0.00000000000f64, 0.00000000000f64],
    4: [1.00000000000f64, 0.00000000000f64, 0.00000000000f64, 0.00000000000f64],
    5: [1.00000000000f64, 0.49992730000f64, 0.10670050000f64, 0.01170221000f64, 0.00056562790f64],
    6: [1.00000000000f64, 0.49993910000f64, 0.11070980000f64, 0.01369984000f64, 0.00095648530f64, 0.00003041721f64],
);

impl<const N: usize> MelLogSpectrumApproximation<N>
where
    Self: Pade<N>,
{
    pub fn new(nmcp: usize) -> Self {
        Self {
            d11: [0.0; N],
            d12: [0.0; N],
            d21: std::array::from_fn(|_| vec![0.0; nmcp]),
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
            let v = self.d12[i] * Self::PPADE[i];
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
            self.d22[i] = fir(&mut self.d21[i - 1], self.d22[i - 1], alpha, coefficients);
            let v = self.d22[i] * Self::PPADE[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        self.d22[0] = *x;
        *x += out;
    }
}

#[cfg(any(
    all(target_arch = "x86_64", target_feature = "fma"),
    all(target_arch = "aarch64", target_feature = "neon"),
))]
macro_rules! mul_add {
    ($s:ident * ($($a:tt)+) + ($($b:tt)+)) => {
        f64::mul_add($s, mul_add!($($a)+), mul_add!($($b)+))
    };
    ($s:ident * ($($a:tt)+) + $b:expr) => {
        f64::mul_add($s, mul_add!($($a)+), $b)
    };
    ($s:ident * $a:ident $([$ai:literal])? + ($($b:tt)+)) => {
        f64::mul_add($s, $a $([$ai])?, mul_add!($($b)+))
    };
    ($s:ident * $a:ident $([$ai:literal])? + $e:expr) => {
        f64::mul_add($s, $a $([$ai])?, $e)
    };
    (-$s:ident * ($($a:tt)+) + ($($b:tt)+)) => {
        f64::mul_add(-$s, mul_add!($($a)+), mul_add!($($b)+))
    };
    (-$s:ident * ($($a:tt)+) + $b:expr) => {
        f64::mul_add(-$s, mul_add!($($a)+), $b)
    };
    (-$s:ident * $a:ident $([$ai:literal])? + ($($b:tt)+)) => {
        f64::mul_add(-$s, $a $([$ai])?, mul_add!($($b)+))
    };
    (-$s:ident * $a:ident $([$ai:literal])? + $e:expr) => {
        f64::mul_add(-$s, $a $([$ai])?, $e)
    };

    ($e:expr) => {
        $e
    };
}

#[cfg_attr(test, inline(never))] // cargo-show-asm passes `--test`
fn fir(d: &mut [f64], x: f64, alpha: f64, coefficients: &[f64]) -> f64 {
    assert!(2 <= d.len());

    let a = alpha;
    let aa = a * a;
    let aaaa = aa * aa;
    let iaa = 1.0 - aa;

    let mut rem = -a * x + d[1];

    d[0] = a * x;
    d[1] = iaa * x + a * d[1];

    let mut y = [0.0; 2];
    let mut c = coefficients[2..d.len()].chunks_exact(4);
    let mut d = d[2..].chunks_exact_mut(4);

    use std::iter::zip;
    for (c, d) in zip(&mut c, &mut d) {
        (d[0], d[1], d[2], d[3], rem) = (
            mul_add!(iaa * rem + a * d[0]),
            mul_add!(iaa * (-a * rem + d[0]) + a * d[1]),
            mul_add!(iaa * (aa * rem + (-a * d[0] + d[1])) + a * d[2]),
            mul_add!(iaa * (aa * (-a * rem + d[0]) + (-a * d[1] + d[2])) + a * d[3]),
            mul_add!(aaaa * rem + (aa * (-a * d[0] + d[1]) + (-a * d[2] + d[3]))),
        );
        y[0] += c[0] * d[0] + c[2] * d[2];
        y[1] += c[1] * d[1] + c[3] * d[3];
    }
    for (c, d) in zip(c.remainder(), d.into_remainder()) {
        (*d, rem) = (iaa * rem + a * *d, -a * rem + *d);
        y[0] += c * *d;
    }

    y[0] + y[1]
}
