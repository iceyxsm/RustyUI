# RustyUI User Guide

## Getting Started

### Installation

```bash
cargo install rustyui
```

### Creating a New Project

```bash
rustyui new my-app --framework egui
cd my-app
```

### Initializing an Existing Project

```bash
cd my-existing-project
rustyui init --framework egui
```

## Development Workflow

### 1. Start Development Mode

```bash
cargo run --features dev-ui
```

This starts your application with:
- Runtime interpretation enabled
- File watching active
- State preservation ready
- Hot reload on save

### 2. Edit Your Code

Make changes to your UI code. RustyUI will:
- Detect file changes within 50ms
- Interpret changes using the best strategy
- Preserve application state
- Apply updates instantly

### 3. See Changes Instantly

No compilation needed! Your changes appear immediately.

### 4. Build for Production

```bash
cargo build --release
```

Production builds:
- Strip all development features
- Achieve zero overhead
- Match native Rust performance

## Framework-Specific Guides

### egui

```rust
use rustyui_adapters::EguiAdapter;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My App");
            // Your UI code here
        });
    }
}
```

### iced

```rust
use rustyui_adapters::IcedAdapter;

// Iced-specific implementation
```

## Configuration

### rustyui.toml

```toml
[dual-mode]
framework = "egui"
watch-paths = ["src/", "ui/"]

[development]
interpretation-strategy = "hybrid"
jit-compilation-threshold = 100
state-preservation = true
performance-monitoring = true
```

## Best Practices

1. **Keep UI Logic Separate**: Separate UI code from business logic
2. **Use State Preservation**: Implement state serialization for complex types
3. **Test Both Modes**: Verify behavior in development and production
4. **Monitor Performance**: Use built-in performance monitoring
5. **Handle Errors Gracefully**: Implement error recovery strategies

## Troubleshooting

See TESTING.md for detailed troubleshooting guide.

## Next Steps

- Read the API documentation
- Explore examples in the repository
- Join the community discussions
