//! Template management for RustyUI projects

use crate::error::{CliError, CliResult};
use console::style;
use std::path::PathBuf;

/// Template manager for generating RustyUI project files
pub struct TemplateManager {
    project_path: PathBuf,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }
    
    /// Generate example code for a specific framework
    pub fn generate_example_code(&self, framework: &str) -> CliResult<()> {
        match framework {
            "egui" => self.generate_egui_example()?,
            "iced" => self.generate_iced_example()?,
            "slint" => self.generate_slint_example()?,
            "tauri" => self.generate_tauri_example()?,
            _ => return Err(CliError::unsupported_framework(framework)),
        }
        
        println!("{} Generated example code for {}", style("✓").green(), framework);
        
        Ok(())
    }
    
    /// Generate egui example with hot reload support
    fn generate_egui_example(&self) -> CliResult<()> {
        let main_rs_content = r#"//! RustyUI egui example with hot reload support

use eframe::egui;
use rustyui_core::{DualModeEngine, DualModeConfig};
use rustyui_adapters::egui::EguiAdapter;
use rustyui_macros::HotReloadable;

fn main() -> Result<(), eframe::Error> {
    // Initialize RustyUI dual-mode engine
    #[cfg(feature = "dev-ui")]
    {
        let config = DualModeConfig::default();
        let mut engine = DualModeEngine::new(config).expect("Failed to initialize dual-mode engine");
        engine.start_development_mode().expect("Failed to start development mode");
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "RustyUI egui Example",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
#[cfg_attr(feature = "dev-ui", derive(HotReloadable))]
struct MyApp {
    name: String,
    age: u32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RustyUI egui Example");
            
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            
            #[cfg(feature = "dev-ui")]
            {
                ui.separator();
                ui.label("🔥 Hot reload is active! Edit this code and see instant changes.");
            }
        });
    }
}
"#;
        
        let main_rs_path = self.project_path.join("src").join("main.rs");
        std::fs::write(main_rs_path, main_rs_content)?;
        
        Ok(())
    }
    
    /// Generate iced example with hot reload support
    fn generate_iced_example(&self) -> CliResult<()> {
        let main_rs_content = r#"//! RustyUI iced example with hot reload support

use iced::{Application, Command, Element, Settings, Theme};
use iced::widget::{button, column, container, text, text_input};
use rustyui_core::{DualModeEngine, DualModeConfig};
use rustyui_adapters::iced::IcedAdapter;
use rustyui_macros::HotReloadable;

fn main() -> iced::Result {
    // Initialize RustyUI dual-mode engine
    #[cfg(feature = "dev-ui")]
    {
        let config = DualModeConfig::default();
        let mut engine = DualModeEngine::new(config).expect("Failed to initialize dual-mode engine");
        engine.start_development_mode().expect("Failed to start development mode");
    }
    
    MyApp::run(Settings::default())
}

#[derive(Default)]
#[cfg_attr(feature = "dev-ui", derive(HotReloadable))]
struct MyApp {
    name: String,
    count: i32,
}

#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    IncrementPressed,
    DecrementPressed,
}

impl Application for MyApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("RustyUI iced Example")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NameChanged(name) => {
                self.name = name;
            }
            Message::IncrementPressed => {
                self.count += 1;
            }
            Message::DecrementPressed => {
                self.count -= 1;
            }
        }
        
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("RustyUI iced Example").size(30),
            text_input("Enter your name...", &self.name)
                .on_input(Message::NameChanged),
            text(format!("Hello, {}!", self.name)).size(20),
            button("Increment").on_press(Message::IncrementPressed),
            text(format!("Count: {}", self.count)).size(20),
            button("Decrement").on_press(Message::DecrementPressed),
        ]
        .spacing(20)
        .padding(20);

        #[cfg(feature = "dev-ui")]
        let content = column![
            content,
            text("🔥 Hot reload is active! Edit this code and see instant changes.")
                .size(14)
        ]
        .spacing(10);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
"#;
        
        let main_rs_path = self.project_path.join("src").join("main.rs");
        std::fs::write(main_rs_path, main_rs_content)?;
        
        Ok(())
    }
    
    /// Generate slint example with hot reload support
    fn generate_slint_example(&self) -> CliResult<()> {
        let main_rs_content = r#"//! RustyUI slint example with hot reload support

use slint::ComponentHandle;
use rustyui_core::{DualModeEngine, DualModeConfig};
use rustyui_adapters::slint::SlintAdapter;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // Initialize RustyUI dual-mode engine
    #[cfg(feature = "dev-ui")]
    {
        let config = DualModeConfig::default();
        let mut engine = DualModeEngine::new(config).expect("Failed to initialize dual-mode engine");
        engine.start_development_mode().expect("Failed to start development mode");
    }
    
    let ui = AppWindow::new()?;
    
    let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });

    ui.run()
}
"#;
        
        let main_rs_path = self.project_path.join("src").join("main.rs");
        std::fs::write(main_rs_path, main_rs_content)?;
        
        // Create slint UI file
        let ui_content = r#"import { Button, VerticalBox } from "std-widgets.slint";

