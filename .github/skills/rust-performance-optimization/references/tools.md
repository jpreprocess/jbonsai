# Profiling Tools Reference

Quick command reference for cargo bench, cargo asm, and flamegraph.

## cargo bench (Rust Nightly Benchmarks)

**Purpose**: Measure iteration performance using unstable bench feature.

### Installation

Already configured in Cargo.toml with `#![feature(test)]` in benches/.

### Basic Usage

```bash
# Run all benchmarks
cargo bench --bench bonsais

# Output format
# test bonsai ... bench: 123,456 ns/iter

# Run specific benchmark
cargo bench --bench bonsais bonsai
```

### With Profile Optimization

```bash
# Bench profile (most optimized, includes debug symbols)
cargo bench --bench bonsais --profile=bench

# Compare to dev build
cargo bench --bench bonsais --profile=dev  # Much slower, for validation only
```

### Warmup & Iterations

The Bencher struct auto-runs:
1. Initial iterations to warm CPU cache
2. 25 measured iterations (default, configurable)
3. Reports mean + variance

### Output Interpretation

```
test bonsai ... bench: 123,456 ns/iter (+/- 2,345 ns)
               ↑ Mean time              ↑ Standard deviation
```

**Good result**: < ±5% variance (stable)  
**Poor result**: > ±10% variance (CPU throttling or background processes)

**Pro tip**: Close other apps and disable CPU frequency scaling for stable benchmarks.

---

## cargo asm (Assembly Inspection)

**Purpose**: View generated machine code to identify panic checks and optimization opportunities.

### Installation

```bash
cargo install cargo-show-asm
```

### Basic Usage

```bash
# View assembly for a function
cargo asm --lib module::function_name

# Example from jbonsai
cargo asm --profile=bench --lib mlpg_adjust::mlpg::ldl_factorization
```

### Preventing Inlining

Use `#[inline(never)]` to ensure accurate assembly inspection:

```rust
#[inline(never)]
fn target_function() {
    // Your hot function code
}
```

**Why it matters**: Without `#[inline(never)]`, the compiler may inline the function, and `cargo asm` will show the caller's context instead of the actual function body. This attribute is essential for accurately comparing before/after assembly and identifying optimization opportunities. See [src/vocoder/mlsa.rs](../../../src/vocoder/mlsa.rs#L127) for an example of test-only usage.

### Reading Output

Output shows Intel x86_64 syntax (default). Each line:

```
0x0000: 48 89 d8           mov %rbx, %rax        # Move register
0x0003: 83 c0 01           add $0x1, %eax        # Add immediate
0x0006: 48 83 ff 00        cmp $0x0, %rdi        # Compare
0x000a: 74 10              je 0x1c               # Jump if equal
```

### Identifying Panic Checks

Search for patterns:

```
panic_bounds_check      # Function call to panic handler
cmp                     # Comparison instruction
je / jne / jl / jge     # Conditional jump (branch)
```

**Example - bounds check**:
```asm
48 3b 47 00             cmp 0x0(%rdi), %rax
0f 87 XX XX XX XX       ja panic_bounds_check
```
(JA = "Jump if Above", triggers panic if index >= length)

### Counting Panic Checks

```bash
# Count panic_bounds_check calls
cargo asm --lib module::func | grep -c panic_bounds_check

# Before optimization: 18
# After optimization:  2
```

### Platform-Specific Assembly

```bash
# ARM64 assembly (macOS/Linux ARM)
cargo asm --target aarch64-unknown-linux-gnu --lib module::func

# x86 without AVX2 (baseline)
RUSTFLAGS="-C target-feature=" cargo asm --lib module::func

# x86 with AVX2+FMA
RUSTFLAGS="-C target-feature=+avx2,+fma" cargo asm --lib module::func
```

---

## cargo flamegraph (Call Stack Profiling)

**Purpose**: Identify where the program spends time across all call stacks.

### Installation

```bash
cargo install flamegraph

# Also requires perf (Linux) or Instruments (macOS)
# Linux: sudo apt install linux-tools-generic
# macOS: Install Xcode command line tools
```

### Basic Usage

```bash
# Generate flame graph (Linux)
cargo flamegraph --bench bonsais -- --bench

# Output: flamegraph.svg (open in web browser)
```

### Reading Flame Graphs

- **X-axis**: Time spent (wider = more time)
- **Y-axis**: Call stack depth (bottom = main, up = deeper calls)
- **Color**: Random (helps distinguish blocks)
- **Hotter colors in some versions**: Red = hot, blue = cold

**Typical structure**:
```
[bonsais-bench]
    ├─ [test_bonsai]
    │   ├─ engine.synthesize (70%)
    │   │   ├─ vocoder.mlsa_filter (30%)
    │   │   └─ mlpg_adjust.mlpg (40%)
    │   └─ model.interpolate (30%)
    └─ [framework overhead] (30%)
```

### Finding Optimization Targets

1. **Click on wide blocks** - these are hot functions
2. **Find functions with many branches** - good candidates for bounds-check reduction
3. **Look for repeated small functions** - inline opportunities

### macOS Profiling (Alternative)

If flamegraph doesn't work, use Instruments:

```bash
# Build in bench profile
cargo build --profile=bench --bench bonsais

# Profile with Instruments
xcrun xctrace record --template "System Trace" \
  --output /tmp/trace.trace \
  ./target/bench/bonsais --bench

# Open in Instruments.app
open /tmp/trace.trace
```

---

## Pro Tips

### 1. Disable Frequency Scaling (for stable benchmarks)

Linux:
```bash
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

macOS: Use Activity Monitor or disable using Energy Saver settings.

### 2. Isolate Benchmark Function

```bash
# Build only the benchmark binary (faster iteration)
cargo build --bench bonsais --profile=bench

# Run just one benchmark test multiple times
./target/bench/bonsais --bench --nightly bonsai
```

### 3. Compare Before/After Codegen

```bash
# Generate LLVM IR (intermediate representation)
RUSTFLAGS="--emit llvm-ir" cargo build --profile=bench --lib

# Find and diff LLVM files
diff target/release/deps/jbonsai-*.ll
```
