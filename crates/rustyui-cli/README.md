# RustyUI CLI

Command-line interface for RustyUI projects, providing project initialization, development mode, and configuration management.

## Features

- 🚀 **Project Initialization**: Set up RustyUI in existing projects or create new ones
- 🔥 **Development Mode**: Start hot reload development server with instant UI updates
- ⚙️ **Configuration Management**: Generate and manage RustyUI configuration files
- 🎯 **Framework Support**: Works with egui, iced, slint, and tauri
- 📝 **Template Generation**: Creates example code and project templates
- 🛠️ **Build Integration**: Seamless integration with Cargo workflows

## Installation

The CLI is part of the RustyUI workspace. Build it with:

```bash
cargo build -p rustyui-cli
```

Or run directly:

```bash
cargo run -p rustyui-cli -- <command>
```

## Commands

### `rustyui init`

Initialize RustyUI in an existing Rust project:

```bash
rustyui init --framework egui
```

Options:
- `--framework <FRAMEWORK>`: UI framework to use (egui, iced, slint, tauri)
- `--path <PATH>`: Project directory (default: current directory)
- `--force`: Force initialization even if rustyui.toml exists
- `--yes`: Skip interactive prompts and use defaults

### `rustyui new`

Create a new RustyUI project:

```bash
rustyui new my-app --framework egui
```

Options:
- `--framework <FRAMEWORK>`: UI framework to use (egui, iced, slint, tauri)
- `--yes`: Skip interactive prompts and use defaults

### `rustyui dev`

Start development mode with hot reload:

```bash
rustyui dev
```

Options:
- `--path <PATH>`: Project directory (default: current directory)
- `--port <PORT>`: Port for development server (if applicable)
- `--no-watch`: Disable file watching

### `rustyui build`

Build project for production:

```bash
rustyui build --release
```

Options:
- `--path <PATH>`: Project directory (default: current directory)
- `--release`: Release build

### `rustyui config`

Show current project configuration:

```bash
rustyui config
```

Options:
- `--path <PATH>`: Project directory (default: current directory)

## Global Options

- `-v, --verbose`: Enable verbose output
- `-q, --quiet`: Suppress all output except errors
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Configuration

RustyUI projects use a `rustyui.toml` configuration file:

```toml
[framework]
type = "egui"

[watch_paths]
paths = ["src"]

[development_settings]
interpretation_strategy = { Hybrid = { rhai_threshold = 10, jit_threshold = 100 } }
jit_compilation_threshold = 100
state_preservation = true
performance_monitoring = true
change_detection_delay_ms = 50

[production_settings]
strip_dev_features = true
optimization_level = "Release"
binary_size_optimization = true
security_hardening = true
```

## Examples

### Initialize an existing egui project

```bash
cd my-egui-project
rustyui init --framework egui
```

### Create a new iced project

```bash
rustyui new my-iced-app --framework iced
cd my-iced-app
rustyui dev
```

### Start development mode

```bash
rustyui dev
# or
cargo run --features dev-ui
```

### Build for production

```bash
rustyui build --release
# or
cargo build --release
```

## Project Structure

After initialization, your project will have:

```
my-project/
├── src/
│   └── main.rs          # Main application with hot reload support
├── rustyui.toml         # RustyUI configuration
├── Cargo.toml           # Updated with RustyUI dependencies
├── README.md            # Project documentation
└── .gitignore           # Git ignore patterns
```

## Framework-Specific Features

### egui
- Automatic egui and eframe dependencies
- Example with hot reloadable components
- State preservation across UI changes

### iced
- Automatic iced dependencies
- Application trait implementation
- Message-based architecture support

### slint
- Automatic slint dependencies
- .slint UI file generation
- Component-based architecture

### tauri
- Automatic tauri dependencies
- HTML/JS frontend template
- Rust backend integration

## Development Workflow

1. **Initialize**: `rustyui init --framework <framework>`
2. **Develop**: `rustyui dev` (edit code and see instant changes)
3. **Build**: `rustyui build --release` (zero-overhead production build)

## Error Handling

The CLI provides clear error messages and suggestions:

- Configuration validation errors
- Project structure issues
- Framework-specific problems
- Build and compilation errors

## Integration

The CLI integrates seamlessly with:

- Cargo workspaces
- Existing Rust projects
- CI/CD pipelines
- Development tools

## Contributing

See the main RustyUI repository for contribution guidelines.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.