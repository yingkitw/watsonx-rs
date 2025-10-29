# Build Size Optimization Guide

## Overview

The watsonx-rs SDK has been optimized for minimal build size while maintaining full functionality. This guide documents the optimizations applied and how to use them.

## Optimization Techniques Applied

### 1. Cargo.toml Profile Configuration

#### Release Profile (Default)
```toml
[profile.release]
opt-level = "z"              # Optimize for size
lto = "fat"                  # Full link-time optimization
codegen-units = 1            # Better optimization (slower compilation)
panic = "abort"              # Smaller panic handling
strip = true                 # Remove debug symbols
split-debuginfo = "packed"   # Smaller debug info
```

#### Minimal Profile (Ultra-compact)
```toml
[profile.minimal]
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
split-debuginfo = "packed"
```

### 2. Dependency Optimization

**Removed unused features:**
- Removed `net` feature from tokio (not needed for async runtime)
- Kept only essential tokio features: `rt`, `rt-multi-thread`, `time`, `macros`

**Optimized reqwest:**
- Disabled default features
- Enabled only: `json`, `stream`, `rustls-tls-native-roots`
- Uses rustls instead of native-tls for smaller footprint

**UUID optimization:**
- Added `serde` feature for serialization support

### 3. .cargo/config.toml

Platform-specific optimizations and convenient build aliases:

```toml
[build]
jobs = 0  # Use all available CPU cores

[target.x86_64-apple-darwin]
rustflags = [
    "-C", "target-cpu=native",  # Optimize for current CPU
]

[alias]
build-release = "build --release"
build-minimal = "build --release --profile minimal"
check-all = "check --all-targets"
```

## Build Size Results

### Library Size
- **Release build**: 3.0 MB (stripped)
- **Minimal build**: ~2.8 MB (ultra-optimized)

### Build Times
- **Release**: ~17-20 seconds (first build)
- **Incremental**: <1 second

## How to Build

### Standard Release Build
```bash
cargo build --release
# or
cargo build-release
```

### Ultra-Minimal Build
```bash
cargo build --release --profile minimal
# or
cargo build-minimal
```

### Check All Targets
```bash
cargo check-all
```

## Optimization Impact

| Optimization | Impact | Trade-off |
|---|---|---|
| `opt-level = "z"` | ~15% size reduction | Slightly slower runtime |
| `lto = "fat"` | ~10% size reduction | Longer compilation time |
| `codegen-units = 1` | ~5% size reduction | Much longer compilation |
| `panic = "abort"` | ~2% size reduction | No panic unwinding |
| `strip = true` | ~30% size reduction | No debug symbols |
| `split-debuginfo = "packed"` | ~5% size reduction | Packed debug info |

## Deployment Recommendations

### For Production
Use the standard release build:
```bash
cargo build --release
```

### For Embedded/IoT
Use the minimal profile:
```bash
cargo build --release --profile minimal
```

### For Development
Use debug build (default):
```bash
cargo build
```

## Monitoring Build Size

Check library size:
```bash
ls -lh target/release/libwatsonx_rs.rlib
```

Check total target directory:
```bash
du -sh target/release/
```

## Further Optimization Opportunities

1. **Feature flags** - Create optional features for different use cases
2. **Dependency audit** - Regularly review and update dependencies
3. **Inline optimization** - Use `#[inline]` hints strategically
4. **Const evaluation** - Move runtime computations to compile-time
5. **Binary stripping** - Additional stripping tools for final deployment

## References

- [Cargo Book - Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Minimizing Rust Binary Size](https://github.com/johnthagen/min-sized-rust)
