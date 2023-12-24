use std::rc::Rc;

use libc::FILE;

use crate::gstream::GStreamSet;
use crate::label::Label;
use crate::model::ModelSet;
use crate::pstream::PStreamSet;
use crate::sstream::SStreamSet;
use crate::util::{HALF_TONE, MAX_LF0, MIN_LF0, DB};

#[derive(Clone)]
pub struct Condition {
    pub sampling_frequency: usize,
    pub fperiod: usize,
    pub volume: f64,
    pub msd_threshold: Vec<f64>,
    pub gv_weight: Vec<f64>,
    pub phoneme_alignment_flag: bool,
    pub speed: f64,
    pub stage: usize,
    pub use_log_gain: bool,
    pub alpha: f64,
    pub beta: f64,
    pub additional_half_tone: f64,
    pub duration_iw: Vec<f64>,
    pub parameter_iw: Vec<Vec<f64>>,
    pub gv_iw: Vec<Vec<f64>>,
}

pub struct Engine {
    pub condition: Condition,
    pub ms: Rc<ModelSet>,
    pub label: Option<Label>,
    pub sss: Option<SStreamSet>,
    pub pss: Option<PStreamSet>,
    pub gss: Option<GStreamSet>,
}

impl Engine {
    pub fn load(voices: &Vec<String>) -> Engine {
        let mut condition = Condition {
            sampling_frequency: 0,
            fperiod: 0,
            volume: 1.0f64,
            msd_threshold: Vec::new(),
            gv_weight: Vec::new(),
            speed: 1.0f64,
            phoneme_alignment_flag: false,
            stage: 0,
            use_log_gain: false,
            alpha: 0.0f64,
            beta: 0.0f64,
            additional_half_tone: 0.0f64,
            duration_iw: Vec::new(),
            parameter_iw: Vec::new(),
            gv_iw: Vec::new(),
        };

        /* load voices */
        let ms = ModelSet::load_htsvoice_files(voices).unwrap();
        let nstream = ms.get_nstream();
        let average_weight = 1.0f64 / voices.len() as f64;

        /* global */
        condition.sampling_frequency = ms.get_sampling_frequency();
        condition.fperiod = ms.get_fperiod();
        condition.msd_threshold = (0..nstream).into_iter().map(|_| 0.5).collect();
        condition.gv_weight = (0..nstream).into_iter().map(|_| 1.0).collect();

        /* spectrum */
        for option in ms.get_option(0) {
            let Some((key, value)) = option.split_once('=') else {
                eprintln!("Skipped unrecognized option {}.", option);
                continue;
            };
            match key {
                "GAMMA" => condition.stage = value.parse().unwrap(),
                "LN_GAIN" => condition.use_log_gain = value == "1",
                "ALPHA" => condition.alpha = value.parse().unwrap(),
                _ => eprintln!("Skipped unrecognized option {}.", option),
            }
        }

        /* interpolation weights */
        condition.duration_iw = (0..voices.len())
            .into_iter()
            .map(|_| average_weight)
            .collect();
        condition.parameter_iw = (0..voices.len())
            .into_iter()
            .map(|_| (0..nstream).into_iter().map(|_| average_weight).collect())
            .collect();
        condition.gv_iw = (0..voices.len())
            .into_iter()
            .map(|_| (0..nstream).into_iter().map(|_| average_weight).collect())
            .collect();

        Engine {
            condition,
            ms: Rc::new(ms),
            label: None,
            sss: None,
            pss: None,
            gss: None,
        }
    }

    pub fn set_sampling_frequency(&mut self, mut i: usize) {
        if i < 1 {
            i = 1;
        }
        self.condition.sampling_frequency = i;
    }

    pub fn get_sampling_frequency(&mut self) -> usize {
        self.condition.sampling_frequency
    }

    pub fn set_fperiod(&mut self, mut i: usize) {
        if i < 1 {
            i = 1;
        }
        self.condition.fperiod = i;
    }

    pub fn get_fperiod(&mut self) -> usize {
        self.condition.fperiod
    }

    pub fn set_volume(&mut self, f: f64) {
        self.condition.volume = (f * DB).exp();
    }

    pub fn get_volume(&mut self) -> f64 {
        self.condition.volume.ln() / DB
    }

