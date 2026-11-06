//! The design system as data.
//!
//! A [`Theme`] bundles every token: colour [`Palette`], [`TypeScale`],
//! [`Space`], [`Radii`], [`Controls`], [`Elevation`] and [`MotionTokens`].
//! It is installed on the [`egui::Context`] with [`crate::install`] and read back
//! with [`crate::theme`].

mod palette;
mod scale;

use std::sync::Arc;

use egui::{Color32, Context, CornerRadius, Id, Margin, Stroke, TextStyle, Ui, vec2};

pub use palette::Palette;
pub use scale::{
    Controls, Elevation, Level, MotionTokens, Radii, ShadowLayer, Space, TextRole, TextSpec,
    TypeScale, Weight,
};

/// Light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Light mode (the hero mode).
    Light,
    /// Dark mode.
    Dark,
}

/// Every design token beau uses, for one [`Mode`].
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Light or dark.
    pub mode: Mode,
    /// Semantic colours.
    pub colors: Palette,
    /// The type scale.
    pub type_scale: TypeScale,
    /// The spacing scale.
    pub space: Space,
    /// Corner radii.
    pub radius: Radii,
    /// Control heights.
    pub controls: Controls,
    /// Layered shadows.
    pub elevation: Elevation,
    /// Motion timings.
    pub motion: MotionTokens,
}

impl Theme {
    /// Graphite light.
    pub fn light() -> Self {
        Self {
            mode: Mode::Light,
            colors: Palette::light(),
            type_scale: TypeScale::graphite(),
            space: Space::default(),
            radius: Radii::default(),
            controls: Controls::default(),
            elevation: Elevation::graphite(Color32::from_rgb(0x0B, 0x0C, 0x0E), 1.0),
            motion: MotionTokens::default(),
        }
    }

    /// Graphite dark.
    pub fn dark() -> Self {
        Self {
            mode: Mode::Dark,
            colors: Palette::dark(),
            elevation: Elevation::graphite(Color32::BLACK, 3.2),
            ..Self::light()
        }
    }

    /// The theme for a mode.
    pub fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Light => Self::light(),
            Mode::Dark => Self::dark(),
        }
    }

    /// Whether this is a dark theme.
    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }

    /// Hairline stroke in the [`Palette::border`] colour.
    pub fn hairline(&self) -> Stroke {
        Stroke::new(1.0, self.colors.border)
    }

    /// Write the tokens into an [`egui::Style`], so stock egui widgets
    /// (labels, scroll bars, selections) match beau.
    pub fn apply_to_style(&self, style: &mut egui::Style) {
        let c = &self.colors;
        let r = |v: u8| CornerRadius::same(v);

        let mut v = if self.is_dark() {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.override_text_color = None;
        v.weak_text_color = Some(c.text_subtle);
        v.hyperlink_color = c.accent_text;
        v.faint_bg_color = c.surface_hover;
        v.extreme_bg_color = c.surface_raised;
        v.text_edit_bg_color = Some(c.surface_raised);
        v.code_bg_color = c.surface_hover;
        v.error_fg_color = c.danger_text;
        v.panel_fill = c.surface;
        v.window_fill = c.surface_overlay;
        v.window_stroke = Stroke::new(1.0, c.border);
        v.window_corner_radius = r(self.radius.lg);
        v.menu_corner_radius = r(self.radius.lg);
        v.window_shadow = egui::Shadow::NONE;
        v.popup_shadow = egui::Shadow::NONE;
        v.selection.bg_fill = c.selection;
        v.selection.stroke = Stroke::new(1.0, c.text);
        v.text_cursor.stroke = Stroke::new(1.5, c.text);
        v.window_highlight_topmost = false;

        let w = &mut v.widgets;
        for (wv, fill, stroke) in [
            (&mut w.noninteractive, c.surface, c.border),
            (&mut w.inactive, c.surface_raised, c.border_strong),
            (&mut w.hovered, c.surface_raised, c.border_strong),
            (&mut w.active, c.surface_raised, c.border_strong),
            (&mut w.open, c.surface_raised, c.border_strong),
        ] {
            wv.bg_fill = fill;
            wv.weak_bg_fill = fill;
            wv.bg_stroke = Stroke::new(1.0, stroke);
            wv.corner_radius = r(self.radius.md);
            wv.fg_stroke = Stroke::new(1.0, c.text);
            wv.expansion = 0.0;
        }
        w.noninteractive.fg_stroke = Stroke::new(1.0, c.text);
        w.inactive.fg_stroke = Stroke::new(1.0, c.text_muted);
        style.visuals = v;

        let s = &mut style.spacing;
        s.item_spacing = vec2(self.space.md, self.space.sm);
        s.button_padding = vec2(self.space.lg, self.space.xs);
        s.interact_size.y = self.controls.md;
        s.window_margin = Margin::same(self.space.xl as i8);
        s.menu_margin = Margin::same(self.space.xs as i8);
        s.scroll.bar_width = 6.0;
        s.scroll.floating_width = 2.0;

        style.text_styles = [
            (TextStyle::Small, TextRole::Small),
            (TextStyle::Body, TextRole::Body),
            (TextStyle::Button, TextRole::BodyStrong),
            (TextStyle::Heading, TextRole::Heading),
            (TextStyle::Monospace, TextRole::Body),
        ]
        .into_iter()
        .map(|(style, role)| (style, self.font(role)))
        .collect();

        style.interaction.selectable_labels = false;
    }
}

fn theme_id() -> Id {
    Id::unique("beau.theme")
}

/// Install `theme` on the context: store it, and map it onto egui's own style.
///
/// Fonts are installed separately by [`crate::install`].
pub fn set_theme(ctx: &Context, theme: Theme) {
    let egui_theme = if theme.is_dark() {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    };
    ctx.set_theme(egui_theme);
    ctx.all_styles_mut(|style| theme.apply_to_style(style));
    ctx.data_mut(|d| d.insert_temp(theme_id(), Arc::new(theme)));
}

/// Anything a theme can be read from.
pub trait ThemeSource {
    /// The egui context.
    fn context(&self) -> &Context;
}

impl ThemeSource for Context {
    fn context(&self) -> &Context {
        self
    }
}

impl ThemeSource for Ui {
    fn context(&self) -> &Context {
        self.ctx()
    }
}

/// The installed [`Theme`] (Graphite light if none was installed).
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let t = beau::theme(ui);
/// let gap = t.space.md;
/// # });
/// ```
pub fn theme(source: &(impl ThemeSource + ?Sized)) -> Arc<Theme> {
    let ctx = source.context();
    if let Some(t) = ctx.data(|d| d.get_temp::<Arc<Theme>>(theme_id())) {
        return t;
    }
    let t = Arc::new(Theme::light());
    ctx.data_mut(|d| d.insert_temp(theme_id(), t.clone()));
    t
}
