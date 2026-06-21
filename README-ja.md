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
jbonsai = "0.4.1"
```

<!-- x-release-please-end -->

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

## パフォーマンス

HTS Engineとjbonsai v0.4.1の合成速度を比較しました．合成した文は日本国憲法の前文で，約128秒と比較的長い文章です．その結果，jbonsaiは1.6–2.2倍の速度で合成できることを確認しました．なお，合成された音声波形は両者で同一でした．

この性能向上は，jbonsaiで行われた，フィルタ演算等の最適化によるものと考えられます．

また，Intel x86_64では，`-C target-cpu=native`及びそれに対応するC言語のオプションを使うことで，HTS Engine，jbonsaiともに大幅に性能が向上しました（「結果詳細」参照）．

なお，このベンチマークは2026年6月，jbonsai v0.4.1に対し当時の最新版のツールを用いて行われました．より新しいjbonsaiのバージョンや，より新しいrustc，LLVM等では異なった結果となる可能性があります．

加えて，今回は音声全体を合成する時間を計測しましたが，jbonsaiはストリーミング合成にも対応しています．ボイスチャットの代わり等，リアルタイム性が重要な用途でお使いください．

![HTS Engineとjbonsaiで，合成にかかる時間を4つのプラットフォーム(Intel Core i5-13500, Apple M2, Raspberry Pi 4, Compute Module 3)で比較した棒グラフ．時間はHTS Engineが100%になるように正規化してある．jbonsaiは安定してHTS Engineよりも高速に動作し，44.8–60.1%の時間で合成できている．](https://raw.githubusercontent.com/jpreprocess/jbonsai/e03dd1416c03a30d77a276e9b0a9637ecf2ce5bf/docs/benchmark_comparison_normalized.png)

<details>
<summary>結果詳細</summary>

|    プラットフォーム | 使用ライブラリ・最適化 |  平均実行時間（秒） |
| ------------------: | ---------------------: | ------------------: |
| Intel Core i5-13500 |     hts_engine_default |           1.8076678 |
| Intel Core i5-13500 |      hts_engine_native |           1.4573735 |
| Intel Core i5-13500 |        jbonsai_default |           0.9408125 |
| Intel Core i5-13500 |            jbonsai_lto |           0.9374387 |
| Intel Core i5-13500 |         jbonsai_native |           0.8016296 |
|            Apple M2 |     hts_engine_default |           2.0302433 |
|            Apple M2 |      hts_engine_native |           1.9764867 |
|            Apple M2 |        jbonsai_default |           0.9127072 |
|            Apple M2 |            jbonsai_lto |           0.8880196 |
|            Apple M2 |         jbonsai_native |           0.8853029 |
|      Raspberry Pi 4 |     hts_engine_default |          14.1488875 |
|      Raspberry Pi 4 |      hts_engine_native |          14.1636672 |
|      Raspberry Pi 4 |        jbonsai_default |           8.2963959 |
|      Raspberry Pi 4 |            jbonsai_lto |           8.3044165 |
|      Raspberry Pi 4 |         jbonsai_native |           8.5164151 |
|    Compute Module 3 |     hts_engine_default |          32.7140591 |
|    Compute Module 3 |      hts_engine_native |          32.7214677 |
|    Compute Module 3 |        jbonsai_default |          22.0358614 |
|    Compute Module 3 |            jbonsai_lto |          21.9978278 |
|    Compute Module 3 |         jbonsai_native |          18.8089429 |

### Core i5-13500 (Manjaro Linux, Clang 22.1.5, Rustc 1.98.0-nightly)

![](https://raw.githubusercontent.com/jpreprocess/jbonsai/e03dd1416c03a30d77a276e9b0a9637ecf2ce5bf/docs/benchmark_detail_i5-13500.png)

### Apple M2 (macOS 26.4.1, Homebrew Clang 22.1.7, Rustc 1.98.0-nightly)

![](https://raw.githubusercontent.com/jpreprocess/jbonsai/e03dd1416c03a30d77a276e9b0a9637ecf2ce5bf/docs/benchmark_detail_macos.png)

### Raspberry Pi 4 (Debian GNU/Linux 13, Clang 22.1.8, Rustc 1.98.0-nightly)

![](https://raw.githubusercontent.com/jpreprocess/jbonsai/e03dd1416c03a30d77a276e9b0a9637ecf2ce5bf/docs/benchmark_detail_rpi4_bench_clang.png)

### Compute Module 3 (Debian GNU/Linux 13, Clang 22.1.8, Rustc 1.98.0-nightly)

![](https://raw.githubusercontent.com/jpreprocess/jbonsai/e03dd1416c03a30d77a276e9b0a9637ecf2ce5bf/docs/benchmark_detail_cm3_bench_clang.png)

</details>

<details>
<summary>手法</summary>

### 目的・評価方法

このベンチマークでは，音声モデルとしてtohoku-f01-neutralを用いたとき，日本国憲法前文を合成するのにかかる時間を測定した．合成された音声サンプルは約128秒（6,130,960 サンプル, 48 kHz）であった．合成速度の評価の前に，`cmp`を用い，生の音声出力が両者で完全に一致することを確認している．

### 評価対象

HTS Engineとjbonsaiの違いに加え，コンパイルオプションによる違いを見るため，以下の5つの組み合わせについて評価を行いました．

- **HTS Engine (Default):** LTO最適化を有効化したほかは通常のRelease条件でコンパイル (`-DCMAKE_BUILD_TYPE=Release -DCPU_NATIVE=OFF`).
- **HTS Engine (Native):** LTO最適化を有効化し，かつNativeのCPU向けにコンパイル (`-DCPU_NATIVE=ON`).
- **jbonsai (Default):** Cargoを用い，通常のRelease条件でコンパイルしている．`codegen-units=1`を指定し，また，LTOは無効化している (`lto=off`)．
- **jbonsai (LTO):** Defaultに加え，LTOを有効化 (`lto=on`)
- **jbonsai (Native):** LTOに加え，NativeのCPU向けにコンパイル (`-C target-cpu=native`)

### 測定

コンパイルと測定は，共通のシェルスクリプト（コミットID `86441a40b74780afef9aa9901ac5602ffd8788fd`の`hts-bench/bench.sh`）を用いて行われた．ただし，いくつかの環境ではこのスクリプトに変更を加えているので，「環境」の項を参照のこと．

測定は`hyperfine` (v1.19.0 / v1.20.0)を用いて行われた．それぞれ本測定の前に5回のウォームアップを行い，続けて25回実行時間の測定を行った．

### 環境

測定は4台の異なるコンピュータを用いて行われた．それぞれのOS等の制約に合わせるため，後述するように測定スクリプトを一部変更している．

| 環境 | CPUアーキテクチャ | OS | コンパイラ・リンカ | Power & Governor Tuning |
|------------------------|------------------|------------------|---------------------------|-------------------------|
| **Intel Core i5-13500**<br>(14 Cores / 20 Threads) | `x86_64` | Manjaro Linux | Clang 22.1.5<br>Rustc 1.98.0-nightly (f428d123a 2026-06-19)<br>LLD 22.1.5 | PL2 power limit capped at 65W via sysfs;<br>`powersave` scaling governor |
| **Apple M2**<br>(8 Physical Cores) | `arm64` | macOS 26.4.1 | Homebrew Clang 22.1.7<br>Rustc 1.98.0-nightly (f428d123a 2026-06-19)<br>Homebrew LLD 22.1.7 | N/A (OS-managed) |
| **Raspberry Pi 4**<br>(Cortex-A72, 4 Cores) | `aarch64` | Debian GNU/Linux 13 (trixie) | Clang 22.1.8<br>Rustc 1.98.0-nightly (1f087276b 2026-06-13)<br>LLD 22 | `ondemand` scaling governor |
| **Compute Module 3**<br>(Cortex-A53, 4 Cores) | `aarch64` | Debian GNU/Linux 13 (trixie) | Clang 22.1.8<br>Rustc 1.98.0-nightly (1f087276b 2026-06-13)<br>LLD 22 | `ondemand` scaling governor |

> **測定スクリプトの変更について**
>
> - **Intel Core i5-13500:** Rustのコンパイルでローカルのリンカを用いるよう，`-fuse-ld=lld-22`から`-fuse-ld=lld`に変更している．
>
> - **Apple M2 (macOS):** システム情報の取得を，Linux向けのコマンドからMacのコマンドに変更している (`sysctl`, `vm_stat`, `sw_vers`)．また，LTOを有効化した際でもコンパイルが通るようにするため，Cargoのコンパイルオプションを一部変更している．

</details>


## Copyright

このソフトウェアは以下のコードを使用しています．

- [hts_engine API](https://hts-engine.sourceforge.net).
  - Copyright (c) 2001-2014 Nagoya Institute of Technology Department of Computer Science
  - Copyright (c) 2001-2008 Tokyo Institute of Technology Interdisciplinary Graduate School of Science and Engineering

## ライセンス

BSD-3-Clause