    pub fn set_msd_threshold(&mut self, stream_index: usize, mut f: f64) {
        if f < 0.0 {
            f = 0.0;
        }
        if f > 1.0 {
            f = 1.0;
        }
        self.condition.msd_threshold[stream_index] = f;
    }

    pub fn get_msd_threshold(&mut self, stream_index: usize) -> f64 {
        self.condition.msd_threshold[stream_index]
    }

    pub fn set_gv_weight(&mut self, stream_index: usize, mut f: f64) {
        if f < 0.0 {
            f = 0.0;
        }
        self.condition.gv_weight[stream_index] = f;
    }

    pub fn get_gv_weight(&mut self, stream_index: usize) -> f64 {
        self.condition.gv_weight[stream_index]
    }

    pub fn set_speed(&mut self, mut f: f64) {
        if f < 1.0E-06f64 {
            f = 1.0E-06f64;
        }
        self.condition.speed = f;
    }

    pub fn set_phoneme_alignment_flag(&mut self, b: bool) {
        self.condition.phoneme_alignment_flag = b;
    }

    pub fn set_alpha(&mut self, mut f: f64) {
        if f < 0.0f64 {
            f = 0.0f64;
        }
        if f > 1.0f64 {
            f = 1.0f64;
        }
        self.condition.alpha = f;
    }

    pub fn get_alpha(&mut self) -> f64 {
        self.condition.alpha
    }

    pub fn set_beta(&mut self, mut f: f64) {
        if f < 0.0f64 {
            f = 0.0f64;
        }
        if f > 1.0f64 {
            f = 1.0f64;
        }
        self.condition.beta = f;
    }

    pub fn get_beta(&mut self) -> f64 {
        self.condition.beta
    }

    pub fn add_half_tone(&mut self, f: f64) {
        self.condition.additional_half_tone = f;
    }

    pub fn set_duration_interpolation_weight(&mut self, voice_index: usize, f: f64) {
        self.condition.duration_iw[voice_index] = f;
    }

    pub fn get_duration_interpolation_weight(&mut self, voice_index: usize) -> f64 {
        self.condition.duration_iw[voice_index]
    }

    pub fn set_parameter_interpolation_weight(
        &mut self,
        voice_index: usize,
        stream_index: usize,
        f: f64,
    ) {
        self.condition.parameter_iw[voice_index][stream_index] = f;
    }

    pub fn get_parameter_interpolation_weight(
        &mut self,
        voice_index: usize,
        stream_index: usize,
    ) -> f64 {
        self.condition.parameter_iw[voice_index][stream_index]
    }

    pub fn set_gv_interpolation_weight(&mut self, voice_index: usize, stream_index: usize, f: f64) {
        self.condition.gv_iw[voice_index][stream_index] = f;
    }

    pub fn get_gv_interpolation_weight(&mut self, voice_index: usize, stream_index: usize) -> f64 {
        self.condition.gv_iw[voice_index][stream_index]
    }

    pub fn get_total_state(&mut self) -> usize {
        self.sss.as_ref().unwrap().get_total_state()
    }

    pub fn set_state_mean(
        &mut self,
        stream_index: usize,
        state_index: usize,
        vector_index: usize,
        f: f64,
    ) {
        self.sss
            .as_mut()
            .unwrap()
            .set_mean(stream_index, state_index, vector_index, f);
    }

    pub fn get_state_mean(
        &mut self,
        stream_index: usize,
        state_index: usize,
        vector_index: usize,
    ) -> f64 {
        self.sss
            .as_ref()
            .unwrap()
            .get_mean(stream_index, state_index, vector_index)
    }

    pub fn get_state_duration(&mut self, state_index: usize) -> usize {
        self.sss.as_ref().unwrap().get_duration(state_index)
    }

    pub fn get_nvoices(&mut self) -> usize {
        self.ms.get_nvoices()
    }

    pub fn get_nstream(&mut self) -> usize {
        self.ms.get_nstream()
    }

    pub fn get_nstate(&mut self) -> usize {
        self.ms.get_nstate()
    }

    pub fn get_fullcontext_label_format(&mut self) -> &str {
        self.ms.get_fullcontext_label_format()
    }

    pub fn get_fullcontext_label_version(&mut self) -> &str {
        self.ms.get_fullcontext_label_version()
    }

