use super::coefficients::GeneralizedCoefficients;

#[derive(Debug, Clone)]
pub struct MelGeneralizedLogSpectrumApproximation {
    d: Vec<Vec<f64>>,
}

impl MelGeneralizedLogSpectrumApproximation {
    pub fn new(n: usize, c_len: usize) -> Self {
        Self {
            d: vec![vec![0.0; c_len]; n],
        }
    }

    pub fn df(&mut self, x: &mut f64, alpha: f64, coefficients: &'_ GeneralizedCoefficients) {
        for i in 0..self.d.len() {
            self.dff(x, alpha, coefficients, i);
        }
    }

    fn dff(
        &mut self,
        x: &mut f64,
        alpha: f64,
        coefficients: &'_ GeneralizedCoefficients,
        i: usize,
    ) {
        let d = &mut self.d[i];
        let aa = 1.0 - alpha * alpha;

        let mut y = d[0] * coefficients[1];
        for i in 1..coefficients.len() - 1 {
            d[i] += alpha * (d[i + 1] - d[i - 1]);
            y += d[i] * coefficients[i + 1];
        }
        *x -= y;
        for i in (1..coefficients.len()).rev() {
            d[i] = d[i - 1];
        }
        d[0] = alpha * d[0] + aa * *x;
    }
}
