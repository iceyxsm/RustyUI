# RustyUI Testing Guide

## System Requirements

**IMPORTANT**: Building RustyUI with all features requires significant system resources:
- Minimum 16GB RAM recommended
- 32GB RAM for comfortable development
- Sufficient virtual memory (paging file) configured

## Running Tests

### Core Tests (Lightweight)
```bash
# Test core functionality without heavy dependencies
cargo test --package rustyui-core --lib --features dev-ui -- --test-threads=1
```

### Interpreter Tests
```bash
# Test interpretation system
cargo test --package rustyui-interpreter --lib --features dev-ui
```

### CLI Tests
```bash
# Test CLI functionality
cargo test --package rustyui-cli --lib
```

### All Tests (Memory Intensive)
```bash
# Run all tests - requires significant memory
cargo test --all --features dev-ui
```

## Running Examples

### Basic Examples (No GUI)
```bash
# Simple interpretation test
cargo run --example basic_interpretation --features dev-ui

# File watching demo
cargo run --example file_watching_demo --features dev-ui

# State preservation demo
cargo run --example state_preservation_demo --features dev-ui
```

### GUI Examples (Memory Intensive)
```bash
# Simple hot reload demo with egui
cargo run --example simple_hot_reload_demo --features "dev-ui,egui-adapter"

# Comprehensive demo
cargo run --example comprehensive_demo --features "dev-ui,egui-adapter"

# Cross-platform demo
cargo run --example cross_platform_demo --features "dev-ui,egui-adapter"
```

## Memory Issues

If you encounter memory allocation errors:

1. **Increase Virtual Memory (Windows)**:
   - System Properties → Advanced → Performance Settings → Advanced → Virtual Memory
   - Set custom size: Initial 16GB, Maximum 32GB

2. **Close Other Applications**: Free up RAM before building

3. **Build Incrementally**:
   ```bash
   # Build one package at a time
   cargo build --package rustyui-core --features dev-ui
   cargo build --package rustyui-interpreter --features dev-ui
   cargo build --package rustyui-adapters --features dev-ui
   cargo build --package rustyui-cli
   ```

4. **Use Release Mode** (uses less memory during compilation):
   ```bash
   cargo build --release --features dev-ui
   ```

## Property-Based Tests

Property-based tests validate correctness properties:

```bash
# Run property tests for dual-mode operation
cargo test --package rustyui-core property_tests_dual_mode --features dev-ui

# Run property tests for state preservation
cargo test --package rustyui-core property_tests_state_preservation --features dev-ui

# Run property tests for performance bounds
cargo test --package rustyui-core property_tests_performance_bounds --features dev-ui
```

## CI/CD Considerations

For CI/CD pipelines, use machines with:
- At least 16GB RAM
- SSD storage for faster builds
- Caching of dependencies to reduce build times

Example GitHub Actions configuration:
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --all --features dev-ui
```

## Known Issues

1. **Memory Allocation Failures**: Cranelift JIT compilation requires significant memory. If tests fail with memory errors, increase virtual memory or use a machine with more RAM.

2. **Windows Paging File**: Error "The paging file is too small" indicates insufficient virtual memory. Increase paging file size in Windows settings.

3. **Compilation Timeouts**: Large dependency trees (egui, cranelift) can take 5-10 minutes to compile on first build. Use `--release` mode or incremental builds.

## Test Coverage

Current test coverage:
-  Dual-mode engine operation
-  Runtime interpretation (Rhai, AST, JIT)
-  State preservation
-  File change detection
-  Error recovery
-  Cross-platform compatibility
-  Performance bounds
-  Zero-overhead production builds
-  Framework adapters (egui, iced, slint, tauri)

## Next Steps

1. Run tests on a machine with adequate resources
2. Set up CI/CD with proper resource allocation
3. Create integration tests for end-to-end workflows
4. Add benchmarks for performance validation
