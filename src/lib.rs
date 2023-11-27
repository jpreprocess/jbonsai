mod HTS_engine;
mod HTS_gstream;
mod HTS_label;
mod HTS_misc;
mod HTS_model;
mod HTS_pstream;
mod HTS_sstream;
mod HTS_vocoder;

mod util;

pub use HTS_engine::*;
pub use HTS_gstream::*;
pub use HTS_label::*;
pub use HTS_misc::*;
pub use HTS_model::*;
pub use HTS_pstream::*;
pub use HTS_sstream::*;
pub use HTS_vocoder::*;

pub mod label;
pub mod model;
pub mod sstream;

#[cfg(test)]
mod tests {
    use std::{ffi::CString, mem::MaybeUninit};

    use crate::{
        HTS_Engine, HTS_Engine_get_generated_speech, HTS_Engine_load,
        HTS_Engine_synthesize_from_strings,
    };

    // 盆栽,名詞,一般,*,*,*,*,盆栽,ボンサイ,ボンサイ,0/4,C2
    pub const SAMPLE_SENTENCE: [&str;8]= [
    "xx^xx-sil+b=o/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_4%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_4/K:1+1-4",
    "xx^sil-b+o=N/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "sil^b-o+N=s/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "b^o-N+s=a/A:-2+2+3/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "o^N-s+a=i/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "N^s-a+i=sil/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "s^a-i+sil=xx/A:0+4+1/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "a^i-sil+xx=xx/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:4_4!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_4/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+1-4",
];

    #[test]
    fn load() {
        let lines: Vec<String> = SAMPLE_SENTENCE.iter().map(|l| l.to_string()).collect();

        let mut htsengine =
            HTS_Engine_load(&vec!["models/nitech_jp_atr503_m001.htsvoice".to_string()]);
        unsafe {
            HTS_Engine_synthesize_from_strings(&mut htsengine, &lines);
            let l2000 = HTS_Engine_get_generated_speech(&mut htsengine, 2000);
            assert_eq!(l2000, 19.35141137623778);
            let l30000 = HTS_Engine_get_generated_speech(&mut htsengine, 30000);
            assert_eq!(l30000, -980.6757547598129);
        }
    }
}
