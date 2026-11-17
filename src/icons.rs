//! Monochrome UI icons (Material Symbols) embedded as 48×48 alpha masks.
//!
//! We avoid an image-decoding dependency by storing each icon as raw 8-bit
//! coverage (alpha) bytes and building the [`egui::ColorImage`] at runtime.
//! Icons are tinted at draw time, so the same asset works in light and dark
//! themes. Source SVGs live in `assets/icons/` (see `THIRD_PARTY_NOTICES.md`).

use eframe::egui::{self, Color32, Rect, TextureHandle, Vec2};

/// Size (edge length) of every embedded icon mask.
pub const ICON_PX: usize = 48;

macro_rules! icon_set {
    ($($variant:ident => $file:literal),+ $(,)?) => {
        /// A named UI icon.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $($variant),+
        }

        impl Icon {
            fn mask(self) -> &'static [u8] {
                match self {
                    $(Icon::$variant => include_bytes!(concat!("../assets/icons/rgba/", $file, ".a.gray"))),+
                }
            }
        }
    };
}

icon_set! {
    Menu => "menu",
    Pause => "pause",
    Play => "play_arrow",
    Undo => "undo",
    Redo => "redo",
    AutoNotes => "filter_9_plus",
    Notes => "edit",
    Erase => "ink_eraser",
    Hint => "lightbulb",
    Timer => "timer",
    Win => "celebration",
}

impl Icon {
    /// Build a white RGBA [`egui::ColorImage`] from the embedded alpha mask.
    fn color_image(self) -> egui::ColorImage {
        let mask = self.mask();
        debug_assert_eq!(mask.len(), ICON_PX * ICON_PX);
        let mut pixels = Vec::with_capacity(ICON_PX * ICON_PX);
        for &a in mask.iter().take(ICON_PX * ICON_PX) {
            pixels.push(Color32::from_rgba_unmultiplied(255, 255, 255, a));
        }
        egui::ColorImage {
            size: [ICON_PX, ICON_PX],
            pixels,
            source_size: Vec2::splat(ICON_PX as f32),
        }
    }
}

/// Lazily-uploaded icon textures, keyed by [`Icon`].
#[derive(Default)]
pub struct IconStore {
    textures: std::collections::HashMap<IconKey, TextureHandle>,
}

#[derive(PartialEq, Eq, Hash)]
struct IconKey(usize);

impl IconStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get (uploading on first use) the texture for an icon.
    fn texture(&mut self, ctx: &egui::Context, icon: Icon) -> &TextureHandle {
        let key = IconKey(icon as usize);
        self.textures.entry(key).or_insert_with(|| {
            ctx.load_texture(
                format!("icon_{:?}", icon),
                icon.color_image(),
                egui::TextureOptions::LINEAR,
            )
        })
    }

    /// Paint `icon` centered in `rect`, tinted with `tint`.
    pub fn draw(
        &mut self,
        painter: &egui::Painter,
        ctx: &egui::Context,
        rect: Rect,
        icon: Icon,
        tint: Color32,
    ) {
        let texture = self.texture(ctx, icon).clone();
        let mut mesh = egui::Mesh::with_texture(texture.id());
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        mesh.add_rect_with_uv(rect, uv, tint);
        painter.add(egui::Shape::mesh(mesh));
    }

    /// An [`egui::Image`] for `icon`, sized to `size_px` and tinted `tint`.
    ///
    /// Intended for use with [`egui::Button`] / menus; the texture is uploaded
    /// on first use.
    pub fn image(
        &mut self,
        ctx: &egui::Context,
        icon: Icon,
        size_px: f32,
        tint: Color32,
    ) -> egui::Image<'static> {
        let texture = self.texture(ctx, icon).clone();
        egui::Image::new(&texture)
            .fit_to_exact_size(Vec2::splat(size_px))
            .tint(tint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_mask_is_correct_size_and_not_blank() {
        for icon in [
            Icon::Menu,
            Icon::Pause,
            Icon::Play,
            Icon::Undo,
            Icon::Redo,
            Icon::AutoNotes,
            Icon::Notes,
            Icon::Erase,
            Icon::Hint,
            Icon::Timer,
            Icon::Win,
        ] {
            let mask = icon.mask();
            assert_eq!(mask.len(), ICON_PX * ICON_PX, "wrong size for {icon:?}");
            let opaque = mask.iter().filter(|&&a| a > 0).count();
            assert!(
                opaque > (ICON_PX * ICON_PX) / 50,
                "{icon:?} icon mask looks blank ({opaque} opaque pixels)"
            );
        }
    }

    #[test]
    fn color_image_builds() {
        let image = Icon::Menu.color_image();
        assert_eq!(image.size, [ICON_PX, ICON_PX]);
        assert_eq!(image.pixels.len(), ICON_PX * ICON_PX);
    }
}
