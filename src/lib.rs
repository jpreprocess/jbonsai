mod constants;

pub mod engine;
pub mod gstream;
pub mod label;
pub mod model;
pub mod pstream;
pub mod sstream;
pub mod vocoder;

#[cfg(test)]
mod tests {
    use crate::{engine::Engine, model::interporation_weight::Weights};

    pub const MODEL_NITECH_ATR503: &str =
        "models/hts_voice_nitech_jp_atr503_m001-1.05/nitech_jp_atr503_m001.htsvoice";
    pub const MODEL_TOHOKU_F01_NORMAL: &str = "models/tohoku-f01/tohoku-f01-neutral.htsvoice";
    pub const MODEL_TOHOKU_F01_HAPPY: &str = "models/tohoku-f01/tohoku-f01-happy.htsvoice";

    // 盆栽,名詞,一般,*,*,*,*,盆栽,ボンサイ,ボンサイ,0/4,C2
    pub const SAMPLE_SENTENCE_1: [&str;8]= [
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
    fn bonsai() {
        let lines: Vec<String> = SAMPLE_SENTENCE_1.iter().map(|l| l.to_string()).collect();

        let engine = Engine::load(&[MODEL_NITECH_ATR503.to_string()]).unwrap();

        let speech_stream = engine.synthesize_from_strings(&lines);
        let speech = speech_stream.get_speech();

        assert_eq!(speech.len(), 66480);
        approx::assert_abs_diff_eq!(speech[2000], 19.35141137623778, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[30000], -980.6757547598129, epsilon = 1.0e-10);
    }

    #[test]
    fn bonsai_multi() {
        let lines: Vec<String> = SAMPLE_SENTENCE_1.iter().map(|l| l.to_string()).collect();

        let mut engine = Engine::load(&[MODEL_TOHOKU_F01_NORMAL, MODEL_TOHOKU_F01_HAPPY]).unwrap();
        let iw = engine.condition.get_interporation_weight_mut();
        iw.set_duration(Weights::new(&[0.7, 0.3]).unwrap()).unwrap();
        iw.set_parameter(0, Weights::new(&[0.7, 0.3]).unwrap())
            .unwrap();
        iw.set_parameter(1, Weights::new(&[0.7, 0.3]).unwrap())
            .unwrap();
        iw.set_parameter(2, Weights::new(&[1.0, 0.0]).unwrap())
            .unwrap();

        let speech_stream = engine.synthesize_from_strings(&lines);
        let speech = speech_stream.get_speech();

        assert_eq!(speech.len(), 74880);
        approx::assert_abs_diff_eq!(speech[2000], 2.3158134981607754e-5, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[30000], 6459.375032316974, epsilon = 1.0e-10);
    }

    // これ,名詞,代名詞,一般,*,*,*,これ,コレ,コレ,0/2,C3,-1
    // は,助詞,係助詞,*,*,*,*,は,ハ,ワ,0/1,動詞%F2/形容詞%F2/名詞%F1,-1
    // 盆栽,名詞,一般,*,*,*,*,盆栽,ボンサイ,ボンサイ,0/4,C2,-1
    // です,助動詞,*,*,*,特殊・デス,基本形,です,デス,デス’,1/2,動詞%F1/形容詞%F2/名詞%F2@1,-1
    // か,助詞,副助詞／並立助詞／終助詞,*,*,*,*,か,カ,カ,0/1,動詞%F2/形容詞%F2/名詞%F1,-1
    // ？,記号,一般,*,*,*,*,？,？,？,0/0,*,-1
    pub const SAMPLE_SENTENCE_2: [&str;20]= [
        "xx^xx-sil+k=o/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:3_3%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:2_10/K:1+2-10",
        "xx^sil-k+o=r/A:-2+1+3/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_10/G:7_5%1_xx_1/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "sil^k-o+r=e/A:-2+1+3/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_10/G:7_5%1_xx_1/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "k^o-r+e=w/A:-1+2+2/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_10/G:7_5%1_xx_1/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "o^r-e+w=a/A:-1+2+2/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_10/G:7_5%1_xx_1/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "r^e-w+a=b/A:0+3+1/B:04-xx_xx/C:24_xx+xx/D:02+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_10/G:7_5%1_xx_1/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "e^w-a+b=o/A:0+3+1/B:04-xx_xx/C:24_xx+xx/D:02+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_10/G:7_5%1_xx_1/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "w^a-b+o=N/A:-4+1+7/B:24-xx_xx/C:02_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "a^b-o+N=s/A:-4+1+7/B:24-xx_xx/C:02_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "b^o-N+s=a/A:-3+2+6/B:24-xx_xx/C:02_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "o^N-s+a=i/A:-2+3+5/B:24-xx_xx/C:02_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "N^s-a+i=d/A:-2+3+5/B:24-xx_xx/C:02_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "s^a-i+d=e/A:-1+4+4/B:24-xx_xx/C:02_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "a^i-d+e=s/A:0+5+3/B:02-xx_xx/C:10_7+2/D:23+xx_xx/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "i^d-e+s=U/A:0+5+3/B:02-xx_xx/C:10_7+2/D:23+xx_xx/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "d^e-s+U=k/A:1+6+2/B:02-xx_xx/C:10_7+2/D:23+xx_xx/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "e^s-U+k=a/A:1+6+2/B:02-xx_xx/C:10_7+2/D:23+xx_xx/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "s^U-k+a=sil/A:2+7+1/B:10-7_2/C:23_xx+xx/D:xx+xx_xx/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "U^k-a+sil=xx/A:2+7+1/B:10-7_2/C:23_xx+xx/D:xx+xx_xx/E:3_3!0_xx-1/F:7_5#1_xx@2_1|4_7/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-10@1+1&1-2|1+10/J:xx_xx/K:1+2-10",
        "k^a-sil+xx=xx/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:7_5!1_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:2_10/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+2-10",
];

    #[test]
    fn is_this_bonsai() {
        let lines: Vec<String> = SAMPLE_SENTENCE_2.iter().map(|l| l.to_string()).collect();

        let engine = Engine::load(&[MODEL_NITECH_ATR503.to_string()]).unwrap();

        let speech_stream = engine.synthesize_from_strings(&lines);
        let speech = speech_stream.get_speech();

        assert_eq!(speech.len(), 100800);
        approx::assert_abs_diff_eq!(speech[2000], 17.15977345625943, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[30000], 2566.2058730889985, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[70000], -1898.2890228814217, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[100799], -13.514971382534956, epsilon = 1.0e-10);
    }

    #[test]
    fn is_this_bonsai_fast() {
        let lines: Vec<String> = SAMPLE_SENTENCE_2.iter().map(|l| l.to_string()).collect();

        let mut engine = Engine::load(&[MODEL_NITECH_ATR503.to_string()]).unwrap();
        engine.condition.set_speed(1.4);

        let speech_stream = engine.synthesize_from_strings(&lines);
        let speech = speech_stream.get_speech();

        assert_eq!(speech.len(), 72000);
        approx::assert_abs_diff_eq!(speech[2000], 15.0481014871396, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[30000], -56.77163803227678, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[70000], -9.15409432584658, epsilon = 1.0e-10);
        approx::assert_abs_diff_eq!(speech[71199], 7.840225089163972, epsilon = 1.0e-10);
    }

    #[test]
    fn empty() {
        let engine = Engine::load(&[MODEL_NITECH_ATR503.to_string()]).unwrap();
        let speech_stream = engine.synthesize_from_strings(&[]);
        let speech = speech_stream.get_speech();
        assert_eq!(speech.len(), 0);
    }
}
