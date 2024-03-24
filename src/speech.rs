use crate::vocoder::Vocoder;

type Parameter = Vec<Vec<f64>>;

pub struct SpeechGenerator {
    fperiod: usize,
    alpha: f64,
    beta: f64,
    volume: f64,
}

impl SpeechGenerator {
    pub fn new(fperiod: usize, alpha: f64, beta: f64, volume: f64) -> Self {
        Self {
            fperiod,
            alpha,
            beta,
            volume,
        }
    }
    /// Generate speech
    pub fn synthesize(
        &self,
        mut v: Vocoder,
        spectrum: Parameter,
        lf0: Parameter,
        lpf: Option<Parameter>,
    ) -> Vec<f64> {
        // check
        if lf0.len() != 1 {
            panic!("The size of lf0 static vector must be 1.");
        }
        if lpf.as_ref().map(|lpf| lpf.len() % 2 == 0) == Some(true) {
            panic!("The number of low-pass filter coefficient must be odd numbers.");
        }

        // create speech buffer
        let total_frame = lf0[0].len();
        let mut speech = vec![0.0; total_frame * self.fperiod];

        // synthesize speech waveform
        for i in 0..total_frame {
            let spectrum_vector: Vec<f64> = (0..spectrum.len())
                .map(|vector_index| spectrum[vector_index][i])
                .collect();
            let lpf_vector = if let Some(ref lpf) = lpf {
                (0..lpf.len())
                    .map(|vector_index| lpf[vector_index][i])
                    .collect()
            } else {
                vec![]
            };

            v.synthesize(
                lf0[0][i],
                &spectrum_vector,
                &lpf_vector,
                self.alpha,
                self.beta,
                self.volume,
                &mut speech[i * self.fperiod..(i + 1) * self.fperiod],
            );
        }

        speech
    }
}
