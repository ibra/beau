//! Semantic colour palettes for light and dark mode.

use egui::{Color32, Rgba};

/// The semantic colour palette.
///
/// Every colour in beau is referred to by role, never by value. See `DESIGN.md`
/// for the rationale behind each token.
///
/// The accent is the app's choice. By default it is *ink* (near-black in light
/// mode, near-white in dark mode); use [`crate::Theme::with_accent`] to pick a
/// brand colour, and every accent-derived token is recomputed with contrast
/// guarantees.
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
    /// Primary action fill.
    pub accent: Color32,
    /// Accent fill while hovered.
    pub accent_hover: Color32,
    /// Accent used as text or icon colour. At least 4.5:1 on [`Palette::surface`].
    pub accent_text: Color32,
    /// Destructive fill.
    pub danger: Color32,
    /// Destructive fill while hovered.
    pub danger_hover: Color32,
    /// Destructive text.
    pub danger_text: Color32,
    /// Text on a [`Palette::danger`] fill.
    pub text_on_danger: Color32,
    /// Keyboard focus ring. At least 3:1 on [`Palette::surface`].
    pub focus_ring: Color32,
    /// Scrim drawn behind modal dialogs.
    pub backdrop: Color32,
    /// Text selection highlight.
    pub selection: Color32,
}

const INK: Color32 = Color32::from_rgb(0x0C, 0x0C, 0x0C);
const PAPER: Color32 = Color32::from_rgb(0xED, 0xED, 0xED);

impl Palette {
    /// Light palette, with an ink accent.
    pub fn light() -> Self {
        let mut p = Self {
            bg: Color32::from_rgb(0xF4, 0xF4, 0xF4),
            surface: Color32::from_rgb(0xFB, 0xFB, 0xFB),
            surface_raised: Color32::WHITE,
            surface_overlay: Color32::WHITE,
            surface_hover: Color32::from_rgba_unmultiplied(0x0C, 0x0C, 0x0C, 12),
            surface_active: Color32::from_rgba_unmultiplied(0x0C, 0x0C, 0x0C, 20),
            border: Color32::from_rgb(0xE5, 0xE5, 0xE5),
            border_strong: Color32::from_rgb(0xD6, 0xD6, 0xD6),
            text: INK,
            text_muted: Color32::from_rgb(0x61, 0x61, 0x61),
            text_subtle: Color32::from_rgb(0x8F, 0x8F, 0x8F),
            danger: Color32::from_rgb(0xE5, 0x48, 0x4D),
            danger_hover: Color32::from_rgb(0xD9, 0x3D, 0x42),
            danger_text: Color32::from_rgb(0xCE, 0x2C, 0x31),
            text_on_danger: Color32::WHITE,
            backdrop: Color32::from_rgba_unmultiplied(0x0C, 0x0C, 0x0C, 72),
            // Accent-derived; filled in by `set_accent`.
            text_on_accent: Color32::WHITE,
            accent: INK,
            accent_hover: INK,
            accent_text: INK,
            focus_ring: INK,
            selection: Color32::TRANSPARENT,
        };
        p.set_accent(INK);
        p
    }