    pub fn get_total_frame(&mut self) -> usize {
        self.gss.as_ref().unwrap().get_total_frame()
    }

    // pub unsafe fn get_nsamples(&mut self) -> usize {
    //     GStreamSet_get_total_nsamples(&mut self.gss)
    // }

    // pub unsafe fn get_generated_parameter(
    //     &mut self,
    //     stream_index: usize,
    //     frame_index: usize,
    //     vector_index: usize,
    // ) -> f64 {
    //     GStreamSet_get_parameter(&mut self.gss, stream_index, frame_index, vector_index)
    // }

    pub fn get_generated_speech(&mut self, index: usize) -> f64 {
        self.gss.as_ref().unwrap().get_speech(index)
    }
    fn generate_state_sequence(&mut self) {
        self.sss = SStreamSet::create(
            self.ms.clone(),
            self.label.as_ref().unwrap(),
            self.condition.phoneme_alignment_flag,
            self.condition.speed,
            &mut self.condition.duration_iw,
            &mut self.condition.parameter_iw,
            &mut self.condition.gv_iw,
        );
        if self.condition.additional_half_tone != 0.0 {
            for i in 0..self.get_total_state() {
                let mut f = self.get_state_mean(1, i, 0);
                f += self.condition.additional_half_tone * HALF_TONE;
                f = f.max(MIN_LF0).min(MAX_LF0);
                self.set_state_mean(1, i, 0, f);
            }
        }
    }

    // pub unsafe fn generate_state_sequence_from_fn(
    //     &mut self,
    //     fn_0: *const libc::c_char,
    // ) {
    //     refresh(engine);
    //     Label_load_from_fn(
    //         &mut self.label,
    //         self.condition.sampling_frequency,
    //         self.condition.fperiod,
    //         fn_0,
    //     );
    //     generate_state_sequence(engine)
    // }

    pub fn generate_state_sequence_from_strings(&mut self, lines: &[String]) {
        self.refresh();
        self.label = Some(Label::load_from_strings(
            self.condition.sampling_frequency,
            self.condition.fperiod,
            lines,
        ));
        self.generate_state_sequence();
    }

    pub fn generate_parameter_sequence(&mut self) {
        self.pss = Some(PStreamSet::create(
            self.sss.as_ref().unwrap(),
            &self.condition.msd_threshold,
            &self.condition.gv_weight,
        ));
    }

    pub fn generate_sample_sequence(&mut self) {
        self.gss = Some(GStreamSet::create(
            self.pss.as_ref().unwrap(),
            self.condition.stage,
            self.condition.use_log_gain,
            self.condition.sampling_frequency,
            self.condition.fperiod,
            self.condition.alpha,
            self.condition.beta,
            self.condition.volume,
        ));
    }
    fn synthesize(&mut self) {
        self.generate_state_sequence();
        self.generate_parameter_sequence();
        self.generate_sample_sequence();
    }

    // pub unsafe fn synthesize_from_fn(
    //     &mut self,
    //     fn_0: *const libc::c_char,
    // ) {
    //     refresh(engine);
    //     Label_load_from_fn(
    //         &mut self.label,
    //         self.condition.sampling_frequency,
    //         self.condition.fperiod,
    //         fn_0,
    //     );
    //     synthesize(engine)
    // }

    pub fn synthesize_from_strings(&mut self, lines: &[String]) {
        self.refresh();
        self.label = Some(Label::load_from_strings(
            self.condition.sampling_frequency,
            self.condition.fperiod,
            lines,
        ));
        self.synthesize()
    }

