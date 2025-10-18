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
    pub fn new(nlpf: usize) -> Self {
        Self {
            pitch_of_curr_point: 0.0,
            pitch_counter: 0.0,
            pitch_inc_per_point: 0.0,
            ring_buffer: RingBuffer::new(nlpf),
            gauss: true,
            mseq: Mseq::new(),
            random: Random::new(32),
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
        *self.ring_buffer.get_antipode_mut() += noise;
    }

    /// lpf.len() == nlpf
    #[allow(clippy::needless_range_loop)]
    fn voiced_frame(&mut self, noise: f64, pulse: f64, lpf: &[f64]) {
        *self.ring_buffer.get_antipode_mut() += noise;

        let (right, left) = self.ring_buffer.as_mut_slices();
        let (lpf_right, lpf_left) = lpf.split_at(right.len());

        let c = pulse - noise;

        for i in 0..right.len() {
            right[i] += c * lpf_right[i];
        }

        assert!(left.len() <= lpf_left.len());
        for i in 0..left.len() {
            left[i] += c * lpf_left[i];
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

    fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let (left, right) = self.buffer.split_at_mut(self.index);
        (right, left)
    }

    fn get_antipode_mut(&mut self) -> &mut T {
        let index = (self.index + (self.buffer.len() - 1) / 2) % self.buffer.len();
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
    queue: Box<[f64]>,
    s: Box<[f64]>,

    used: usize,
    next: usize,
}

impl Random {
    pub fn new(rep: usize) -> Self {
        Self {
            queue: vec![0.0; rep * 2].into_boxed_slice(),
            s: vec![0.0; rep].into_boxed_slice(),

            used: rep * 2,
            next: 1,
        }
    }

    fn nrandom(&mut self) -> f64 {
        if self.used >= self.s.len() * 2 {
            self.fill_queue();
            self.used = 0;
        }

        let ret = self.queue[self.used];
        self.used += 1;
        ret
    }

    fn fill_queue(&mut self) {
        let (chunks, _) = self.queue.as_chunks_mut::<2>();
        assert!(self.s.len() <= chunks.len());

        let mut i = 0;
        while i < self.s.len() {
            let r1 = 2.0 * rnd(&mut self.next) - 1.0;
            let r2 = 2.0 * rnd(&mut self.next) - 1.0;
            let s = r1 * r1 + r2 * r2;
            if 0.0 < s && s < 1.0 {
                chunks[i][0] = r1;
                chunks[i][1] = r2;
                self.s[i] = s;
                i += 1;
            }
        }

        for (s, chunk) in self.s.iter().zip(chunks) {
            let m = (-2.0 * s.ln() / s).sqrt();
            chunk[0] *= m;
            chunk[1] *= m;
        }
    }
}

fn rnd(next: &mut usize) -> f64 {
    *next = next.wrapping_mul(1103515245).wrapping_add(12345);
    let r = next.wrapping_div(65536).wrapping_rem(32768);
    r as f64 / 32767.0
}
