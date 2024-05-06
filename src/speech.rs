use crate::vocoder::Vocoder;

type Parameter = Vec<Vec<f64>>;

pub struct SpeechGenerator {
    fperiod: usize,
}

impl SpeechGenerator {
    pub fn new(fperiod: usize) -> Self {
        Self { fperiod }
    }
    /// Generate speech
    pub fn synthesize(
        &self,
        mut v: Vocoder,
        spectrum: Parameter,
        lf0: Parameter,
        lpf: Parameter,
    ) -> Vec<f64> {
        // check
        if !lf0.is_empty() {
            if lf0[0].len() != 1 {
                panic!("The size of lf0 static vector must be 1.");
            }
            if lpf[0].len() % 2 == 0 {
                panic!("The number of low-pass filter coefficient must be odd numbers.");
            }
        }

        // create speech buffer
        let total_frame = lf0.len();
        let mut speech = vec![0.0; total_frame * self.fperiod];

        // synthesize speech waveform
        for i in 0..total_frame {
            v.synthesize(
                lf0[i][0],
                &spectrum[i],
                &lpf[i],
                &mut speech[i * self.fperiod..(i + 1) * self.fperiod],
            );
        }

        speech
    }
}
