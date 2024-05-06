use jbonsai::engine::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let label_str = std::fs::read_to_string("examples/genji/genji.lab")?;

    let lines: Vec<_> = label_str.lines().collect();
    let mut engine = Engine::load(&[
        "models/tohoku-f01/tohoku-f01-sad.htsvoice",
        "models/tohoku-f01/tohoku-f01-happy.htsvoice",
    ])?;
    let iw = engine.condition.get_interporation_weight_mut();
    iw.set_duration(&[0.5, 0.5])?;
    iw.set_parameter(0, &[0.5, 0.5])?;
    iw.set_parameter(1, &[0.5, 0.5])?;
    iw.set_parameter(2, &[1.0, 0.0])?;

    let speech = engine.synthesize_from_strings(&lines)?;

    println!(
        "The synthesized voice has {} samples in total.",
        speech.len()
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
    // for value in speech {
    //     let clamped = value.min(i16::MAX as f64).max(i16::MIN as f64);
    //     writer.write_sample(clamped as i16)?;
    // }

    Ok(())
}
