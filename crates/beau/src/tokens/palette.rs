//! Semantic colour palettes for light and dark mode.

use egui::Color32;

/// The semantic colour palette.
///
/// Every colour in beau is referred to by role, never by value. See `DESIGN.md`
/// for the rationale behind each token.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    /// App canvas and sidebar.
    pub bg: Color32,
    /// Main content area.
    pub surface: Color32,
    /// Cards, inputs and secondary buttons.
    pub surface_raised: Color32,
    /// Dialogs and popovers.
    pub surface_overlay: Color32,
    /// Translucent hover wash, layered over any surface.
    pub surface_hover: Color32,
    /// Translucent pressed/selected wash, layered over any surface.
    pub surface_active: Color32,
    /// Hairline dividers and quiet outlines.
    pub border: Color32,
    /// Outlines of interactive controls.
    pub border_strong: Color32,
    /// Primary text.
    pub text: Color32,
    /// Secondary text.
    pub text_muted: Color32,
    /// Hints, IDs and placeholders.
    pub text_subtle: Color32,
    /// Text drawn on top of an [`Palette::accent`] fill.
    pub text_on_accent: Color32,
    /// Primary accent fill (electric lime).
    pub accent: Color32,
    /// Accent fill while hovered.
    pub accent_hover: Color32,
    /// Accent used as text or icon colour. Readable on [`Palette::surface`].
    pub accent_text: Color32,
    /// Destructive fill.
    pub danger: Color32,
    /// Destructive fill while hovered.
    pub danger_hover: Color32,
    /// Destructive text.
    pub danger_text: Color32,
    /// Text on a [`Palette::danger`] fill.
    pub text_on_danger: Color32,
    /// Keyboard focus ring.
    pub focus_ring: Color32,
    /// Scrim drawn behind modal dialogs.
    pub backdrop: Color32,
    /// Text selection highlight.
    pub selection: Color32,
}

const INK: Color32 = Color32::from_rgb(0x0B, 0x0C, 0x0E);
const LIME: Color32 = Color32::from_rgb(0xC6, 0xF4, 0x32);

impl Palette {
    /// Graphite light: the hero palette.
    pub fn light() -> Self {
        Self {
            bg: Color32::from_rgb(0xF3, 0xF3, 0xF4),
            surface: Color32::from_rgb(0xFB, 0xFB, 0xFB),
            surface_raised: Color32::WHITE,
            surface_overlay: Color32::WHITE,
            surface_hover: Color32::from_rgba_unmultiplied(0x0B, 0x0C, 0x0E, 12),
            surface_active: Color32::from_rgba_unmultiplied(0x0B, 0x0C, 0x0E, 20),
            border: Color32::from_rgb(0xE4, 0xE4, 0xE7),
            border_strong: Color32::from_rgb(0xD2, 0xD3, 0xD7),
            text: INK,
            text_muted: Color32::from_rgb(0x5D, 0x60, 0x68),
            text_subtle: Color32::from_rgb(0x8E, 0x91, 0x99),
            text_on_accent: INK,
            accent: LIME,
            accent_hover: Color32::from_rgb(0xB9, 0xE8, 0x26),
            accent_text: Color32::from_rgb(0x4D, 0x6B, 0x00),
            danger: Color32::from_rgb(0xE5, 0x48, 0x4D),
            danger_hover: Color32::from_rgb(0xD9, 0x3D, 0x42),
            danger_text: Color32::from_rgb(0xCE, 0x2C, 0x31),
            text_on_danger: Color32::WHITE,
            focus_ring: Color32::from_rgb(0x6F, 0x9C, 0x00),
            backdrop: Color32::from_rgba_unmultiplied(0x0B, 0x0C, 0x0E, 72),
            selection: Color32::from_rgba_unmultiplied(0xC6, 0xF4, 0x32, 150),
        }
    }

    /// Graphite dark.
    pub fn dark() -> Self {
        Self {
            bg: INK,
            surface: Color32::from_rgb(0x11, 0x12, 0x15),
            surface_raised: Color32::from_rgb(0x17, 0x18, 0x1C),
            surface_overlay: Color32::from_rgb(0x1B, 0x1C, 0x21),
            surface_hover: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 13),
            surface_active: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 23),
            border: Color32::from_rgb(0x25, 0x27, 0x2C),
            border_strong: Color32::from_rgb(0x33, 0x35, 0x3B),
            text: Color32::from_rgb(0xED, 0xEE, 0xF0),
            text_muted: Color32::from_rgb(0x9C, 0x9E, 0xA5),
            text_subtle: Color32::from_rgb(0x68, 0x6B, 0x73),
            text_on_accent: INK,
            accent: LIME,
            accent_hover: Color32::from_rgb(0xD3, 0xFB, 0x5A),
            accent_text: LIME,
            danger: Color32::from_rgb(0xE5, 0x48, 0x4D),
            danger_hover: Color32::from_rgb(0xF0, 0x58, 0x5D),
            danger_text: Color32::from_rgb(0xFF, 0x63, 0x69),
            text_on_danger: Color32::WHITE,
            focus_ring: LIME,
            backdrop: Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            selection: Color32::from_rgba_unmultiplied(0xC6, 0xF4, 0x32, 70),
        }
    }
}
