#![feature(test)]

use jbonsai::engine::Engine;
use test::Bencher;

extern crate test;

const MODEL_NITECH_ATR503: &str =
    "models/hts_voice_nitech_jp_atr503_m001-1.05/nitech_jp_atr503_m001.htsvoice";

#[bench]
fn bonsai(bencher: &mut Bencher) {
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

    let engine = Engine::load(&[MODEL_NITECH_ATR503]).unwrap();

    bencher.iter(|| {
        engine.synthesize(&lines).unwrap();
    });
}

#[bench]
fn is_bonsai(bencher: &mut Bencher) {
    // これ,名詞,代名詞,一般,*,*,*,これ,コレ,コレ,0/2,C3,-1
    // は,助詞,係助詞,*,*,*,*,は,ハ,ワ,0/1,動詞%F2/形容詞%F2/名詞%F1,1
    // 盆栽,名詞,一般,*,*,*,*,盆栽,ボンサイ,ボンサイ,5/4,C2,0
    // です,助動詞,*,*,*,特殊・デス,基本形,です,デス,デス’,1/2,動詞%F1/形容詞%F2/名詞%F2@1,1
    // か,助詞,副助詞／並立助詞／終助詞,*,*,*,*,か,カ,カ,0/1,動詞%F2/形容詞%F2/名詞%F1,1
    // ？,記号,一般,*,*,*,*,？,？,？,0/0,*,0
    let lines = [
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

    let engine = Engine::load(&[MODEL_NITECH_ATR503]).unwrap();

    bencher.iter(|| {
        engine.synthesize(&lines).unwrap();
    });
}

#[bench]
fn bonsai_letter(bencher: &mut Bencher) {
    // 彼岸過迄（夏目漱石）より

    // 手紙,名詞,一般,*,*,*,*,手紙,テガミ,テガミ,0/3,C2,-1
    // の,助詞,連体化,*,*,*,*,の,ノ,ノ,0/1,動詞%F2/形容詞%F1,1
    // 末,名詞,非自立,副詞可能,*,*,*,末,スエ,スエ,2/2,C4,0
    // 段,名詞,一般,*,*,*,*,段,ダン,ダン,1/2,C3,1
    // に,助詞,格助詞,一般,*,*,*,に,ニ,ニ,0/1,動詞%F5/形容詞%F1/名詞%F1,1
    // は,助詞,係助詞,*,*,*,*,は,ハ,ワ,0/1,動詞%F2/形容詞%F2/名詞%F1,1
    // 盆栽,名詞,一般,*,*,*,*,盆栽,ボンサイ,ボンサイ,0/4,C2,0
    // の,助詞,連体化,*,*,*,*,の,ノ,ノ,0/1,動詞%F2/形容詞%F1,1
    // 事,名詞,非自立,一般,*,*,*,事,コト,コト,2/2,C3,0
    // が,助詞,格助詞,一般,*,*,*,が,ガ,ガ,0/1,名詞%F1,1
    // 書い,動詞,自立,*,*,五段・カ行イ音便,連用タ接続,書い,カイ,カイ,1/2,*,0
    // て,助詞,接続助詞,*,*,*,*,て,テ,テ,0/1,動詞%F1/形容詞%F1/名詞%F5,1
    // あっ,動詞,非自立,*,*,五段・ラ行,連用タ接続,あっ,アッ,アッ,1/2,*,0
    // た,助動詞,*,*,*,特殊・タ,基本形,た,タ,タ,0/1,動詞%F2@1/形容詞%F4@-2,1
    // 。,記号,句点,*,*,*,*,。,、,、,0/0,*,0
    let lines = [
        "xx^xx-sil+t=e/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_4%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:6_24/K:1+6-24",
        "xx^sil-t+e=g/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:23+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "sil^t-e+g=a/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:23+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "t^e-g+a=m/A:-2+2+3/B:xx-xx_xx/C:02_xx+xx/D:23+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "e^g-a+m=i/A:-2+2+3/B:xx-xx_xx/C:02_xx+xx/D:23+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "g^a-m+i=n/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:23+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^m-i+n=o/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:23+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "m^i-n+o=s/A:0+4+1/B:02-xx_xx/C:23_xx+xx/D:22+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "i^n-o+s=u/A:0+4+1/B:02-xx_xx/C:23_xx+xx/D:22+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_6|1_24/G:6_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "n^o-s+u=e/A:-1+1+6/B:23-xx_xx/C:22_xx+xx/D:02+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "o^s-u+e=d/A:-1+1+6/B:23-xx_xx/C:22_xx+xx/D:02+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "s^u-e+d=a/A:0+2+5/B:23-xx_xx/C:22_xx+xx/D:02+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "u^e-d+a=N/A:1+3+4/B:22-xx_xx/C:02_xx+xx/D:13+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "e^d-a+N=n/A:1+3+4/B:22-xx_xx/C:02_xx+xx/D:13+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "d^a-N+n=i/A:2+4+3/B:22-xx_xx/C:02_xx+xx/D:13+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^N-n+i=w/A:3+5+2/B:02-xx_xx/C:13_xx+xx/D:24+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "N^n-i+w=a/A:3+5+2/B:02-xx_xx/C:13_xx+xx/D:24+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "n^i-w+a=b/A:4+6+1/B:13-xx_xx/C:24_xx+xx/D:02+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "i^w-a+b=o/A:4+6+1/B:13-xx_xx/C:24_xx+xx/D:02+xx_xx/E:4_4!0_xx-1/F:6_2#0_xx@2_5|5_20/G:5_5%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "w^a-b+o=N/A:-4+1+5/B:24-xx_xx/C:02_xx+xx/D:23+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^b-o+N=s/A:-4+1+5/B:24-xx_xx/C:02_xx+xx/D:23+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "b^o-N+s=a/A:-3+2+4/B:24-xx_xx/C:02_xx+xx/D:23+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "o^N-s+a=i/A:-2+3+3/B:24-xx_xx/C:02_xx+xx/D:23+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "N^s-a+i=n/A:-2+3+3/B:24-xx_xx/C:02_xx+xx/D:23+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "s^a-i+n=o/A:-1+4+2/B:24-xx_xx/C:02_xx+xx/D:23+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^i-n+o=k/A:0+5+1/B:02-xx_xx/C:23_xx+xx/D:22+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "i^n-o+k=o/A:0+5+1/B:02-xx_xx/C:23_xx+xx/D:22+xx_xx/E:6_2!0_xx-1/F:5_5#0_xx@3_4|11_14/G:3_2%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "n^o-k+o=t/A:-1+1+3/B:23-xx_xx/C:22_xx+xx/D:13+xx_xx/E:5_5!0_xx-1/F:3_2#0_xx@4_3|16_9/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "o^k-o+t=o/A:-1+1+3/B:23-xx_xx/C:22_xx+xx/D:13+xx_xx/E:5_5!0_xx-1/F:3_2#0_xx@4_3|16_9/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "k^o-t+o=g/A:0+2+2/B:23-xx_xx/C:22_xx+xx/D:13+xx_xx/E:5_5!0_xx-1/F:3_2#0_xx@4_3|16_9/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "o^t-o+g=a/A:0+2+2/B:23-xx_xx/C:22_xx+xx/D:13+xx_xx/E:5_5!0_xx-1/F:3_2#0_xx@4_3|16_9/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "t^o-g+a=k/A:1+3+1/B:22-xx_xx/C:13_xx+xx/D:20+1_1/E:5_5!0_xx-1/F:3_2#0_xx@4_3|16_9/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "o^g-a+k=a/A:1+3+1/B:22-xx_xx/C:13_xx+xx/D:20+1_1/E:5_5!0_xx-1/F:3_2#0_xx@4_3|16_9/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "g^a-k+a=i/A:0+1+3/B:13-xx_xx/C:20_1+1/D:12+xx_xx/E:3_2!0_xx-1/F:3_1#0_xx@5_2|19_6/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^k-a+i=t/A:0+1+3/B:13-xx_xx/C:20_1+1/D:12+xx_xx/E:3_2!0_xx-1/F:3_1#0_xx@5_2|19_6/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "k^a-i+t=e/A:1+2+2/B:13-xx_xx/C:20_1+1/D:12+xx_xx/E:3_2!0_xx-1/F:3_1#0_xx@5_2|19_6/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^i-t+e=a/A:2+3+1/B:20-1_1/C:12_xx+xx/D:17+1_1/E:3_2!0_xx-1/F:3_1#0_xx@5_2|19_6/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "i^t-e+a=cl/A:2+3+1/B:20-1_1/C:12_xx+xx/D:17+1_1/E:3_2!0_xx-1/F:3_1#0_xx@5_2|19_6/G:3_1%0_xx_1/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "t^e-a+cl=t/A:0+1+3/B:12-xx_xx/C:17_1+1/D:10+7_2/E:3_1!0_xx-1/F:3_1#0_xx@6_1|22_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "e^a-cl+t=a/A:1+2+2/B:12-xx_xx/C:17_1+1/D:10+7_2/E:3_1!0_xx-1/F:3_1#0_xx@6_1|22_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "a^cl-t+a=sil/A:2+3+1/B:17-1_1/C:10_7+2/D:xx+xx_xx/E:3_1!0_xx-1/F:3_1#0_xx@6_1|22_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "cl^t-a+sil=xx/A:2+3+1/B:17-1_1/C:10_7+2/D:xx+xx_xx/E:3_1!0_xx-1/F:3_1#0_xx@6_1|22_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:6-24@1+1&1-6|1+24/J:xx_xx/K:1+6-24",
        "t^a-sil+xx=xx/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:3_1!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:6_24/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+6-24",
    ];

    let engine = Engine::load(&[MODEL_NITECH_ATR503]).unwrap();

    bencher.iter(|| {
        engine.synthesize(&lines).unwrap();
    });
}