    pub unsafe fn save_information(&mut self, fp: *mut FILE) {
        // let mut i: usize = 0;
        // let mut j: usize = 0;
        // let mut k: usize = 0;
        // let mut l: usize = 0;
        // let mut m: usize = 0;
        // let mut n: usize = 0;
        // let mut temp: f64 = 0.;
        // let condition: &mut Condition = &mut self.condition;
        // let ms: &mut ModelSet = &mut self.ms;
        // let label: &mut Label = &mut self.label;
        // let sss = &mut self.sss;
        // let pss = &mut self.pss;
        // fprintf(
        //     fp,
        //     b"[Global parameter]\n\0" as *const u8 as *const libc::c_char,
        // );
        // fprintf(
        //     fp,
        //     b"Sampring frequency                     -> %8lu(Hz)\n\0" as *const u8
        //         as *const libc::c_char,
        //     condition.sampling_frequency,
        // );
        // fprintf(
        //     fp,
        //     b"Frame period                           -> %8lu(point)\n\0" as *const u8
        //         as *const libc::c_char,
        //     condition.fperiod,
        // );
        // fprintf(
        //     fp,
        //     b"                                          %8.5f(msec)\n\0" as *const u8
        //         as *const libc::c_char,
        //     1e+3f64 * condition.fperiod as f64
        //         / condition.sampling_frequency as f64,
        // );
        // fprintf(
        //     fp,
        //     b"All-pass constant                      -> %8.5f\n\0" as *const u8 as *const libc::c_char,
        //     condition.alpha as libc::c_float as f64,
        // );
        // fprintf(
        //     fp,
        //     b"Gamma                                  -> %8.5f\n\0" as *const u8 as *const libc::c_char,
        //     (if condition.stage == 0 as libc::c_int {
        //         0.0f64
        //     } else {
        //         -1.0f64 / condition.stage as f64
        //     }) as libc::c_float as f64,
        // );
        // if condition.stage != 0 as libc::c_int {
        //     if condition.use_log_gain as libc::c_int == 1 as libc::c_int {
        //         fprintf(
        //             fp,
        //             b"Log gain flag                          ->     TRUE\n\0" as *const u8
        //                 as *const libc::c_char,
        //         );
        //     } else {
        //         fprintf(
        //             fp,
        //             b"Log gain flag                          ->    FALSE\n\0" as *const u8
        //                 as *const libc::c_char,
        //         );
        //     }
        // }
        // fprintf(
        //     fp,
        //     b"Postfiltering coefficient              -> %8.5f\n\0" as *const u8 as *const libc::c_char,
        //     condition.beta as libc::c_float as f64,
        // );
        // fprintf(
        //     fp,
        //     b"Audio buffer size                      -> %8lu(sample)\n\0" as *const u8
        //         as *const libc::c_char,
        //     condition.audio_buff_size,
        // );
        // fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
        // fprintf(
        //     fp,
        //     b"[Duration parameter]\n\0" as *const u8 as *const libc::c_char,
        // );
        // fprintf(
        //     fp,
        //     b"Number of states                       -> %8lu\n\0" as *const u8 as *const libc::c_char,
        //     ModelSet_get_nstate(ms),
        // );
        // fprintf(
        //     fp,
        //     b"         Interpolation size            -> %8lu\n\0" as *const u8 as *const libc::c_char,
        //     ModelSet_get_nvoices(ms),
        // );
        // i = 0 as libc::c_int;
        // temp = 0.0f64;
        // while i < ModelSet_get_nvoices(ms) {
        //     temp += *(condition.duration_iw).offset(i as isize);
        //     i = i.wrapping_add(1);
        // }
        // i = 0 as libc::c_int;
        // while i < ModelSet_get_nvoices(ms) {
        //     if *(condition.duration_iw).offset(i as isize) != 0.0f64 {
        //         *(condition.duration_iw).offset(i as isize) /= temp;
        //     }
        //     i = i.wrapping_add(1);
        // }
        // i = 0 as libc::c_int;
        // while i < ModelSet_get_nvoices(ms) {
        //     fprintf(
        //         fp,
        //         b"         Interpolation weight[%2lu]      -> %8.0f(%%)\n\0" as *const u8
        //             as *const libc::c_char,
        //         i,
        //         (100 as libc::c_int as f64 * *(condition.duration_iw).offset(i as isize))
        //             as libc::c_float as f64,
        //     );
        //     i = i.wrapping_add(1);
        // }
        // fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
        // fprintf(
        //     fp,
        //     b"[Stream parameter]\n\0" as *const u8 as *const libc::c_char,
        // );
        // i = 0 as libc::c_int;
        // while i < ModelSet_get_nstream(ms) {
        //     fprintf(
        //         fp,
        //         b"Stream[%2lu] vector length               -> %8lu\n\0" as *const u8
        //             as *const libc::c_char,
        //         i,
        //         ModelSet_get_vector_length(ms, i),
        //     );
        //     fprintf(
        //         fp,
        //         b"           Dynamic window size         -> %8lu\n\0" as *const u8
        //             as *const libc::c_char,
        //         ModelSet_get_window_size(ms, i),
        //     );
        //     fprintf(
        //         fp,
        //         b"           Interpolation size          -> %8lu\n\0" as *const u8
        //             as *const libc::c_char,
        //         ModelSet_get_nvoices(ms),
        //     );
        //     j = 0 as libc::c_int;
        //     temp = 0.0f64;
        //     while j < ModelSet_get_nvoices(ms) {
        //         temp += *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize);
        //         j = j.wrapping_add(1);
        //     }
        //     j = 0 as libc::c_int;
        //     while j < ModelSet_get_nvoices(ms) {
        //         if *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize) != 0.0f64 {
        //             *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize) /= temp;
        //         }
        //         j = j.wrapping_add(1);
        //     }
        //     j = 0 as libc::c_int;
        //     while j < ModelSet_get_nvoices(ms) {
        //         fprintf(
        //             fp,
        //             b"           Interpolation weight[%2lu]    -> %8.0f(%%)\n\0" as *const u8
        //                 as *const libc::c_char,
        //             j,
        //             (100 as libc::c_int as f64
        //                 * *(*(condition.parameter_iw).offset(j as isize)).offset(i as isize))
        //                 as libc::c_float as f64,
        //         );
        //         j = j.wrapping_add(1);
        //     }
        //     if ModelSet_is_msd(ms, i) != 0 {
        //         fprintf(
        //             fp,
        //             b"           MSD flag                    ->     TRUE\n\0" as *const u8
        //                 as *const libc::c_char,
        //         );
        //         fprintf(
        //             fp,
        //             b"           MSD threshold               -> %8.5f\n\0" as *const u8
        //                 as *const libc::c_char,
        //             *(condition.msd_threshold).offset(i as isize),
        //         );
        //     } else {
        //         fprintf(
        //             fp,
        //             b"           MSD flag                    ->    FALSE\n\0" as *const u8
        //                 as *const libc::c_char,
        //         );
        //     }
        //     if ModelSet_use_gv(ms, i) != 0 {
        //         fprintf(
        //             fp,
        //             b"           GV flag                     ->     TRUE\n\0" as *const u8
        //                 as *const libc::c_char,
        //         );
        //         fprintf(
        //             fp,
        //             b"           GV weight                   -> %8.0f(%%)\n\0" as *const u8
        //                 as *const libc::c_char,
        //             (100 as libc::c_int as f64 * *(condition.gv_weight).offset(i as isize))
        //                 as libc::c_float as f64,
        //         );
        //         fprintf(
        //             fp,
        //             b"           GV interpolation size       -> %8lu\n\0" as *const u8
        //                 as *const libc::c_char,
        //             ModelSet_get_nvoices(ms),
        //         );
        //         j = 0 as libc::c_int;
        //         temp = 0.0f64;
        //         while j < ModelSet_get_nvoices(ms) {
        //             temp += *(*(condition.gv_iw).offset(j as isize)).offset(i as isize);
        //             j = j.wrapping_add(1);
        //         }
        //         j = 0 as libc::c_int;
        //         while j < ModelSet_get_nvoices(ms) {
        //             if *(*(condition.gv_iw).offset(j as isize)).offset(i as isize) != 0.0f64 {
        //                 *(*(condition.gv_iw).offset(j as isize)).offset(i as isize) /= temp;
        //             }
        //             j = j.wrapping_add(1);
        //         }
        //         j = 0 as libc::c_int;
        //         while j < ModelSet_get_nvoices(ms) {
        //             fprintf(
        //                 fp,
        //                 b"           GV interpolation weight[%2lu] -> %8.0f(%%)\n\0" as *const u8
        //                     as *const libc::c_char,
        //                 j,
        //                 (100 as libc::c_int as f64
        //                     * *(*(condition.gv_iw).offset(j as isize)).offset(i as isize))
        //                     as libc::c_float as f64,
        //             );
        //             j = j.wrapping_add(1);
        //         }
        //     } else {
        //         fprintf(
        //             fp,
        //             b"           GV flag                     ->    FALSE\n\0" as *const u8
        //                 as *const libc::c_char,
        //         );
        //     }
        //     i = i.wrapping_add(1);
        // }
        // fprintf(fp, b"\n\0" as *const u8 as *const libc::c_char);
        // fprintf(
        //     fp,
        //     b"[Generated sequence]\n\0" as *const u8 as *const libc::c_char,
        // );
        // fprintf(
        //     fp,
        //     b"Number of HMMs                         -> %8lu\n\0" as *const u8 as *const libc::c_char,
        //     Label_get_size(label),
        // );
        // fprintf(
        //     fp,
        //     b"Number of stats                        -> %8lu\n\0" as *const u8 as *const libc::c_char,
        //     (Label_get_size(label)).wrapping_mul(ModelSet_get_nstate(ms)),
        // );
        // fprintf(
        //     fp,
        //     b"Length of this speech                  -> %8.3f(sec)\n\0" as *const u8
        //         as *const libc::c_char,
        //     (PStreamSet_get_total_frame(pss) as f64
        //         * condition.fperiod as f64
        //         / condition.sampling_frequency as f64) as libc::c_float
        //         as f64,
        // );
        // fprintf(
        //     fp,
        //     b"                                       -> %8lu(frames)\n\0" as *const u8
        //         as *const libc::c_char,
        //     (PStreamSet_get_total_frame(pss)).wrapping_mul(condition.fperiod),
        // );
        // i = 0 as libc::c_int;
        // while i < Label_get_size(label) {
        //     fprintf(fp, b"HMM[%2lu]\n\0" as *const u8 as *const libc::c_char, i);
        //     fprintf(
        //         fp,
        //         b"  Name                                 -> %s\n\0" as *const u8 as *const libc::c_char,
        //         Label_get_string(label, i),
        //     );
        //     fprintf(fp, b"  Duration\n\0" as *const u8 as *const libc::c_char);
        //     j = 0 as libc::c_int;
        //     while j < ModelSet_get_nvoices(ms) {
        //         fprintf(
        //             fp,
        //             b"    Interpolation[%2lu]\n\0" as *const u8 as *const libc::c_char,
        //             j,
        //         );
        //         ModelSet_get_duration_index(ms, j, Label_get_string(label, i), &mut k, &mut l);
        //         fprintf(
        //             fp,
        //             b"      Tree index                       -> %8lu\n\0" as *const u8
        //                 as *const libc::c_char,
        //             k,
        //         );
        //         fprintf(
        //             fp,
        //             b"      PDF index                        -> %8lu\n\0" as *const u8
        //                 as *const libc::c_char,
        //             l,
        //         );
        //         j = j.wrapping_add(1);
        //     }
        //     j = 0 as libc::c_int;
        //     while j < ModelSet_get_nstate(ms) {
        //         fprintf(
        //             fp,
        //             b"  State[%2lu]\n\0" as *const u8 as *const libc::c_char,
        //             j.wrapping_add(2 as libc::c_int as libc::c_ulong),
        //         );
        //         fprintf(
        //             fp,
        //             b"    Length                             -> %8lu(frames)\n\0" as *const u8
        //                 as *const libc::c_char,
        //             SStreamSet_get_duration(sss, (i * ModelSet_get_nstate(ms)).wrapping_add(j)),
        //         );
        //         k = 0 as libc::c_int;
        //         while k < ModelSet_get_nstream(ms) {
        //             fprintf(
        //                 fp,
        //                 b"    Stream[%2lu]\n\0" as *const u8 as *const libc::c_char,
        //                 k,
        //             );
        //             if ModelSet_is_msd(ms, k) != 0 {
        //                 if SStreamSet_get_msd(
        //                     sss,
        //                     k,
        //                     (i * ModelSet_get_nstate(ms)).wrapping_add(j),
        //                 ) > *(condition.msd_threshold).offset(k as isize)
        //                 {
        //                     fprintf(
        //                         fp,
        //                         b"      MSD flag                         ->     TRUE\n\0" as *const u8
        //                             as *const libc::c_char,
        //                     );
        //                 } else {
        //                     fprintf(
        //                         fp,
        //                         b"      MSD flag                         ->    FALSE\n\0" as *const u8
        //                             as *const libc::c_char,
        //                     );
        //                 }
        //             }
        //             l = 0 as libc::c_int;
        //             while l < ModelSet_get_nvoices(ms) {
        //                 fprintf(
        //                     fp,
        //                     b"      Interpolation[%2lu]\n\0" as *const u8 as *const libc::c_char,
        //                     l,
        //                 );
        //                 ModelSet_get_parameter_index(
        //                     ms,
        //                     l,
        //                     k,
        //                     j.wrapping_add(2 as libc::c_int),
        //                     Label_get_string(label, i),
        //                     &mut m,
        //                     &mut n,
        //                 );
        //                 fprintf(
        //                     fp,
        //                     b"        Tree index                     -> %8lu\n\0" as *const u8
        //                         as *const libc::c_char,
        //                     m,
        //                 );
        //                 fprintf(
        //                     fp,
        //                     b"        PDF index                      -> %8lu\n\0" as *const u8
        //                         as *const libc::c_char,
        //                     n,
        //                 );
        //                 l = l.wrapping_add(1);
        //             }
        //             k = k.wrapping_add(1);
        //         }
        //         j = j.wrapping_add(1);
        //     }
        //     i = i.wrapping_add(1);
        // }
    }

