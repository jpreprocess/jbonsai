use std::sync::Arc;

use crate::constants::{DB, HALF_TONE, MAX_LF0, MIN_LF0};
use crate::gstream::GenerateSpeechStreamSet;
use crate::label::Label;
use crate::model::ModelSet;
use crate::pstream::ParameterStreamSet;
use crate::sstream::StateStreamSet;

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

impl Default for Condition {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Condition {
    pub fn load_model(&mut self, ms: &ModelSet) {
        let voice_len = ms.get_nvoices();
        let nstream = ms.get_nstream();
        let average_weight = 1.0f64 / voice_len as f64;

        /* global */
        self.sampling_frequency = ms.get_sampling_frequency();
        self.fperiod = ms.get_fperiod();
        self.msd_threshold = (0..nstream).map(|_| 0.5).collect();
        self.gv_weight = (0..nstream).map(|_| 1.0).collect();

        /* spectrum */
        for option in ms.get_option(0) {
            let Some((key, value)) = option.split_once('=') else {
                eprintln!("Skipped unrecognized option {}.", option);
                continue;
            };
            match key {
                "GAMMA" => self.stage = value.parse().unwrap(),
                "LN_GAIN" => self.use_log_gain = value == "1",
                "ALPHA" => self.alpha = value.parse().unwrap(),
                _ => eprintln!("Skipped unrecognized option {}.", option),
            }
        }

        /* interpolation weights */
        self.duration_iw = (0..voice_len).map(|_| average_weight).collect();
        self.parameter_iw = (0..voice_len)
            .map(|_| (0..nstream).map(|_| average_weight).collect())
            .collect();
        self.gv_iw = (0..voice_len)
            .map(|_| (0..nstream).map(|_| average_weight).collect())
            .collect();
    }
}

pub struct Engine {
    pub condition: Condition,
    pub ms: Arc<ModelSet>,
    pub label: Option<Label>,
    pub sss: Option<StateStreamSet>,
    pub pss: Option<ParameterStreamSet>,
    pub gss: Option<GenerateSpeechStreamSet>,
}

impl Engine {
    pub fn load(voices: &[String]) -> Engine {
        let ms = ModelSet::load_htsvoice_files(voices).unwrap();
        Self::new(Arc::new(ms))
    }
    pub fn new(ms: Arc<ModelSet>) -> Engine {
        let mut condition = Condition::default();
        condition.load_model(&ms);

        Engine {
            condition,
            ms,
            label: None,
            sss: None,
            pss: None,
            gss: None,
        }
    }

    pub fn set_sampling_frequency(&mut self, i: usize) {
        self.condition.sampling_frequency = i.max(1);
    }

    pub fn get_sampling_frequency(&self) -> usize {
        self.condition.sampling_frequency
    }

    pub fn set_fperiod(&mut self, i: usize) {
        self.condition.fperiod = i.max(1);
    }

    pub fn get_fperiod(&mut self) -> usize {
        self.condition.fperiod
    }

    pub fn set_volume(&mut self, f: f64) {
        self.condition.volume = (f * DB).exp();
    }

    pub fn get_volume(&self) -> f64 {
        self.condition.volume.ln() / DB
    }

    pub fn set_msd_threshold(&mut self, stream_index: usize, f: f64) {
        self.condition.msd_threshold[stream_index] = f.min(1.0).max(0.0);
    }

    pub fn get_msd_threshold(&self, stream_index: usize) -> f64 {
        self.condition.msd_threshold[stream_index]
    }

    pub fn set_gv_weight(&mut self, stream_index: usize, f: f64) {
        self.condition.gv_weight[stream_index] = f.max(0.0);
    }

    pub fn get_gv_weight(&self, stream_index: usize) -> f64 {
        self.condition.gv_weight[stream_index]
    }

    pub fn set_speed(&mut self, f: f64) {
        self.condition.speed = f.max(1.0E-06);
    }

    pub fn set_phoneme_alignment_flag(&mut self, b: bool) {
        self.condition.phoneme_alignment_flag = b;
    }

    pub fn set_alpha(&mut self, f: f64) {
        self.condition.alpha = f.max(0.0).min(1.0);
    }

    pub fn get_alpha(&self) -> f64 {
        self.condition.alpha
    }

    pub fn set_beta(&mut self, f: f64) {
        self.condition.beta = f.max(0.0).min(1.0);
    }

    pub fn get_beta(&self) -> f64 {
        self.condition.beta
    }

    pub fn add_half_tone(&mut self, f: f64) {
        self.condition.additional_half_tone = f;
    }

    pub fn set_duration_interpolation_weight(&mut self, voice_index: usize, f: f64) {
        self.condition.duration_iw[voice_index] = f;
    }

    pub fn get_duration_interpolation_weight(&self, voice_index: usize) -> f64 {
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
        &self,
        stream_index: usize,
        state_index: usize,
        vector_index: usize,
    ) -> f64 {
        self.sss
            .as_ref()
            .unwrap()
            .get_mean(stream_index, state_index, vector_index)
    }

    pub fn get_state_duration(&self, state_index: usize) -> usize {
        self.sss.as_ref().unwrap().get_duration(state_index)
    }

    pub fn get_nvoices(&self) -> usize {
        self.ms.get_nvoices()
    }

    pub fn get_nstream(&self) -> usize {
        self.ms.get_nstream()
    }

    pub fn get_nstate(&self) -> usize {
        self.ms.get_nstate()
    }

    pub fn get_fullcontext_label_format(&self) -> &str {
        self.ms.get_fullcontext_label_format()
    }

    pub fn get_fullcontext_label_version(&self) -> &str {
        self.ms.get_fullcontext_label_version()
    }

    pub fn get_total_nsamples(&self) -> usize {
        self.gss.as_ref().unwrap().get_total_nsamples()
    }

    pub fn get_generated_speech(&self, index: usize) -> f64 {
        self.gss.as_ref().unwrap().get_speech(index)
    }
    fn generate_state_sequence(&mut self) {
        self.sss = StateStreamSet::create(
            self.ms.clone(),
            self.label.as_ref().unwrap(),
            self.condition.phoneme_alignment_flag,
            self.condition.speed,
            &self.condition.duration_iw,
            &self.condition.parameter_iw,
            &self.condition.gv_iw,
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
        self.pss = Some(ParameterStreamSet::create(
            self.sss.as_ref().unwrap(),
            &self.condition.msd_threshold,
            &self.condition.gv_weight,
        ));
    }

    pub fn generate_sample_sequence(&mut self) {
        self.gss = Some(GenerateSpeechStreamSet::create(
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

    pub fn synthesize_from_strings(&mut self, lines: &[String]) {
        self.refresh();
        self.label = Some(Label::load_from_strings(
            self.condition.sampling_frequency,
            self.condition.fperiod,
            lines,
        ));
        self.synthesize()
    }

    pub fn refresh(&mut self) {
        self.label = None;
        self.sss = None;
        self.pss = None;
        self.gss = None;
    }
}
