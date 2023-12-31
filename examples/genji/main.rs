use jbonsai::engine::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let label_str = std::fs::read_to_string("examples/genji/genji.lab")?;

    let lines: Vec<String> = label_str.lines().map(|s| s.to_string()).collect();
    let mut engine = Engine::load(&vec![
        "models/tohoku-f01/tohoku-f01-sad.htsvoice",
        "models/tohoku-f01/tohoku-f01-happy.htsvoice",
    ]);
    let iw = engine.condition.get_interporation_weight_mut();
    iw.set_duration(vec![0.5, 0.5]);
    iw.set_parameter(0, vec![0.5, 0.5]);
    iw.set_parameter(1, vec![0.5, 0.5]);
    iw.set_parameter(2, vec![1.0, 0.0]);
    engine.synthesize_from_strings(&lines);

    println!(
        "The synthesized voice has {} samples in total.",
        engine.get_total_nsamples()
    );

    // let mut writer = hound::WavWriter::create(
    //     "result/genji.wav",
    //     hound::WavSpec {
    //         channels: 1,
    //         sample_rate: 48000,
    //         bits_per_sample: 16,
    //         sample_format: hound::SampleFormat::Int,
    //     },
    // )?;
    // for i in 0..engine.get_total_nsamples() {
    //     let value = engine.get_generated_speech_with_index(i);
    //     writer.write_sample(value as i16)?;
    // }

    Ok(())
}
