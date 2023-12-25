#[derive(Debug, Clone)]
pub struct Excitation {
    pitch_of_curr_point: f64,
    pitch_counter: f64,
    pitch_inc_per_point: f64,
    ring_buffer: RingBuffer<f64>,
    gauss: bool,
    mseq: Mseq,
    random: Random,
}

impl Excitation {
    pub fn new(pitch: f64, nlpf: usize) -> Self {
        Self {
            pitch_of_curr_point: pitch,
            pitch_counter: pitch,
            pitch_inc_per_point: 0.0,
            ring_buffer: RingBuffer::new(nlpf),
            gauss: true,
            mseq: Mseq::new(),
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
        if self.gauss {
            self.random.nrandom()
        } else {
            self.mseq.next() as f64
        }
    }

    fn unvoiced_frame(&mut self, noise: f64) {
        let center = (self.ring_buffer.len() - 1) / 2;
        *self.ring_buffer.get_mut_with_offset(center) += noise;
    }

    /// lpf.len() == nlpf
    fn voiced_frame(&mut self, noise: f64, pulse: f64, lpf: &[f64]) {
        let center = (self.ring_buffer.len() - 1) / 2;
        if noise != 0.0 {
            for i in 0..self.ring_buffer.len() {
                if i == center {
                    *self.ring_buffer.get_mut_with_offset(i) += noise * (1.0 - lpf[i]);
                } else {
                    *self.ring_buffer.get_mut_with_offset(i) += noise * (0.0 - lpf[i]);
                }
            }
        }
        if pulse != 0.0 {
            for i in 0..self.ring_buffer.len() {
                *self.ring_buffer.get_mut_with_offset(i) += pulse * lpf[i];
            }
        }
    }

    /// lpf.len() == nlpf
    pub fn get(&mut self, lpf: &[f64]) -> f64 {
        if self.ring_buffer.len() > 0 {
            let noise = self.white_noise();
            if self.pitch_of_curr_point == 0.0 {
                self.unvoiced_frame(noise);
            } else {
                self.pitch_counter += 1.0;
                let pulse = if self.pitch_counter >= self.pitch_of_curr_point {
                    self.pitch_counter -= self.pitch_of_curr_point;
                    self.pitch_of_curr_point.sqrt()
                } else {
                    0.0
                };
                self.voiced_frame(noise, pulse, lpf);
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
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    index: usize,
}

impl<T> RingBuffer<T> {
    fn new(size: usize) -> Self
    where
        T: Default + Clone,
    {
        Self {
            buffer: vec![Default::default(); size],
            index: 0,
        }
    }

    fn get(&self) -> &T {
        &self.buffer[self.index]
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.buffer[self.index]
    }

    fn get_mut_with_offset(&mut self, i: usize) -> &mut T {
        let index = (self.index + i) % self.buffer.len();
        &mut self.buffer[index]
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
pub struct Mseq {
    x: u32,
}

impl Mseq {
    pub fn new() -> Self {
        Self { x: 0x55555555 }
    }

    fn next(&mut self) -> i32 {
        self.x >>= 1;
        let x0 = if self.x & 0x00000001 != 0 { 1 } else { -1 };
        let x28 = if self.x & 0x10000000 != 0 { 1 } else { -1 };
        if x0 + x28 != 0 {
            self.x &= 0x7fffffff;
        } else {
            self.x |= 0x80000000;
        }
        x0
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
