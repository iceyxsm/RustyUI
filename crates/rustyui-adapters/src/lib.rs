//! # RustyUI Adapters
//! 
//! Framework adapters for RustyUI supporting egui, iced, slint, and tauri

pub mod traits;

#[cfg(feature = "egui-adapter")]
pub mod egui_adapter;

#[cfg(feature = "egui-adapter")]
pub mod egui_components;

#[cfg(feature = "iced-adapter")]
pub mod iced_adapter;

#[cfg(feature = "slint-adapter")]
pub mod slint_adapter;

#[cfg(feature = "tauri-adapter")]
pub mod tauri_adapter;

pub use traits::{
    UIFrameworkAdapter, RenderContext, UIComponent, AdapterFactory,
    FrameworkState, RuntimeUpdate, UpdateType, FrameworkConfig,
    ComponentStyle, Rect, RenderFeature, LayoutType, FrameworkMetrics,
    AdapterResult, AdapterError, Padding, Margin, Border, BorderStyle,
};

#[cfg(feature = "dev-ui")]
pub use traits::{InterpretationHint, HotReloadMetrics};

#[cfg(feature = "egui-adapter")]
pub use egui_adapter::EguiAdapter;

#[cfg(feature = "egui-adapter")]
pub use egui_components::{EguiButton, EguiText, EguiInput, EguiLayout, LayoutDirection};

#[cfg(feature = "iced-adapter")]
pub use iced_adapter::IcedAdapter;

#[cfg(feature = "slint-adapter")]
pub use slint_adapter::SlintAdapter;

#[cfg(feature = "tauri-adapter")]
pub use tauri_adapter::TauriAdapter;