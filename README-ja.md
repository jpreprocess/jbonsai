# jbonsai

[English](README.md)

フルコンテキストラベルから音声を合成するライブラリです．

TTSアプリケーションで使うことを意識して書かれています．

なお，現在は[HTS Engine](https://hts-engine.sourceforge.net)をRustで書いたものになっていますが，これは今後変わる可能性があります．また，現時点においても，jbonsaiとHTS Engineの出力が一致することは保証されていません．

## 目標

- 可能な限り可読性を改善すること
- 可読性を損なわない範囲で，
  - 高速であること
  - メモリ消費量が少ないこと
- Webassembly向けにコンパイル可能であること

## 使い方

`Cargo.toml`に次のように書いてください．

<!-- x-release-please-start-version -->

```toml
[dependencies]
jbonsai = "0.1.1"
```

<!-- x-release-please-end -->

### SIMD (experimental)

jbonsaiは、feature [portable_simd](https://github.com/rust-lang/portable-simd)による高速化をサポートしています。SIMD高速化を有効にするには、

- nightly ツールチェーンを使用する必要があります。
- 次のように`features = ["simd"]`を指定する必要があります。
  <!-- x-release-please-start-version -->
  ```toml
  [dependencies]
  jbonsai = { version = "0.2.0", features = ["simd"] }
  ```
  <!-- x-release-please-end -->

SIMDサポートは非常に実験的であり、いつでも変更される可能性があります。

## 使用例

以下の例は，「盆栽」と読み上げる音声を生成し，`speech`変数にモノラル, 48000 HzのPCMとして格納します．

```rust
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
    // `.htsvoice`モデルファイルへのパスです．
    // 現在はjlabelの制約により，日本語の音声合成モデルのみに対応しています．
    "models/hts_voice_nitech_jp_atr503_m001-1.05/nitech_jp_atr503_m001.htsvoice",
])?;
let speech = engine.synthesize(&lines)?;
println!(
    "The synthesized voice has {} samples in total.",
    speech.len()
);
```

## Copyright

このソフトウェアは以下のコードを使用しています．

- [hts_engine API](https://hts-engine.sourceforge.net).
  - Copyright (c) 2001-2014 Nagoya Institute of Technology Department of Computer Science
  - Copyright (c) 2001-2008 Tokyo Institute of Technology Interdisciplinary Graduate School of Science and Engineering

## ライセンス

BSD-3-Clause
