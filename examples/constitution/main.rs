use jbonsai::engine::Engine;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let label_str = std::fs::read_to_string("examples/constitution/constitution.lab")?;

    let lines: Vec<_> = label_str.lines().collect();
    let engine = Engine::load(["models/tohoku-f01/tohoku-f01-neutral.htsvoice"])?;
    let speech = engine.synthesize(&*lines)?;

    for value in speech {
        let clamped = value.min(i16::MAX as f64).max(i16::MIN as f64);
        let bytes = (clamped as i16).to_le_bytes();
        std::io::stdout().write_all(&bytes)?;
    }

    Ok(())
}
