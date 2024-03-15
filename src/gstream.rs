use crate::{pstream::ParameterStreamSet, vocoder::Vocoder};

pub struct GenerateSpeechStreamSet {
    speech: Vec<f64>,
}

impl GenerateSpeechStreamSet {
    /// Generate speech
    pub fn create(
        pss: &ParameterStreamSet,
        mut v: Vocoder,
        fperiod: usize,
        alpha: f64,
        beta: f64,
        volume: f64,
    ) -> Self {
        // check
        if pss.get_nstream() != 2 && pss.get_nstream() != 3 {
            panic!("The number of streams must be 2 or 3.");
        }
        if pss.get_vector_length(1) != 1 {
            panic!("The size of lf0 static vector must be 1.");
        }
        if pss.get_nstream() >= 3 && pss.get_vector_length(2) % 2 == 0 {
            panic!("The number of low-pass filter coefficient must be odd numbers.");
        }

        // create speech buffer
        let total_frame = pss.get_total_frame();
        let mut speech = vec![0.0; total_frame * fperiod];

        // synthesize speech waveform
        for i in 0..total_frame {
            let lpf = if pss.get_nstream() >= 3 {
                (0..pss.get_vector_length(2))
                    .map(|vector_index| pss.get_parameter(2, i, vector_index))
                    .collect()
            } else {
                vec![]
            };
            let spectrum: Vec<f64> = (0..pss.get_vector_length(0))
                .map(|vector_index| pss.get_parameter(0, i, vector_index))
                .collect();

            v.synthesize(
                pss.get_parameter(1, i, 0),
                &spectrum,
                &lpf,
                alpha,
                beta,
                volume,
                &mut speech[i * fperiod..(i + 1) * fperiod],
            );
        }

        GenerateSpeechStreamSet { speech }
    }

    /// Get synthesized speech waveform
    pub fn get_speech(&self) -> &[f64] {
        &self.speech
    }
}
