# Performance Guidelines

## Overview

This document defines performance guidelines for Rust applications following Microsoft's recommendations. Focus on identifying hot paths, measuring before optimizing, and optimizing for throughput.

## Identify Hot Paths (M-HOTPATH)

Early in development, determine if your application is performance-critical:

### Assessment Questions

1. Is this a user-facing application with responsiveness requirements?
2. Does it process large amounts of data?
3. Is it called frequently in tight loops?
4. Are there latency requirements (< 100ms response)?

### For Performance-Critical Code

1. **Identify hot paths** - Code executed frequently or with strict timing
2. **Create benchmarks** - Measure baseline performance
3. **Profile regularly** - Use profiling tools during development
4. **Document hot paths** - Mark performance-sensitive areas

## Benchmarking

### Using Criterion

```rust
// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_app::data_processor::DataProcessor;

fn benchmark_processing(c: &mut Criterion) {
    let processor = DataProcessor::new();
    let data = generate_test_data(1000);

    c.bench_function("process_1000_items", |b| {
        b.iter(|| {
            processor.process(black_box(&data))
        })
    });
}

fn benchmark_serialization(c: &mut Criterion) {
    let data = generate_large_struct();

    let mut group = c.benchmark_group("serialization");

    group.bench_function("json", |b| {
        b.iter(|| serde_json::to_string(black_box(&data)))
    });

    group.bench_function("bincode", |b| {
        b.iter(|| bincode::serialize(black_box(&data)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_processing, benchmark_serialization);
criterion_main!(benches);
```

### Cargo.toml Configuration

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmark"
harness = false

[profile.bench]
debug = 1  # Enable symbols for profiling
```

### Using Divan (Alternative)

```rust
use divan::Bencher;

#[divan::bench]
fn process_items(bencher: Bencher) {
    let data = generate_test_data(1000);

    bencher.bench(|| {
        process(&data)
    });
}

fn main() {
    divan::main();
}
```

## Throughput Optimization (M-THROUGHPUT)

Optimize for items processed per CPU cycle:

### Batch Processing

```rust
// BAD: Processing one item at a time
for item in items {
    process_item(item).await;  // Overhead per item
}

// GOOD: Batch processing
let chunks: Vec<_> = items.chunks(100).collect();
for chunk in chunks {
    process_batch(chunk).await;  // Amortize overhead
}
```

### Parallel Processing

```rust
use rayon::prelude::*;

// BAD: Sequential processing
let results: Vec<_> = items.iter().map(process).collect();

// GOOD: Parallel processing (for CPU-bound work)
let results: Vec<_> = items.par_iter().map(process).collect();
```

### Async Batching

```rust
use futures::stream::{self, StreamExt};

// Process items concurrently with bounded parallelism
let results: Vec<_> = stream::iter(items)
    .map(|item| async move { process_async(item).await })
    .buffer_unordered(10)  // Process up to 10 concurrently
    .collect()
    .await;
```

## Common Performance Issues

### String Allocations

```rust
// BAD: Frequent string allocations
fn build_message(parts: &[&str]) -> String {
    let mut result = String::new();
    for part in parts {
        result = result + part;  // New allocation each iteration!
    }
    result
}

// GOOD: Pre-allocate capacity
fn build_message(parts: &[&str]) -> String {
    let total_len: usize = parts.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(total_len);
    for part in parts {
        result.push_str(part);
    }
    result
}

// BETTER: Use join or format macros appropriately
fn build_message(parts: &[&str]) -> String {
    parts.join("")
}
```

### Unnecessary Cloning

```rust
// BAD: Clone when borrow would work
fn process(data: &Data) {
    let cloned = data.clone();  // Unnecessary!
    do_something(&cloned);
}

// GOOD: Use references
fn process(data: &Data) {
    do_something(data);
}

// When you need ownership, take ownership
fn consume(data: Data) {  // Takes ownership
    // ...
}
```

### Collection Reallocation

```rust
// BAD: Growing Vec without capacity hint
let mut results = Vec::new();
for i in 0..10000 {
    results.push(compute(i));  // Multiple reallocations
}

