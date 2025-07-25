#[derive(Debug, Clone)]
pub struct Excitation {
    pitch: Pitch,
    ring_buffer: Option<RingBuffer>,
    random: Random,
}

impl Excitation {
    pub fn new(nlpf: usize) -> Self {
        let ring_buffer = if nlpf > 0 {
            Some(RingBuffer::new(nlpf))
        } else {
            None
        };
        Self {
            pitch: Pitch::new(),
            ring_buffer,
            random: Random::new(),
        }
    }

    pub fn start(&mut self, pitch: f64, fperiod: usize) {
        self.pitch.start(pitch, fperiod);
    }

    /// lpf.len() == nlpf
    pub fn get(&mut self, lpf: &[f64]) -> f64 {
        match &mut self.ring_buffer {
            Some(ring_buffer) => {
                if self.pitch.is_voiced() {
                    let noise = self.random.nrandom();
                    let pulse = self.pitch.get_pulse();
                    ring_buffer.voiced_frame(noise, pulse, lpf)
                } else {
                    let noise = self.random.nrandom();
                    ring_buffer.unvoiced_frame(noise);
                }
                ring_buffer.advance()
            }
            None => {
                if self.pitch.is_voiced() {
                    self.pitch.get_pulse()
                } else {
                    self.random.nrandom()
                }
            }
        }
    }

    pub fn end(&mut self, pitch: f64) {
        self.pitch.end(pitch);
    }
}

#[derive(Debug, Clone)]
struct Pitch {
    current: f64,
    counter: f64,
    increment: f64,
}

impl Pitch {
    fn new() -> Self {
        Self {
            current: 0.0,
            counter: 0.0,
            increment: 0.0,
        }
    }

    fn start(&mut self, pitch: f64, fperiod: usize) {
        if self.current != 0.0 && pitch != 0.0 {
            self.increment = (pitch - self.current) / fperiod as f64;
        } else {
            self.increment = 0.0;
            self.current = pitch;
            self.counter = pitch;
        }
    }

    fn is_voiced(&self) -> bool {
        self.current != 0.0
    }

    fn get_pulse(&mut self) -> f64 {
        self.counter += 1.0;
        let ret = if self.counter >= self.current {
            self.counter -= self.current;
            self.current.sqrt()
        } else {
            0.0
        };
        self.current += self.increment;
        ret
    }

    fn end(&mut self, pitch: f64) {
        self.current = pitch;
    }
}

#[derive(Debug, Clone)]
struct RingBuffer {
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

    fn advance(&mut self) -> f64 {
        let ret = self.buffer[self.index];
        self.buffer[self.index] = 0.0;
        self.index = (self.index + 1) % self.buffer.len();
        ret
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Debug, Clone)]
struct Random {
    sw: bool,
    r1: f64,
    r2: f64,
    s: f64,
    next: usize,
}

impl Random {
    fn new() -> Self {
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
