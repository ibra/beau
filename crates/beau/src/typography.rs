//! Typography: the bundled Inter font and helpers to turn a [`TextRole`] into
//! egui text.
//!
//! beau bundles **Inter** (SIL Open Font License 1.1, see
//! `assets/fonts/OFL.txt`) as a single variable font, subset to Latin. The
//! subset was produced with:
//!
//! ```text
//! pyftsubset InterVariable.ttf --unicodes="U+0000-00FF,U+0131,U+0152-0153,\
//!   U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20AC,U+2122,U+2190-21FF,\
//!   U+2212,U+2215,U+2303-2318,U+2325,U+2387,U+238B,U+25A0-25FF,U+2713-2717,\
//!   U+FEFF,U+FFFD" --layout-features='kern,liga,calt,tnum,cv11,ss03,ccmp,locl,mark,mkmk'
//! ```
//!
//! Each [`Weight`] is a separate egui font family that points at the same font
//! bytes with different variation coordinates (`wght`, `opsz`).
//!
//! **Tabular figures.** beau uses fixed-width digits everywhere so counts,
//! IDs and dates align. egui can't switch OpenType features, so
//! `InterTabularDigits.ttf` (9 KB, built by `assets/fonts/tools/tabular_digits.py`)
//! holds Inter's `tnum` digits as default glyphs and sits first in every family.

use std::sync::Arc;

use egui::epaint::text::{FontData, FontDefinitions, FontTweak, VariationCoords};
use egui::{Context, FontFamily, FontId, RichText};

use crate::tokens::{TextRole, TextSpec, Theme, Weight};

/// Inter variable font bytes (Latin subset).
pub static INTER_VARIABLE: &[u8] = include_bytes!("../assets/fonts/InterVariable.ttf");

/// Inter's tabular digits only (see the module docs).
pub static INTER_TABULAR_DIGITS: &[u8] = include_bytes!("../assets/fonts/InterTabularDigits.ttf");

/// The egui font family name for a weight.
pub fn family_name(weight: Weight) -> &'static str {
    match weight {
        Weight::Regular => "beau-inter-400",
        Weight::Medium => "beau-inter-500",
        Weight::Semibold => "beau-inter-600",
        Weight::Display => "beau-inter-600-display",
    }
}

/// The egui [`FontFamily`] for a weight.
pub fn family(weight: Weight) -> FontFamily {
    FontFamily::Name(family_name(weight).into())
}

fn coords(weight: Weight) -> VariationCoords {
    let (wght, opsz) = match weight {
        Weight::Regular => (400.0, 14.0),
        Weight::Medium => (500.0, 14.0),
        Weight::Semibold => (600.0, 14.0),
        Weight::Display => (600.0, 28.0),
    };
    VariationCoords::new([(b"wght", wght), (b"opsz", opsz)])
}

/// Font definitions with beau's families prepended to `base`.
///
/// Anything Inter lacks (emoji, CJK) falls back to the fonts already in `base`.
pub fn font_definitions(mut base: FontDefinitions) -> FontDefinitions {
    let blob: egui::epaint::text::Blob = Arc::new(INTER_VARIABLE);
    let digits: egui::epaint::text::Blob = Arc::new(INTER_TABULAR_DIGITS);
    let fallbacks: Vec<String> = base
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    for weight in [
        Weight::Regular,
        Weight::Medium,
        Weight::Semibold,
        Weight::Display,
    ] {
        let name = family_name(weight).to_owned();
        let tweak = FontTweak {
            coords: coords(weight),
            ..Default::default()
        };
        let data = FontData::from_blob(blob.clone(), 0).tweak(tweak.clone());
        base.font_data.insert(name.clone(), Arc::new(data));
        let digits_name = format!("{name}-tnum");
        let digits_data = FontData::from_blob(digits.clone(), 0).tweak(tweak);
        base.font_data
            .insert(digits_name.clone(), Arc::new(digits_data));
        let mut chain = vec![digits_name, name];
        chain.extend(fallbacks.iter().cloned());
        base.families.insert(family(weight), chain);
    }

    let regular = family_name(Weight::Regular).to_owned();
    for fam in [FontFamily::Proportional, FontFamily::Monospace] {
        let chain = base.families.entry(fam).or_default();
        chain.insert(0, regular.clone());
        chain.insert(0, format!("{regular}-tnum"));
    }
    base
}

/// Install beau's fonts on the context.
pub fn install_fonts(ctx: &Context) {
    ctx.set_fonts(font_definitions(FontDefinitions::default()));
}

impl TextSpec {
    /// The egui [`FontId`] for this spec.
    pub fn font_id(&self) -> FontId {
        FontId::new(self.size, family(self.weight))
    }
}

impl Theme {
    /// The [`FontId`] for a role.
    pub fn font(&self, role: TextRole) -> FontId {
        self.type_scale.get(role).font_id()
    }

    /// Styled text for a role: font, line height and tracking, in
    /// [`crate::tokens::Palette::text`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// use beau::tokens::TextRole;
    /// let t = beau::theme(ui);
    /// ui.label(t.text("Active issues", TextRole::Heading));
    /// # });
    /// ```
    pub fn text(&self, text: impl Into<String>, role: TextRole) -> RichText {
        let spec = self.type_scale.get(role);
        RichText::new(text)
            .font(spec.font_id())
            .line_height(Some(spec.line_height))
            .extra_letter_spacing(spec.tracking)
            .color(self.colors.text)
    }
}
