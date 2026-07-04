---
name: rust-performance-optimization
description: 'Optimize Rust code performance using safe patterns and profiling tools. Use when: analyzing hot loops, reducing bounds-check panics, comparing before/after implementations, measuring improvements with cargo bench and cargo asm.'
argument-hint: 'What function or module needs optimization? Example: jbonsai::vocoder::mlsa::fir'
---

# Rust Performance Optimization Workflow

A systematic approach to safe performance improvements using profiling tools, assembly inspection, and benchmarking for jbonsai.

## When to Use

- Analyzing and optimizing hot code paths
- Comparing performance before and after refactoring
- Reducing redundant bounds checks or panic branches
- Applying SIMD-friendly memory layouts
- Validating that optimizations don't break correctness

## Key Principles

1. **No unsafe code**: Optimize within safe Rust only
2. **Before/after comparison**: Always measure improvements with concrete data
3. **Correctness first**: Validate every refactoring with `cargo test`
4. **Data-driven decisions**: Use profiling to identify actual bottlenecks

## Common Safe Optimization Patterns

### 1. Move Panic Checks Outside Loops

**Problem**: Repeated bounds checks in tight loops generate panic-checking code.

```rust
// ❌ Before: bounds check in loop
for t in 0..length {
    for i in 1..width {
        result[t][i] = arr[t - i][i] * factor;  // Bounds check each iteration
    }
}

// ✅ After: use .min() to prove bounds at compile time
for t in 0..length {
    for i in 1..width.min(t + 1) {  // Compiler proves: i <= t, so t - i >= 0
        result[t][i] = arr[t - i][i] * factor;  // No panic check needed
    }
}
```

### 2. Flatten 2D Arrays + chunks_exact

**Problem**: Nested `Vec<Vec<f64>>` requires multi-level indexing and bounds checks.

**Solution**: Flatten to `Vec<f64>` with `chunks_exact()` for cache efficiency and fewer panic branches.

```rust
// ❌ Before: nested vectors
let mut wuw: Vec<Vec<f64>> = vec![vec![0.0; width]; length];
for t in 0..length {
    for i in 1..width {
        wuw[t][i] = /* ... */;
    }
}

// ✅ After: flat with chunks_exact
let mut wuw: Vec<f64> = vec![0.0; width * length];
for t in 0..length {
    let row = &mut wuw[t * width..(t + 1) * width];
    for i in 1..width {
        row[i] = /* ... */;
    }
}

// Or with split_at_mut for non-contiguous slices
let (left, right) = wuw.split_at_mut(t * width);
let row = &mut right[..width];
```

### 3. Pre-slice Ranges

**Problem**: Computing array ranges dynamically in hot loops.

```rust
// ❌ Before: range computation each iteration
for item in items {
    let slice = &data[item.start..item.end];
    process(slice);
}

// ✅ After: pre-slice into contiguous buffer
let slice = &data[start..end];
for chunk in slice.chunks_exact(chunk_size) {
    process(chunk);
}
```

## Performance Analysis Workflow

### Step 1: Establish Baseline Benchmark

```bash
# Run the existing benchmark for your hot function
cargo bench --bench bonsais

# Output shows: time per iteration
# Example: test bonsai ... bench: 123,456 ns/iter
```

**Record the baseline number** for later comparison.

### Step 2: Inspect Assembly

Use `cargo asm` (from `cargo-show-asm`) to see generated code and identify panic branches. See [cargo asm reference](./references/tools.md#cargo-asm-assembly-inspection) for detailed usage.

**Key inspection points**:
- Count panic-check calls per loop iteration
- Look for redundant register computations
- Identify memory access patterns

### Step 3: Implement Optimization

Apply one of the [Common Safe Optimization Patterns](#common-safe-optimization-patterns):

- Move panic checks outside loops with `.min()` guards
- Flatten 2D arrays to 1D `Vec` + `chunks_exact()`
- Pre-slice ranges to avoid dynamic computation
- Use `split_at_mut()` for non-overlapping mutable borrows

### Step 4: Validate Correctness

```bash
# Run full test suite to ensure refactoring is sound
cargo test --lib
```

**All tests must pass** before measuring performance.

### Step 5: Measure Improvement

```bash
# Run benchmark again with optimized code
cargo bench --bench bonsais

# Compare the number (typically shown as ns/iter)
# Calculate: (baseline - optimized) / baseline * 100 = % improvement
```

**Example**:
- Before: 123,456 ns/iter
- After:  98,765 ns/iter
- Improvement: ~20% faster

### Step 6: Advanced Profiling (Optional)

For deeper analysis, install and use `cargo flamegraph`:

```bash
# Install flamegraph (requires perf on Linux, instruments on macOS)
cargo install flamegraph

# Generate flame graph (Linux: requires sudo)
cargo flamegraph --bench bonsais -- --bench

# Output: flamegraph.svg (open in browser)
# Shows call stack frequency, identifies true bottlenecks
```

**On macOS**, use Instruments.app or Swift profiler instead:
```bash
# Profile with macOS instruments (if available)
cargo build --profile=bench --bench bonsais
xcrun xctrace record --template "System Trace" \
  ./target/bench/deps/bonsais-* --bench
```

## Platform-Specific Considerations

### x86_64

Use SIMD-friendly layouts (contiguous arrays) to benefit from:
- AVX2 vectorization
- Cache prefetching
- Instruction-level parallelism

Enable feature flags for CI:
```bash
RUSTFLAGS="-C target-feature=+avx2,+fma" cargo bench
```

### aarch64 / ARM

SIMD benefits from:
- NEON vectorization
- Flattened data structures
- Fewer branches (predication is expensive)

### macOS (Apple Silicon)

- SIMD is efficient but memory layout matters more than x86_64
- Flatten arrays for better cache behavior
- Test with and without `target-cpu=native`

## Checklist: Safe Optimization

- [ ] Baseline benchmark recorded and documented
- [ ] Assembly inspected (`cargo asm`)
- [ ] Optimization implemented (using safe patterns only)
- [ ] `cargo test` passes 100%
- [ ] Performance improvement measured and > 5% (or justified)
- [ ] Code reviewed for correctness and maintainability
- [ ] Commit message: note % improvement and tool used (e.g., "perf: reduce bounds checks 15%")

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/) — in-depth optimization guide
- [cargo-show-asm](https://github.com/pacak/cargo-show-asm) — assembly inspection
- [cargo flamegraph](https://www.brendangregg.com/flamegraphs.html) — profiling methodology
