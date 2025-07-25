#[derive(Debug, Clone)]
pub struct Excitation {
    pitch_of_curr_point: f64,
    pitch_counter: f64,
    pitch_inc_per_point: f64,
    ring_buffer: RingBuffer,
    random: Random,
}

impl Excitation {
    pub fn new(nlpf: usize) -> Self {
        Self {
            pitch_of_curr_point: 0.0,
            pitch_counter: 0.0,
            pitch_inc_per_point: 0.0,
            ring_buffer: RingBuffer::new(nlpf),
            random: Random::new(),
        }
    }

    pub fn start(&mut self, pitch: f64, fperiod: usize) {
        if self.pitch_of_curr_point != 0.0 && pitch != 0.0 {
            self.pitch_inc_per_point = (pitch - self.pitch_of_curr_point) / fperiod as f64;
        } else {
            self.pitch_inc_per_point = 0.0;
            self.pitch_of_curr_point = pitch;
            self.pitch_counter = pitch;
        }
    }

    fn white_noise(&mut self) -> f64 {
        self.random.nrandom()
    }

    /// lpf.len() == nlpf
    pub fn get(&mut self, lpf: &[f64]) -> f64 {
        if self.ring_buffer.len() > 0 {
            let noise = self.white_noise();
            if self.pitch_of_curr_point == 0.0 {
                self.ring_buffer.unvoiced_frame(noise);
            } else {
                self.pitch_counter += 1.0;
                let pulse = if self.pitch_counter >= self.pitch_of_curr_point {
                    self.pitch_counter -= self.pitch_of_curr_point;
                    self.pitch_of_curr_point.sqrt()
                } else {
                    0.0
                };
                self.ring_buffer.voiced_frame(noise, pulse, lpf);
                self.pitch_of_curr_point += self.pitch_inc_per_point;
            }
            let x = *self.ring_buffer.get();
            *self.ring_buffer.get_mut() = 0.0;
            self.ring_buffer.advance();
            x
        } else if self.pitch_of_curr_point == 0.0 {
            self.white_noise()
        } else {
            self.pitch_counter += 1.0;
            let x = if self.pitch_counter >= self.pitch_of_curr_point {
                self.pitch_counter -= self.pitch_of_curr_point;
                self.pitch_of_curr_point.sqrt()
            } else {
                0.0
            };
            self.pitch_of_curr_point += self.pitch_inc_per_point;
            x
        }
    }

    pub fn end(&mut self, pitch: f64) {
        self.pitch_of_curr_point = pitch;
    }
}

#[derive(Debug, Clone)]
pub struct RingBuffer {
    buffer: Vec<f64>,
    index: usize,
}

impl RingBuffer {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
        }
    }

    fn get(&self) -> &f64 {
        &self.buffer[self.index]
    }

    fn get_mut(&mut self) -> &mut f64 {
        &mut self.buffer[self.index]
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        let (left, right) = self.buffer.split_at_mut(self.index);
        right.iter_mut().chain(left.iter_mut())
    }

    fn unvoiced_frame(&mut self, noise: f64) {
        let index = (self.index + (self.len() - 1) / 2) % self.buffer.len();
        self.buffer[index] += noise;
    }

    #[allow(clippy::needless_range_loop)]
    fn voiced_frame(&mut self, noise: f64, pulse: f64, lpf: &[f64]) {
        self.unvoiced_frame(noise);
        for (bi, lpf_i) in self.iter_mut().zip(lpf.iter()) {
            *bi += (pulse - noise) * lpf_i;
        }
    }

    fn advance(&mut self) {
        self.index += 1;
        if self.index >= self.buffer.len() {
            self.index = 0;
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Debug, Clone)]
pub struct Random {
    sw: bool,
    r1: f64,
    r2: f64,
    s: f64,
    next: usize,
}

impl Random {
    pub fn new() -> Self {
        Self {
            sw: false,
            r1: 0.0,
            r2: 0.0,
            s: 0.0,
            next: 1,
        }
    }

    fn nrandom(&mut self) -> f64 {
        if self.sw {
            self.sw = false;
            self.r2 * self.s
        } else {
            self.sw = true;
            loop {
                self.r1 = 2.0 * self.rnd() - 1.0;
                self.r2 = 2.0 * self.rnd() - 1.0;
                self.s = self.r1 * self.r1 + self.r2 * self.r2;
                if !(self.s > 1.0 || self.s == 0.0) {
                    break;
                }
            }
            self.s = (-2.0 * self.s.ln() / self.s).sqrt();
            self.r1 * self.s
        }
    }

    fn rnd(&mut self) -> f64 {
        self.next = self.next.wrapping_mul(1103515245).wrapping_add(12345);
        let r = self.next.wrapping_div(65536).wrapping_rem(32768);
        r as f64 / 32767.0
    }
}
