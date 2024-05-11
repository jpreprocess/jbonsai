//! Holds parameters and generate speech iteratively or in batch.

use crate::vocoder::Vocoder;

type Parameter = Vec<Vec<f64>>;

/// A structure that contains all parameters necessary to generate speech waveform.
pub struct SpeechGenerator {
    fperiod: usize,
    vocoder: Vocoder,
    spectrum: Parameter,
    lf0: Parameter,
    lpf: Parameter,

    next: usize,
}

impl SpeechGenerator {
    /// Create a new [`SpeechGenerator`] with provided parameters.
    ///
    /// This function will panic unless all the following conditions are met:
    /// - The outer length of spectrum, lf0, and lpf are the same.
    /// - The inner length of LF0 must be 1.
    /// - The inner length of LPF must be an odd number.
    pub fn new(
        fperiod: usize,
        vocoder: Vocoder,
        spectrum: Parameter,
        lf0: Parameter,
        lpf: Parameter,
    ) -> Self {
        if spectrum.len() != lf0.len() || spectrum.len() != lpf.len() {
            panic!("The length of spectrum, lf0, lpf must be the same.")
        }
        if !lf0.is_empty() && lf0[0].len() != 1 {
            panic!("The size of lf0 static vector must be 1.");
        }
        if !lpf.is_empty() && lpf[0].len() % 2 == 0 {
            panic!("The number of low-pass filter coefficient must be odd numbers.");
        }

        Self {
            fperiod,
            vocoder,
            spectrum,
            lf0,
            lpf,
            next: 0,
        }
    }

    /// Get `fperiod`, which equals to the number of samples synthesized in a single call of [`SpeechGenerator::generate_step`].
    pub fn fperiod(&self) -> usize {
        self.fperiod
    }

    /// Get the number of frames that are already synthesized.
    pub fn synthesized_frames(&self) -> usize {
        self.next
    }

    /// Generate speech of length `fperiod` in `speech`.
    ///
    /// The length of `speech` must be longer than `fperiod`, otherwise, this function will panic.
    pub fn generate_step(&mut self, speech: &mut [f64]) -> usize {
        if self.lf0.len() <= self.next {
            return 0;
        }
        if speech.len() < self.fperiod {
            panic!("The length of speech buffer must be larger than fperiod.");
        }

        self.vocoder.synthesize(
            self.lf0[self.next][0],
            &self.spectrum[self.next],
            &self.lpf[self.next],
            speech,
        );
        self.next += 1;

        self.fperiod
    }

    /// Generate the full-length speech.
    ///
    /// Please note that this function will not generate previously-synthesized frames again.
    pub fn generate_all(mut self) -> Vec<f64> {
        if self.next != 0 {
            eprintln!("The speech generator has already synthesized some frames.");
        }

        let mut buf = vec![0.0; (self.lf0.len() - self.next) * self.fperiod];
        while self.generate_step(&mut buf[self.next * self.fperiod..]) > 0 {}

        buf
    }
}
