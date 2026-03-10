//! # RustyUI Adapters
//! 
//! Framework adapters for RustyUI supporting egui, iced, slint, and tauri

pub mod traits;

#[cfg(feature = "egui-adapter")]
pub mod egui_adapter;

#[cfg(feature = "iced-adapter")]
pub mod iced_adapter;

#[cfg(feature = "slint-adapter")]
pub mod slint_adapter;

#[cfg(feature = "tauri-adapter")]
pub mod tauri_adapter;

pub use traits::{UIFrameworkAdapter, RenderContext, FrameworkState};

#[cfg(feature = "egui-adapter")]
pub use egui_adapter::EguiAdapter;

#[cfg(feature = "iced-adapter")]
pub use iced_adapter::IcedAdapter;

#[cfg(feature = "slint-adapter")]
pub use slint_adapter::SlintAdapter;

#[cfg(feature = "tauri-adapter")]
pub use tauri_adapter::TauriAdapter;