//! Platform integration helpers (safe-area insets).
//!
//! Desktop has no insets. On Android we query the activity's content rectangle
//! via `android-activity`, which already accounts for the status bar, notch and
//! gesture/navigation bar, and convert it to logical (egui point) insets.

/// Logical-pixel insets to keep content clear of notches / system bars.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Insets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Insets {
    pub const ZERO: Insets = Insets {
        top: 0.0,
        bottom: 0.0,
        left: 0.0,
        right: 0.0,
    };
}

#[cfg(all(feature = "desktop", target_os = "android"))]
pub struct InsetsProvider {
    app: winit::platform::android::activity::AndroidApp,
}

#[cfg(all(feature = "desktop", target_os = "android"))]
impl InsetsProvider {
    pub fn new(app: winit::platform::android::activity::AndroidApp) -> Self {
        Self { app }
    }

    /// Current content insets in logical points, given the window's physical
    /// size and egui's pixels-per-point scale.
    pub fn insets(&self, window_physical: (f32, f32), scale: f32) -> Insets {
        let rect = self.app.content_rect();
        let scale = if scale > 0.0 { scale } else { 1.0 };
        let (win_w, win_h) = window_physical;
        let top = (rect.top as f32).max(0.0) / scale;
        let bottom = ((win_h - rect.bottom as f32).max(0.0)) / scale;
        let left = (rect.left as f32).max(0.0) / scale;
        let right = ((win_w - rect.right as f32).max(0.0)) / scale;
        let _ = win_w;
        Insets {
            top,
            bottom,
            left,
            right,
        }
    }
}

#[cfg(not(all(feature = "desktop", target_os = "android")))]
pub struct InsetsProvider;

#[cfg(not(all(feature = "desktop", target_os = "android")))]
impl InsetsProvider {
    pub fn new() -> Self {
        InsetsProvider
    }

    pub fn insets(&self, _window_physical: (f32, f32), _scale: f32) -> Insets {
        Insets::ZERO
    }
}

#[cfg(not(all(feature = "desktop", target_os = "android")))]
impl Default for InsetsProvider {
    fn default() -> Self {
        Self::new()
    }
}