    pub unsafe fn save_label(&mut self, fp: *mut FILE) {
        // let mut i = 0;
        // let mut j = 0;
        // let mut frame = 0;
        // let mut state = 0;
        // let mut duration = 0;
        // let label = self.label.as_ref().unwrap();
        // let sss = self.sss.as_ref().unwrap();
        // let nstate = self.ms.get_nstate();
        // let rate: f64 =
        //     self.condition.fperiod as f64 * 1.0e+07f64 / self.condition.sampling_frequency as f64;
        // i = 0;
        // state = 0;
        // frame = 0;
        // while i < label.get_size() {
        //     j = 0;
        //     duration = 0;
        //     while j < nstate {
        //         let fresh2 = state;
        //         state += 1;
        //         duration += sss.get_duration(fresh2);
        //         j = j.wrapping_add(1);
        //     }
        //     fprintf(
        //         fp,
        //         b"%lu %lu %s\n\0" as *const u8 as *const libc::c_char,
        //         (frame as f64 * rate) as libc::c_ulong,
        //         ((frame as f64 + duration as f64) * rate) as libc::c_ulong,
        //         label.get_string(i),
        //     );
        //     frame += duration;
        //     i = i.wrapping_add(1);
        // }
    }

    // pub unsafe fn save_generated_parameter(
    //     &mut self,
    //     stream_index: usize,
    //     fp: *mut FILE,
    // ) {
    //     let mut i: usize = 0;
    //     let mut j: usize = 0;
    //     let mut temp: libc::c_float = 0.;
    //     let gss: &mut GStreamSet = &mut self.gss;
    //     i = 0 as libc::c_int;
    //     while i < GStreamSet_get_total_frame(gss) {
    //         j = 0 as libc::c_int;
    //         while j < GStreamSet_get_vector_length(gss, stream_index) {
    //             temp = GStreamSet_get_parameter(gss, stream_index, i, j) as libc::c_float;
    //             fwrite(
    //                 &mut temp as *mut libc::c_float as *const libc::c_void,
    //                 ::core::mem::size_of::<libc::c_float>() as libc::c_ulong,
    //                 1 as libc::c_int as libc::c_ulong,
    //                 fp,
    //             );
    //             j = j.wrapping_add(1);
    //         }
    //         i = i.wrapping_add(1);
    //     }
    // }

