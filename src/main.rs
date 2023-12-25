use jbonsai::engine::Engine;

fn main() {
    let lines: Vec<String> = [
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
].iter().map(|l| l.to_string()).collect();
    let mut engine = Engine::load(&vec!["models/nitech_jp_atr503_m001.htsvoice".to_string()]);
    engine.set_speed(1.0);

    engine.synthesize_from_strings(&lines);
    let mut writer = hound::WavWriter::create(
        "bonsai.wav",
        hound::WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
dbg!(engine.get_total_nsamples());
    for i in 0..engine.get_total_nsamples() {
        let value = engine.get_generated_speech(i);
        writer.write_sample(value as i16).unwrap();
    }
}
