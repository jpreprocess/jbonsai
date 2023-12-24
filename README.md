# jbonsai

フルコンテキストラベルから音声を合成します．

[hts_engine_API](https://hts-engine.sourceforge.net)をRustで書き直したものです．

## 目標

- 可能な限り可読性を改善すること
- 可読性を損なわない範囲で，
  - 高速であること
  - メモリ消費量が少ないこと
- Webassembly向けにビルド可能であること

また，合成結果に関しては，短期的には
「[hts_engine_API](https://hts-engine.sourceforge.net)と同じ合成結果が得られること」
を目指していますが，将来的にはよりよいアルゴリズムがあれば，それを導入する可能性もあります．

## Copyright

This software includes source code from:

- [hts_engine_API](https://hts-engine.sourceforge.net).
  - 2001-2014 Nagoya Institute of Technology Department of Computer Science
  - 2001-2008 Tokyo Institute of Technology Interdisciplinary Graduate School of Science and Engineering

## License

BSD-3-Clause
