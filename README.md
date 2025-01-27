# jbonsai

[日本語](https://github.com/jpreprocess/jbonsai/blob/main/README-ja.md)

Voice synthesis library for Text-to-Speech applications.

"jbonsai" converts sequence of full-context labels into audio waveform.

This project is currently a rewrite of [HTS Engine](https://hts-engine.sourceforge.net) in Rust language (This may change at any time, and there is no guarantee that jbonsai produces the same result as HTS Engine).

## Objectives

- Improve readability as much as possible.
- Without compromising readability,
  - Improve speed.
  - Keep memory consumption low.
- Can be compiled for WebAssembly.

## Usage

Put the following in `Cargo.toml`.

```toml
[dependencies]
jbonsai = "0.1.0"
```

### SIMD (experimental)

jbonsai supports acceleration provided by feature [portable_simd](https://github.com/rust-lang/portable-simd). In order to enable SIMD acceleration,

- you must use nightly toolchain.
- you have to specify `features = ["simd"]` as follows:
  ```toml
  [dependencies]
  jbonsai = { version = "0.1.0", features = ["simd"] }
  ```

The SIMD support is highly experimental and may change at any time.

## Example

This example produces a mono, 48,000 Hz (typically) PCM data saying 「盆栽」(ぼんさい; bonsai) in `speech` variable.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
// 盆栽,名詞,一般,*,*,*,*,盆栽,ボンサイ,ボンサイ,0/4,C2
let lines = [
    "xx^xx-sil+b=o/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_4%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_4/K:1+1-4",
    "xx^sil-b+o=N/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "sil^b-o+N=s/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "b^o-N+s=a/A:-2+2+3/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "o^N-s+a=i/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "N^s-a+i=sil/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "s^a-i+sil=xx/A:0+4+1/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
    "a^i-sil+xx=xx/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:4_4!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_4/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+1-4",
];
let engine = jbonsai::Engine::load(&[
    // The path to the `.htsvoice` model file.
    // Currently only Japanese models are supported (due to the limitation of jlabel).
    "models/hts_voice_nitech_jp_atr503_m001-1.05/nitech_jp_atr503_m001.htsvoice",
])?;
let speech = engine.synthesize(&lines)?;
println!(
    "The synthesized voice has {} samples in total.",
    speech.len()
);
# Ok(())
# }
```

## Copyright

This software includes source code from:

- [hts_engine API](https://hts-engine.sourceforge.net).
  - Copyright (c) 2001-2014 Nagoya Institute of Technology Department of Computer Science
  - Copyright (c) 2001-2008 Tokyo Institute of Technology Interdisciplinary Graduate School of Science and Engineering

## License

BSD-3-Clause