    // pub unsafe fn save_generated_speech(&mut self, fp: *mut FILE) {
    //     let mut i: usize = 0;
    //     let mut x: f64 = 0.;
    //     let mut temp: libc::c_short = 0;
    //     let gss: &mut GStreamSet = &mut self.gss;
    //     i = 0 as libc::c_int;
    //     while i < GStreamSet_get_total_nsamples(gss) {
    //         x = GStreamSet_get_speech(gss, i);
    //         if x > 32767.0f64 {
    //             temp = 32767 as libc::c_int as libc::c_short;
    //         } else if x < -32768.0f64 {
    //             temp = -(32768 as libc::c_int) as libc::c_short;
    //         } else {
    //             temp = x as libc::c_short;
    //         }
    //         fwrite(
    //             &mut temp as *mut libc::c_short as *const libc::c_void,
    //             ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
    //             1 as libc::c_int as libc::c_ulong,
    //             fp,
    //         );
    //         i = i.wrapping_add(1);
    //     }
    // }

    // pub unsafe fn save_riff(&mut self, fp: *mut FILE) {
    //     let mut i: usize = 0;
    //     let mut x: f64 = 0.;
    //     let mut temp: libc::c_short = 0;
    //     let gss: &mut GStreamSet = &mut self.gss;
    //     let mut data_01_04: [libc::c_char; 4] = [
    //         'R' as i32 as libc::c_char,
    //         'I' as i32 as libc::c_char,
    //         'F' as i32 as libc::c_char,
    //         'F' as i32 as libc::c_char,
    //     ];
    //     let mut data_05_08: libc::c_int = (GStreamSet_get_total_nsamples(gss))
    //         .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
    //         .wrapping_add(36 as libc::c_int as libc::c_ulong)
    //         as libc::c_int;
    //     let mut data_09_12: [libc::c_char; 4] = [
    //         'W' as i32 as libc::c_char,
    //         'A' as i32 as libc::c_char,
    //         'V' as i32 as libc::c_char,
    //         'E' as i32 as libc::c_char,
    //     ];
    //     let mut data_13_16: [libc::c_char; 4] = [
    //         'f' as i32 as libc::c_char,
    //         'm' as i32 as libc::c_char,
    //         't' as i32 as libc::c_char,
    //         ' ' as i32 as libc::c_char,
    //     ];
    //     let mut data_17_20: libc::c_int = 16 as libc::c_int;
    //     let mut data_21_22: libc::c_short = 1 as libc::c_int as libc::c_short;
    //     let mut data_23_24: libc::c_short = 1 as libc::c_int as libc::c_short;
    //     let mut data_25_28: libc::c_int = self.condition.sampling_frequency as libc::c_int;
    //     let mut data_29_32: libc::c_int = (self.condition.sampling_frequency)
    //         .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
    //         as libc::c_int;
    //     let mut data_33_34: libc::c_short =
    //         ::core::mem::size_of::<libc::c_short>() as libc::c_ulong as libc::c_short;
    //     let mut data_35_36: libc::c_short = (::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
    //         .wrapping_mul(8 as libc::c_int as libc::c_ulong)
    //         as libc::c_short;
    //     let mut data_37_40: [libc::c_char; 4] = [
    //         'd' as i32 as libc::c_char,
    //         'a' as i32 as libc::c_char,
    //         't' as i32 as libc::c_char,
    //         'a' as i32 as libc::c_char,
    //     ];
    //     let mut data_41_44: libc::c_int = (GStreamSet_get_total_nsamples(gss))
    //         .wrapping_mul(::core::mem::size_of::<libc::c_short>() as libc::c_ulong)
    //         as libc::c_int;
    //     fwrite_little_endian(
    //         data_01_04.as_mut_ptr() as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
    //         4 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_05_08 as *mut libc::c_int as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         data_09_12.as_mut_ptr() as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
    //         4 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         data_13_16.as_mut_ptr() as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
    //         4 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_17_20 as *mut libc::c_int as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_21_22 as *mut libc::c_short as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_23_24 as *mut libc::c_short as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_25_28 as *mut libc::c_int as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_29_32 as *mut libc::c_int as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_33_34 as *mut libc::c_short as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_35_36 as *mut libc::c_short as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         data_37_40.as_mut_ptr() as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
    //         4 as libc::c_int,
    //         fp,
    //     );
    //     fwrite_little_endian(
    //         &mut data_41_44 as *mut libc::c_int as *const libc::c_void,
    //         ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
    //         1 as libc::c_int,
    //         fp,
    //     );
    //     i = 0 as libc::c_int;
    //     while i < GStreamSet_get_total_nsamples(gss) {
    //         x = GStreamSet_get_speech(gss, i);
    //         if x > 32767.0f64 {
    //             temp = 32767 as libc::c_int as libc::c_short;
    //         } else if x < -32768.0f64 {
    //             temp = -(32768 as libc::c_int) as libc::c_short;
    //         } else {
    //             temp = x as libc::c_short;
    //         }
    //         fwrite_little_endian(
    //             &mut temp as *mut libc::c_short as *const libc::c_void,
    //             ::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
    //             1 as libc::c_int,
    //             fp,
    //         );
    //         i = i.wrapping_add(1);
    //     }
    // }

    pub fn refresh(&mut self) {
        self.label = None;
        self.sss = None;
        self.pss = None;
        self.gss = None;
    }
}
