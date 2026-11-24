//! Native iOS platform support for GPUI.
//!
//! This crate connects GPUI to UIKit, CoreText, and Metal through the shared
//! Apple renderer. It owns iOS application lifecycle, windowing, native
//! text input, safe-area and keyboard insets, and raw touch delivery.

#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(target_os = "ios")]
pub use ios::{
    ExternalDisplay, IosPlatform, current_platform, current_touch_is_pencil, external_display,
    PencilAction, on_multi_finger_tap, on_pencil_action, set_interface_style, set_status_bar_style,
    share_file, window_controls_rect,
};

/// The foreground color used by the iOS status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusBarContentStyle {
    /// Light status-bar content for dark backgrounds.
    Light,
    /// Dark status-bar content for light backgrounds.
    #[default]
    Dark,
}

/// The interface style the app's windows are held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterfaceStyle {
    /// Follow the system's light or dark appearance.
    #[default]
    System,
    /// Always light, with dark status-bar content.
    Light,
    /// Always dark, with light status-bar content.
    Dark,
}