// GOOD: Pre-allocate when size is known
let mut results = Vec::with_capacity(10000);
for i in 0..10000 {
    results.push(compute(i));
}

// BETTER: Use iterator
let results: Vec<_> = (0..10000).map(compute).collect();
```

### HashMap Performance

```rust
use std::collections::HashMap;
use rustc_hash::FxHashMap;  // Faster for non-cryptographic use

// Default HashMap uses SipHash (cryptographically secure but slower)
let map: HashMap<String, i32> = HashMap::new();

// FxHashMap is faster when you don't need DoS resistance
let map: FxHashMap<String, i32> = FxHashMap::default();
```

## COSMIC App Performance

### Minimize View Rebuilds

```rust
impl Application for App {
    fn view(&self) -> Element<Message> {
        // GOOD: Only rebuild what changed
        let content = match self.current_page {
            Page::Home => self.view_home(),
            Page::Settings => self.view_settings(),
        };

        // BAD: Rebuilding entire nav bar on every view
        // let nav = self.build_entire_nav();

        content.into()
    }
}
```

### Efficient List Rendering

```rust
fn view_list(&self) -> Element<Message> {
    // For large lists, consider lazy loading or virtualization
    let items: Vec<Element<Message>> = self.items
        .iter()
        .take(100)  // Limit visible items
        .map(|item| self.view_item(item))
        .collect();

    cosmic::widget::scrollable(
        cosmic::widget::column::with_children(items)
    )
    .into()
}
```

### Async Data Loading

```rust
// Load data asynchronously to keep UI responsive
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::LoadData => {
            Task::perform(
                async {
                    // Heavy computation off main thread
                    load_large_dataset().await
                },
                Message::DataLoaded,
            )
        }
        _ => Task::none(),
    }
}
```

## Profiling Tools

### Linux
- `perf` - System profiler
- `flamegraph` - Visualization
- `valgrind` / `massif` - Memory profiling

```bash
# Generate flamegraph
cargo install flamegraph
cargo flamegraph --bench benchmark
```

### Cross-Platform
- **Intel VTune** - Detailed CPU analysis
- **Superluminal** - Windows profiler
- **Tracy** - Frame profiler for real-time apps

### Memory Profiling

```bash
# Check for memory leaks with valgrind
cargo build --release
valgrind --leak-check=full ./target/release/my_app

# Heap profiling with DHAT
cargo install cargo-dhat
cargo dhat --bench benchmark
```

## Performance Checklist

**Before Optimizing:**
- [ ] Have benchmarks that prove there's a problem
- [ ] Profile to identify actual bottlenecks
- [ ] Understand the algorithmic complexity

**Hot Path Optimization:**
- [ ] Minimize allocations in loops
- [ ] Use pre-allocated buffers
- [ ] Prefer references over clones
- [ ] Use appropriate data structures

**Throughput:**
- [ ] Batch related operations
- [ ] Use parallel processing for CPU-bound work
- [ ] Use async for I/O-bound work
- [ ] Avoid work stealing for individual items

**COSMIC Apps:**
- [ ] Minimize view rebuilds
- [ ] Load data asynchronously
- [ ] Use lazy loading for large lists
- [ ] Profile with release builds

## Anti-Patterns

```rust
// DON'T: Premature optimization
fn process(data: &[u8]) {
    // Don't use unsafe "for performance" without benchmarks
    unsafe { ... }  // Is this actually faster? Measure!
}

// DON'T: Micro-optimize cold paths
fn init() {
    // Called once at startup - don't optimize this
    let config = Config::load();  // Fine if it takes 10ms
}

// DON'T: Hot spinning
loop {
    if has_work() {
        do_work();
    }
    // BAD: Spins CPU even when idle
}

// DO: Sleep when no work
loop {
    if let Some(work) = try_get_work() {
        do_work(work);
    } else {
        std::thread::sleep(Duration::from_millis(10));
    }
}
```

## References

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Divan Benchmarking](https://github.com/nvzqz/divan)