    /// Dark palette, with an ink (near-white) accent.
    pub fn dark() -> Self {
        let mut p = Self {
            bg: INK,
            surface: Color32::from_rgb(0x12, 0x12, 0x12),
            surface_raised: Color32::from_rgb(0x18, 0x18, 0x18),
            surface_overlay: Color32::from_rgb(0x1C, 0x1C, 0x1C),
            surface_hover: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 13),
            surface_active: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 23),
            border: Color32::from_rgb(0x26, 0x26, 0x26),
            border_strong: Color32::from_rgb(0x33, 0x33, 0x33),
            text: PAPER,
            text_muted: Color32::from_rgb(0xA0, 0xA0, 0xA0),
            text_subtle: Color32::from_rgb(0x6B, 0x6B, 0x6B),
            danger: Color32::from_rgb(0xE5, 0x48, 0x4D),
            danger_hover: Color32::from_rgb(0xF0, 0x58, 0x5D),
            danger_text: Color32::from_rgb(0xFF, 0x63, 0x69),
            text_on_danger: Color32::WHITE,
            backdrop: Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            text_on_accent: INK,
            accent: PAPER,
            accent_hover: PAPER,
            accent_text: PAPER,
            focus_ring: PAPER,
            selection: Color32::TRANSPARENT,
        };
        p.set_accent(PAPER);
        p
    }

    /// Whether this palette is for a dark UI.
    fn is_dark(&self) -> bool {
        luminance(self.surface) < 0.5
    }

    /// Set the accent and re-derive every accent-dependent token.
    ///
    /// * `text_on_accent`: ink or white, whichever contrasts more.
    /// * `accent_hover`: one step towards the surface's opposite.
    /// * `accent_text`: the accent, pushed towards `text` until it reaches
    ///   4.5:1 on `surface`.
    /// * `focus_ring`: likewise, at 3:1.
    /// * `selection`: a translucent wash of the accent.
    pub fn set_accent(&mut self, accent: Color32) {
        let accent = opaque(accent);
        self.accent = accent;
        self.text_on_accent = if contrast(INK, accent) >= contrast(Color32::WHITE, accent) {
            INK
        } else {
            Color32::WHITE
        };
        self.accent_hover = if luminance(accent) < 0.18 {
            accent.lerp_to_gamma(Color32::WHITE, 0.12)
        } else {
            accent.lerp_to_gamma(Color32::BLACK, 0.08)
        };
        self.accent_text = ensure_contrast(accent, self.surface, self.text, 4.5);
        self.focus_ring = ensure_contrast(accent, self.surface, self.text, 3.0);
        let wash = if self.is_dark() { 0.32 } else { 0.18 };
        let base = if luminance(accent) > 0.6 && !self.is_dark() {
            self.accent_text
        } else {
            accent
        };
        self.selection = base.gamma_multiply(wash);
    }
}

fn opaque(c: Color32) -> Color32 {
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    Color32::from_rgb(r, g, b)
}

/// WCAG relative luminance of an opaque colour.
pub fn luminance(c: Color32) -> f32 {
    let l = Rgba::from(opaque(c));
    0.2126 * l.r() + 0.7152 * l.g() + 0.0722 * l.b()
}

/// WCAG contrast ratio between two opaque colours (1 to 21).
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Move `fg` towards `toward` until it reaches `min` contrast on `bg`.
fn ensure_contrast(fg: Color32, bg: Color32, toward: Color32, min: f32) -> Color32 {
    let mut c = fg;
    for i in 1..=20 {
        if contrast(c, bg) >= min {
            break;
        }
        c = fg.lerp_to_gamma(toward, i as f32 / 20.0);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_accent_is_ink() {
        assert_eq!(Palette::light().accent, INK);
        assert_eq!(Palette::light().text_on_accent, Color32::WHITE);
        assert_eq!(Palette::dark().accent, PAPER);
        assert_eq!(Palette::dark().text_on_accent, INK);
    }

    #[test]
    fn any_accent_meets_contrast() {
        let accents = [
            Color32::from_rgb(0xC6, 0xF4, 0x32), // pale lime
            Color32::from_rgb(0xFF, 0xD6, 0x0A), // yellow
            Color32::from_rgb(0x3B, 0x82, 0xF6), // blue
            Color32::from_rgb(0x0F, 0x76, 0x6E), // dark teal
            Color32::from_rgb(0xFF, 0xFF, 0xFF), // white
        ];
        for base in [Palette::light(), Palette::dark()] {
            for a in accents {
                let mut p = base.clone();
                p.set_accent(a);
                assert!(contrast(p.accent_text, p.surface) >= 4.5, "{a:?} text");
                assert!(contrast(p.focus_ring, p.surface) >= 3.0, "{a:?} ring");
                assert!(
                    contrast(p.text_on_accent, p.accent) >= 3.0,
                    "{a:?} on-accent"
                );
            }
        }
    }
}
