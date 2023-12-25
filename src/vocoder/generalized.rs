use super::buffer::Buffer;

pub trait Generalized: Clone + Buffer {
    fn gamma(&self) -> f64;

    fn gnorm(&self) -> Self {
        let mut target = self.clone();

        if self.gamma() != 0.0 {
            let k = 1.0 + self.gamma() * self[0];
            target[0] = k.powf(1.0 / self.gamma());
            for i in 1..self.len() {
                target[i] = self[i] / k;
            }
        } else {
            target[0] = self[0].exp();
            target[1..].copy_from_slice(&self[1..]);
        };

        target
    }

    fn ignorm(&self) -> Self {
        let mut target = self.clone();

        if self.gamma() != 0.0 {
            let k = self[0].powf(self.gamma());
            target[0] = (k - 1.0) / self.gamma();
            for i in 1..self.len() {
                target[i] = self[i] * k;
            }
        } else {
            target[0] = self[0].ln();
            target[1..].copy_from_slice(&self[1..]);
        };

        target
    }
}
