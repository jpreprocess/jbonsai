use crate::{constants::NODATA, pstream::ParameterStreamSet, vocoder::Vocoder};

pub struct GenerateSpeechStreamSet {
    speech: Vec<f64>,
}

type FrameIndexTable = Vec<Vec<usize>>;

impl GenerateSpeechStreamSet {
    /// Generate speech
    pub fn create(pss: &ParameterStreamSet, v: Vocoder, fperiod: usize, chunk_size: usize) -> Self {
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
        let table = Self::generate_frame_index_table(pss);
        let mut speech = vec![0.0; total_frame * fperiod];

        // synthesize speech waveform
        if cfg!(feature = "multithread") && chunk_size < total_frame {
            #[cfg(feature = "multithread")]
            {
                use rayon::prelude::*;
                let num_chunks = total_frame.div_ceil(chunk_size);
                speech
                    .par_chunks_mut(chunk_size * fperiod)
                    .enumerate()
                    .for_each(|(i, speech_chunk)| {
                        let start_chunk = chunk_size * i;
                        let size = if i == num_chunks - 1 {
                            // last chunk
                            total_frame - start_chunk
                        } else {
                            chunk_size
                        };
                        Self::dispatch_synthesis(
                            v.clone(),
                            speech_chunk,
                            pss,
                            start_chunk,
                            size,
                            fperiod,
                            &table,
                        );
                    });
            }

            #[cfg(not(feature = "multithread"))]
            unreachable!();
        } else {
            Self::dispatch_synthesis(v, &mut speech, pss, 0, total_frame, fperiod, &table);
        }

        GenerateSpeechStreamSet { speech }
    }

    fn generate_frame_index_table(pss: &ParameterStreamSet) -> FrameIndexTable {
        (0..pss.get_nstream())
            .map(|stream_index| {
                (0..pss.get_total_frame())
                    .scan((0, 0), |(_, index), frame_index| {
                        let orig_index = *index;
                        if pss.get_msd_flag(stream_index, frame_index) {
                            *index += 1
                        }
                        Some((orig_index, *index))
                    })
                    .map(|(index, _)| index)
                    .collect()
            })
            .collect()
    }

    fn dispatch_synthesis(
        mut v: Vocoder,
        speech: &mut [f64],
        pss: &ParameterStreamSet,
        start_index: usize,
        frame_len: usize,
        fperiod: usize,
        table: &FrameIndexTable,
    ) {
        for i in 0..frame_len {
            let get_parameter = |stream_index: usize, vector_index: usize| {
                let frame_index = start_index + i;
                if !pss.get_msd_flag(stream_index, frame_index) {
                    NODATA
                } else {
                    pss.get_parameter(stream_index, table[stream_index][frame_index], vector_index)
                }
            };

            let lpf = if pss.get_nstream() >= 3 {
                (0..pss.get_vector_length(2))
                    .map(|vector_index| get_parameter(2, vector_index))
                    .collect()
            } else {
                vec![]
            };
            let spectrum: Vec<f64> = (0..pss.get_vector_length(0))
                .map(|vector_index| get_parameter(0, vector_index))
                .collect();

            v.synthesize(
                get_parameter(1, 0),
                &spectrum,
                &lpf,
                &mut speech[i * fperiod..(i + 1) * fperiod],
            );
        }
    }

    /// Get synthesized speech waveform
    pub fn get_speech(&self) -> &[f64] {
        &self.speech
    }
}
