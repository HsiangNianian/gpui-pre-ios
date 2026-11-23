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
    PencilAction, on_multi_finger_tap, on_pencil_action, share_file, window_controls_rect,
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