export component AppWindow inherits Window {
    in-out property <int> counter: 42;
    callback request-increase-value();
    
    VerticalBox {
        Text {
            text: "RustyUI Slint Example";
            font-size: 24px;
        }
        
        Text {
            text: @tr("Counter: {}", root.counter);
            font-size: 18px;
        }
        
        Button {
            text: "Increase value";
            clicked => {
                root.request-increase-value();
            }
        }
        
        @if (debug) : Text {
            text: "🔥 Hot reload is active! Edit this code and see instant changes.";
            font-size: 12px;
        }
    }
}
"#;
        
        let ui_path = self.project_path.join("ui").join("appwindow.slint");
        std::fs::create_dir_all(ui_path.parent().unwrap())?;
        std::fs::write(ui_path, ui_content)?;
        
        Ok(())
    }
    
    /// Generate tauri example with hot reload support
    fn generate_tauri_example(&self) -> CliResult<()> {
        let main_rs_content = r#"//! RustyUI tauri example with hot reload support

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use rustyui_core::{DualModeEngine, DualModeConfig};
use rustyui_adapters::tauri::TauriAdapter;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    // Initialize RustyUI dual-mode engine
    #[cfg(feature = "dev-ui")]
    {
        let config = DualModeConfig::default();
        let mut engine = DualModeEngine::new(config).expect("Failed to initialize dual-mode engine");
        engine.start_development_mode().expect("Failed to start development mode");
    }
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
"#;
        
        let main_rs_path = self.project_path.join("src-tauri").join("src").join("main.rs");
        std::fs::create_dir_all(main_rs_path.parent().unwrap())?;
        std::fs::write(main_rs_path, main_rs_content)?;
        
        // Create basic HTML file
        let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>RustyUI Tauri Example</title>
</head>
<body>
    <div id="app">
        <h1>RustyUI Tauri Example</h1>
        <div class="row">
            <input id="greet-input" placeholder="Enter a name..." />
            <button type="button" id="greet-button">Greet</button>
        </div>
        <p id="greet-msg"></p>
        <p style="font-size: 12px; color: #666;">
            🔥 Hot reload is active! Edit the Rust code and see instant changes.
        </p>
    </div>
    
    <script>
        const invoke = window.__TAURI__.tauri.invoke;
        
        const greetInputEl = document.querySelector('#greet-input');
        const greetMsgEl = document.querySelector('#greet-msg');
        
        async function greet() {
            greetMsgEl.textContent = await invoke('greet', { name: greetInputEl.value });
        }
        
        document.querySelector('#greet-button').addEventListener('click', () => greet());
        
        window.addEventListener('DOMContentLoaded', () => {
            greetInputEl.focus();
        });
    </script>
</body>
</html>
"#;
        
        let html_path = self.project_path.join("index.html");
        std::fs::write(html_path, html_content)?;
        
        Ok(())
    }
    
    /// Generate .gitignore file for RustyUI projects
    pub fn generate_gitignore(&self) -> CliResult<()> {
        let gitignore_content = r#"# Rust
/target/
**/*.rs.bk
Cargo.lock

# RustyUI
*.rustyui-cache
.rustyui/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
"#;
        
        let gitignore_path = self.project_path.join(".gitignore");
        if !gitignore_path.exists() {
            std::fs::write(gitignore_path, gitignore_content)?;
            println!("{} Created .gitignore", style("✓").green());
        }
        
        Ok(())
    }
    
    /// Generate README.md for RustyUI projects
    pub fn generate_readme(&self, project_name: &str, framework: &str) -> CliResult<()> {
        let readme_content = format!(r#"# {project_name}

A RustyUI project using {framework} with instant hot reload capabilities.

## Features

- 🔥 **Instant Hot Reload**: 0ms compilation time for UI changes during development
- 🚀 **Zero Production Overhead**: Native Rust performance with no runtime penalties
- 🎯 **Framework Agnostic**: Works with {framework} and other Rust UI frameworks
- 💾 **State Preservation**: Application state maintained across code changes
- 🔄 **Seamless Transition**: Same codebase for development and production

## Getting Started

### Development Mode

Start the development server with hot reload:

```bash
rustyui dev
```

Or manually with cargo:

```bash
cargo run --features dev-ui
```

### Production Build

Build for production (strips all development features):

```bash
rustyui build --release
```

Or manually with cargo:

```bash
cargo build --release
```

## Project Structure

```
{project_name}/
├── src/
│   └── main.rs          # Main application code
├── rustyui.toml         # RustyUI configuration
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

## Configuration

Edit `rustyui.toml` to customize:

- UI framework settings
- File watching paths
- Development mode behavior
- Production build options

## Learn More

- [RustyUI Documentation](https://github.com/iceyxsm/Rustyui)
- [{framework} Documentation](https://docs.rs/{framework})
- [Rust Book](https://doc.rust-lang.org/book/)
"#, project_name = project_name, framework = framework);
        
        let readme_path = self.project_path.join("README.md");
        if !readme_path.exists() {
            std::fs::write(readme_path, readme_content)?;
            println!("{} Created README.md", style("✓").green());
        }
        
        Ok(())
    }
}