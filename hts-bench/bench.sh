#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SCRIPT_DIR"

rm -rf builds hts_engine_API || true
mkdir -p builds
mkdir -p "$SCRIPT_DIR/results"

echo "## CPU" > "$SCRIPT_DIR/results/system_info.txt"
lscpu >> "$SCRIPT_DIR/results/system_info.txt"
echo "## Memory" >> "$SCRIPT_DIR/results/system_info.txt"
sudo free -h >> "$SCRIPT_DIR/results/system_info.txt"
echo "## OS" >> "$SCRIPT_DIR/results/system_info.txt"
cat /etc/os-release >> "$SCRIPT_DIR/results/system_info.txt"
uname -rsv >> "$SCRIPT_DIR/results/system_info.txt"
echo "## Compiler Versions" >> "$SCRIPT_DIR/results/system_info.txt"
ldd --version | head -n 1 >> "$SCRIPT_DIR/results/system_info.txt"
clang-22 --version >> "$SCRIPT_DIR/results/system_info.txt"
rustc --version --verbose >> "$SCRIPT_DIR/results/system_info.txt"
echo "## Hyperfine Version" >> "$SCRIPT_DIR/results/system_info.txt"
hyperfine --version >> "$SCRIPT_DIR/results/system_info.txt"

git clone https://github.com/jpreprocess/hts_engine_API.git
cd hts_engine_API
git checkout 5ac9af390e45bfdf2869818634891f5d6da9a6bd

patch -p1 <<'EOF'
diff --git a/src/bin/hts_engine.c b/src/bin/hts_engine.c
index 532c381..a94587a 100644
--- a/src/bin/hts_engine.c
+++ b/src/bin/hts_engine.c
@@ -66,8 +66,8 @@ int main(void)
    HTS_Engine engine;
 
    /* Hardcoded Configuration Paths */
-   char *default_voice = "./tohoku-f01/tohoku-f01-neutral.htsvoice";
-   char *labfn         = "./bonsai_letter.lab";
+   char *default_voice = "models/tohoku-f01/tohoku-f01-neutral.htsvoice";
+   char *labfn         = "examples/constitution/constitution.lab";
 
    /* Fixed single-voice array container on the stack */
    char *fn_voices[1];
EOF

mkdir -p src/build
cd src/build

CC=clang-22 CXX=clang-22 cmake -DCMAKE_BUILD_TYPE=Release -DCPU_NATIVE=OFF ..
make -j1
cp bin/hts_engine "$SCRIPT_DIR/builds/hts_engine_default"

CC=clang-22 CXX=clang-22 cmake -DCMAKE_BUILD_TYPE=Release -DCPU_NATIVE=ON ..
make -j1
cp bin/hts_engine "$SCRIPT_DIR/builds/hts_engine_native"

cd "$PROJECT_ROOT"

RUSTFLAGS="-C codegen-units=1 -C linker-plugin-lto=off -C linker=clang-22 -C link-arg=-fuse-ld=lld-22" cargo build --example constitution --features=binary --release
cp target/release/examples/constitution "$SCRIPT_DIR/builds/jbonsai_default"

RUSTFLAGS="-C codegen-units=1 -C linker-plugin-lto=on -C linker=clang-22 -C link-arg=-fuse-ld=lld-22" cargo build --example constitution --features=binary --release
cp target/release/examples/constitution "$SCRIPT_DIR/builds/jbonsai_lto"

RUSTFLAGS="-C codegen-units=1 -C linker-plugin-lto=on -C linker=clang-22 -C link-arg=-fuse-ld=lld-22 -C target-cpu=native" cargo build --example constitution --features=binary --release
cp target/release/examples/constitution "$SCRIPT_DIR/builds/jbonsai_native"

"$SCRIPT_DIR/builds/hts_engine_native" > "$SCRIPT_DIR/results/hts_engine.raw"
"$SCRIPT_DIR/builds/jbonsai_native" > "$SCRIPT_DIR/results/jbonsai.raw"

cmp --silent "$SCRIPT_DIR/results/hts_engine.raw" "$SCRIPT_DIR/results/jbonsai.raw" && echo "Outputs match!" || echo "Output mismatch between HTS Engine and jbonsai!"

echo "## CPU Scaling Governors" >> "$SCRIPT_DIR/results/system_info.txt"
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | sort | uniq >> "$SCRIPT_DIR/results/system_info.txt"

hyperfine --warmup 5 --runs 25 \
    --export-json "$SCRIPT_DIR/results/benchmark.json" \
    "$SCRIPT_DIR/builds/hts_engine_default" \
    "$SCRIPT_DIR/builds/hts_engine_native" \
    "$SCRIPT_DIR/builds/jbonsai_default" \
    "$SCRIPT_DIR/builds/jbonsai_lto" \
    "$SCRIPT_DIR/builds/jbonsai_native"

echo "HTS Engine API commit: $(git -C "$SCRIPT_DIR/hts_engine_API" rev-parse HEAD)" > "$SCRIPT_DIR/results/hts_engine_commit.txt"
echo "JBonsai commit: $(git -C "$PROJECT_ROOT" rev-parse HEAD)" > "$SCRIPT_DIR/results/jbonsai_commit.txt"

tar -czvf "$SCRIPT_DIR/benchmark_results.tar.gz" -C "$SCRIPT_DIR/results" .
