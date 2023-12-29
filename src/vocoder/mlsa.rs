use super::{buffer::Buffer, coefficients::Coefficients};

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

#[derive(Debug, Clone)]
pub struct MelLogSpectrumApproximation {
    pd: usize,
    ppade: &'static [f64],
    d11: Vec<f64>,
    d12: Vec<f64>,
    d21: Vec<Vec<f64>>,
    d22: Vec<f64>,
}

impl MelLogSpectrumApproximation {
    pub fn new(pd: usize, c_len: usize) -> Self {
        Self {
            pd,
            ppade: &PADE[(pd * (pd + 1) / 2)..],
            d11: vec![0.0; pd + 1],
            d12: vec![0.0; pd + 1],
            d21: vec![vec![0.0; c_len + 1]; pd],
            d22: vec![0.0; pd + 1],
        }
    }

    pub fn df(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        self.df1(x, alpha, coefficients);
        self.df2(x, alpha, coefficients);
    }

    fn df1(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        let aa = 1.0 - alpha * alpha;
        let mut out = 0.0;
        for i in (1..=self.pd).rev() {
            self.d11[i] = aa * self.d12[i - 1] + alpha * self.d11[i];
            self.d12[i] = self.d11[i] * coefficients[1];
            let v = self.d12[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        self.d12[0] = *x;
        *x += out;
    }

    fn df2(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ Coefficients) {
        let mut out = 0.0;
        for i in (1..=self.pd).rev() {
            self.d22[i] = self.fir(self.d22[i - 1], alpha, coefficients, i - 1);
            let v = self.d22[i] * self.ppade[i];
            *x += if i & 1 != 0 { v } else { -v };
            out += v;
        }
        self.d22[0] = *x;
        *x += out;
    }

    fn fir(&mut self, x: f64, alpha: f64, coefficients: &'_ Coefficients, i: usize) -> f64 {
        let aa = 1.0 - alpha * alpha;
        let d = &mut self.d21[i];
        d[0] = x;
        d[1] = aa * d[0] + alpha * d[1];

        // Code optimization was done here.
        // The original code is as follows.
        // ```rs
        // for i in 2..coefficients.len() {
        //     d[i] += alpha * (d[i + 1] - d[i - 1]);
        // }
        // let mut y = 0.0;
        // for i in 2..coefficients.len() {
        //     y += d[i] * coefficients[i];
        // }
        // for i in (2..d.len()).rev() {
        //     d[i] = d[i - 1];
        // }
        // ```
        let mut y = 0.0;
        let mut prev = d[1];
        for i in 2..coefficients.len() {
            let di = d[i] + alpha * (d[i + 1] - prev);
            y += di * coefficients[i];
            d[i] = prev;
            prev = di;
        }
        d[coefficients.len()] = prev;

        y
    }
}
