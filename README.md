# jbonsai

Voice synthesis library for Text-to-Speech applications. Converts sequence of full-context labels into audio waveform.

フルコンテキストラベルから音声を合成するライブラリです．TTSアプリケーションで使うことを意識して作られています．

This project is currently a rewrite of [HTS Engine](https://hts-engine.sourceforge.net) in Rust language.

現在のところ，[HTS Engine](https://hts-engine.sourceforge.net)をRustで書き直したものとなっています．

## Objectives / 目標

- Improve readability as much as possible.
- Without compromising readability,
  - Improve speed.
  - Keep memory consumption low.
- Buildable for WebAssembly.

- 可能な限り可読性を改善すること．
- 可読性を損なわない範囲で，
  - 高速であること
  - メモリ消費量が少ないこと
- Webassembly向けにビルド可能であること

## Usage

Put the following in `Cargo.toml`.

```toml
[dependencies]
jbonsai = "0.1.0"
```

## Example

This is an example of creating a TTS application using [jpreprocess v0.9.1](https://crates.io/crates/jpreprocess/0.9.1),
[hound v3.5.1](https://crates.io/crates/hound/3.5.1) along with jbonsai.

For simpler example using only jbonsai, please refer to [docs.rs](https://docs.rs/jbonsai/0.1.0/jbonsai/).

```rust
// First, convert text into full-context label (requires jpreprocess v0.9.1).
let config = jpreprocess::JPreprocessConfig {
    dictionary: jpreprocess::SystemDictionaryConfig::File(/* path to dictionary file */),
    user_dictionary: None,
};
let jpreprocess = jpreprocess::JPreprocess::from_config(config)?;

let jpcommon_label = jpreprocess
    .extract_fullcontext("日本語文を解析し、音声合成エンジンに渡せる形式に変換します．")?;

// Next, synthesize voice.
let engine = crate::Engine::load(&[
    "models/hts_voice_nitech_jp_atr503_m001-1.05/nitech_jp_atr503_m001.htsvoice",
])?;
let speech = engine.synthesize(jpcommon_label)?;

println!(
    "The synthesized voice has {} samples in total.",
    speech.len()
);

// Finally, write the resulting audio file to `result/sample.wav` (requires hound v3.5.1).
let mut writer = hound::WavWriter::create(
    "result/sample.wav",
    hound::WavSpec {
        channels: 1,
        // As `nitech_jp_atr503_m001` voice model's sampling frequency is 48,000 Hz,
        // the resulting audio data will also be 48,000 Hz.
        sample_rate: 48000,
        // jbonsai produces f64 waveform, but i16 is a more popular format, so we will use i16 here.
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    },
)?;
for value in speech {
    let clamped = value.clamp(i16::MIN as f64, i16::MAX as f64);
    writer.write_sample(clamped as i16)?;
}
```

## Copyright

This software includes source code from:

- [hts_engine API](https://hts-engine.sourceforge.net).
  - Copyright (c) 2001-2014 Nagoya Institute of Technology Department of Computer Science
  - Copyright (c) 2001-2008 Tokyo Institute of Technology Interdisciplinary Graduate School of Science and Engineering

## License

BSD-3-Clause
