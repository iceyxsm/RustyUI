# RustyUI API Documentation

## Core API

### DualModeEngine

The main engine that manages runtime interpretation and production compilation.

```rust
use rustyui_core::DualModeEngine;

// Create engine with configuration
let config = DualModeConfig::default();
let mut engine = DualModeEngine::new(config)?;

// Initialize for development mode
engine.initialize()?;
engine.start_development_mode()?;

// Interpret UI changes
let result = engine.interpret_ui_change(code, component_id)?;
```

### RuntimeInterpreter

Handles code interpretation with multiple strategies.

```rust
use rustyui_interpreter::RuntimeInterpreter;

let mut interpreter = RuntimeInterpreter::new()?;

// Interpret UI change
let change = UIChange {
    content: "fn update() { /* code */ }".to_string(),
    interpretation_strategy: None, // Auto-select
    component_id: Some("my_component".to_string()),
    change_type: ChangeType::ComponentUpdate,
};

let result = interpreter.interpret_change(&change)?;
```

### StatePreservor

Manages application state across hot reloads.

```rust
use rustyui_core::StatePreservor;

let mut preservor = StatePreservor::new();

// Save state
preservor.save_global_state("component_id", &state_data)?;

// Restore state
let state = preservor.restore_global_state::<MyState>("component_id")?;
```

## Framework Adapters

### UIFrameworkAdapter Trait

```rust
pub trait UIFrameworkAdapter {
    fn apply_runtime_update(&mut self, update: &UIUpdate) -> Result<()>;
    fn get_component_state(&self, component_id: &str) -> Option<serde_json::Value>;
    fn restore_component_state(&mut self, component_id: &str, state: serde_json::Value) -> Result<()>;
}
```

### EguiAdapter

```rust
use rustyui_adapters::EguiAdapter;

let adapter = EguiAdapter::new();
adapter.apply_runtime_update(&update)?;
```

## CLI Commands

### rustyui init

Initialize RustyUI in an existing project.

```bash
rustyui init --framework egui
```

### rustyui dev

Start development mode with hot reload.

```bash
rustyui dev --watch --port 3000
```

## Configuration

### DualModeConfig

```rust
pub struct DualModeConfig {
    pub framework: UIFramework,
    pub watch_paths: Vec<PathBuf>,
    pub development_settings: DevelopmentSettings,
}
```

See full documentation at https://docs.rs/rustyui
