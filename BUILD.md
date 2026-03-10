# RustyUI Build System

RustyUI uses a dual-mode build system that provides instant feedback during development and zero overhead in production.

## Build Modes

### Development Mode (with dev-ui feature)

```bash
# Enable development mode with runtime interpretation
cargo run --features dev-ui

# Check development build
cargo check --features dev-ui

# Build development binary
cargo build --features dev-ui
```

**Development mode includes:**
- Runtime interpretation with Rhai scripting
- File system change monitoring
- State preservation across hot reloads
- AST interpretation and JIT compilation
- Memory overhead: ~2MB

### Production Mode (default)

```bash
# Build optimized production binary
cargo build --release

# Run production binary (no dev features)
cargo run --release
```

**Production mode characteristics:**
- Zero runtime overhead compared to standard Rust
- All development features stripped at compile time
- Maximum optimization with LTO enabled
- Binary size equivalent to standard Rust applications

## Feature Flags

### Core Features

- `dev-ui`: Enables all development-time features
  - Runtime interpretation
  - File change monitoring  
  - State preservation
  - Hot reload capabilities

### Framework Features (in rustyui-adapters)

- `egui-adapter`: Support for egui framework
- `iced-adapter`: Support for iced framework  
- `slint-adapter`: Support for slint framework
- `tauri-adapter`: Support for tauri framework
- `all-adapters`: Enable all framework adapters

## Conditional Compilation

RustyUI uses extensive conditional compilation to ensure zero overhead in production:

```rust
// Development-only code
#[cfg(feature = "dev-ui")]
pub fn start_hot_reload(&mut self) -> Result<()> {
    // Hot reload implementation
}

// Production stub (zero overhead)
#[cfg(not(feature = "dev-ui"))]
pub fn start_hot_reload(&mut self) -> Result<()> {
    Ok(()) // No-op in production
}
```

## Build Profiles

### Development Profile

```toml
[profile.dev]
opt-level = 0
debug = true
incremental = true
```

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## CLI Build Commands

### Using RustyUI CLI

```bash
# Initialize existing project
rustyui init --framework egui

# Create new project
rustyui new my-app --framework egui

# Start development mode
rustyui dev
```

### Manual Cargo Commands

```bash
# Development workflow
cargo run --features dev-ui

# Production build
cargo build --release

# Cross-compilation
cargo build --release --target x86_64-pc-windows-msvc
```

## Performance Characteristics

### Development Mode
- Interpretation overhead: ~14% slower than native Rust
- Memory overhead: ~2MB for development features
- Hot reload time: <50ms for typical UI changes
- File change detection: <50ms

### Production Mode
- Runtime performance: Identical to native Rust
- Memory overhead: 0 bytes
- Binary size: Equivalent to standard Rust applications
- Startup time: No additional overhead

## Verification

### Check Build Mode

```rust
use rustyui_core::BuildInfo;

fn main() {
    println!("Build mode: {}", BuildInfo::build_mode());
    println!("Has dev features: {}", BuildInfo::has_dev_features());
    println!("Memory overhead: {} bytes", BuildInfo::dev_memory_overhead_bytes());
}
```

### Verify Zero Overhead

```bash
# Compare binary sizes
cargo build --release
ls -la target/release/my-app

# Compare with standard Rust app of similar complexity
# Should be nearly identical in size and performance
```

## Troubleshooting

### Development Mode Not Working

1. Ensure `dev-ui` feature is enabled:
   ```bash
   cargo run --features dev-ui
   ```

2. Check that rustyui.toml exists:
   ```bash
   rustyui init
   ```

### Production Build Too Large

1. Verify dev features are disabled:
   ```bash
   cargo build --release
   # Should NOT include --features dev-ui
   ```

2. Check that LTO is enabled in Cargo.toml:
   ```toml
   [profile.release]
   lto = true
   ```

### Performance Issues

1. Development mode is expected to be ~14% slower
2. Production mode should match native Rust performance
3. Use `cargo build --release` for production builds