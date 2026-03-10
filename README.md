# RustyUI

Revolutionary dual-mode UI development system for Rust that provides instant feedback during development through runtime interpretation while maintaining zero overhead in production builds.

## Overview

RustyUI transforms Rust UI development by implementing a dual-mode architecture inspired by Flutter's JIT/AOT model and modern game engine live coding systems. Edit your UI code and see changes instantly without compilation delays, then deploy with full native Rust performance.

## Key Features

- **Instant UI Updates**: 0ms compilation time during development through runtime interpretation
- **Zero Production Overhead**: Full native Rust performance with all development features stripped
- **Framework Agnostic**: Works with egui, iced, slint, tauri, and any Rust UI framework
- **State Preservation**: Application state maintained across code changes
- **Cross-Platform**: Windows, macOS, and Linux support with platform optimizations
- **Safe Execution**: Sandboxed runtime interpretation with security limits

## Architecture

### Development Mode
```bash
cargo run --features dev-ui
```
- Runtime interpretation using Rhai scripting, AST parsing, or Cranelift JIT
- File watching with 50ms change detection
- State preservation across interpretation cycles
- Memory overhead under 50MB

### Production Mode
```bash
cargo build --release
```
- Standard Rust compilation with conditional feature stripping
- Identical performance to native Rust applications
- Zero runtime interpretation overhead
- Full compiler optimizations and security

## Quick Start

### Installation
```bash
cargo install rustyui
```

### Initialize Project
```bash
# New project
rustyui new my-app --framework egui

# Existing project
cd my-existing-project
rustyui init --framework egui
```

### Development Workflow
```bash
# Start development mode with instant updates
cargo run --features dev-ui

# Edit your UI code - see changes instantly
# No compilation required during development

# Build for production with zero overhead
cargo build --release
```

## Supported UI Frameworks

- **egui**: Immediate mode GUI with 13M+ downloads
- **iced**: Elm-inspired retained mode architecture
- **slint**: Native GUI with OpenGL renderer
- **tauri**: Web-based desktop applications
- **Custom**: Generic adapter for any Rust UI framework

## Performance Targets

### Development Mode
- **Rhai Scripts**: 0ms interpretation time
- **AST Interpretation**: Under 5ms
- **JIT Compilation**: Under 100ms (Cranelift)
- **Change Detection**: Under 50ms
- **Memory Overhead**: Under 50MB

### Production Mode
- **Performance**: 100% native Rust speed
- **Binary Size**: Equivalent to standard Rust builds
- **Memory Usage**: Zero interpretation overhead
- **Security**: Full Rust safety guarantees

## Runtime Interpretation Strategies

### Rhai Scripting Engine
- Rust-like syntax for UI logic
- Instant execution with 0ms compilation
- Sandboxed environment with resource limits
- Production-ready with 2x Python performance

### AST Interpretation
- Direct Rust syntax parsing using syn crate
- Near-instant interpretation under 5ms
- Full Rust language feature support
- Type-safe runtime evaluation

### Cranelift JIT Compilation
- Just-in-time compilation for performance-critical code
- 10x faster compilation than rustc
- 14% slower runtime than fully optimized Rust
- Seamless fallback from interpretation

## Conditional Compilation

RustyUI uses extensive conditional compilation to ensure zero production overhead:

```rust
// Development-only runtime interpretation
#[cfg(feature = "dev-ui")]
pub struct RuntimeInterpreter {
    rhai_engine: rhai::Engine,
    ast_parser: syn::File,
    jit_compiler: cranelift_jit::JITModule,
}

// Production builds exclude all interpretation code
#[cfg(not(feature = "dev-ui"))]
pub struct RuntimeInterpreter;

// Shared UI components work in both modes
pub trait UIComponent {
    fn render(&mut self, ctx: &mut dyn RenderContext);
    
    #[cfg(feature = "dev-ui")]
    fn hot_reload_state(&self) -> serde_json::Value;
    
    #[cfg(feature = "dev-ui")]
    fn restore_state(&mut self, state: serde_json::Value);
}
```

## State Preservation

Application state is automatically preserved across code changes:

- **Simple State**: Automatic serialization/deserialization
- **Complex Objects**: Custom preservation strategies
- **Closures**: Safe handling with state extraction
- **Framework State**: UI framework-specific preservation
- **Error Recovery**: Graceful fallback with state reset

## Security and Sandboxing

Runtime code execution is secured through multiple layers:

- **Rhai Engine**: Limited operations and module imports
- **AST Validation**: Syntax checking before interpretation
- **Resource Limits**: Memory, execution time, and operation constraints
- **WASM Sandboxing**: Isolated execution environment for plugins
- **No Unsafe Code**: All interpretation uses safe Rust patterns

## Error Handling

Robust error handling maintains development flow:

- **Error Isolation**: Interpretation failures don't crash the application
- **Fallback Chain**: Rhai → AST → JIT → Last Working State
- **Clear Diagnostics**: Detailed error messages with suggestions
- **State Recovery**: Automatic preservation of working application state
- **Graceful Degradation**: Continued operation with reduced functionality

## Platform Support

- **Windows**: 10 and later with native API integration
- **macOS**: 10.15 and later with platform optimizations
- **Linux**: glibc 2.28+ with distribution compatibility
- **Architecture**: x86_64, ARM64 support via Cranelift

## Repository Structure

```
rustyui/
├── crates/
│   ├── rustyui-core/          # Core dual-mode engine
│   ├── rustyui-interpreter/   # Runtime interpretation systems
│   ├── rustyui-adapters/      # UI framework adapters
│   ├── rustyui-cli/           # Command-line interface
│   └── rustyui-macros/        # Procedural macros
├── examples/                  # Framework-specific examples
├── docs/                      # Documentation and guides
├── tests/                     # Integration and property tests
└── benchmarks/                # Performance benchmarks
```

## Contributing

We welcome contributions to RustyUI. Please read our contributing guidelines and code of conduct before submitting pull requests.

### Development Setup
```bash
git clone https://github.com/iceyxsm/Rustyui.git
cd Rustyui
cargo build --all-features
cargo test --all-features
```

### Testing
```bash
# Run all tests including property-based tests
cargo test --all-features

# Run benchmarks
cargo bench

# Test specific framework adapters
cargo test --features egui-adapter
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Acknowledgments

Inspired by Flutter's JIT/AOT dual-mode architecture, modern game engine live coding systems, and the Rust community's commitment to zero-cost abstractions.