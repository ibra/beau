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

pub use palette::{Palette, contrast, luminance};
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
    /// The light theme.
    pub fn light() -> Self {
        Self {
            mode: Mode::Light,
            colors: Palette::light(),
            type_scale: TypeScale::default(),
            space: Space::default(),
            radius: Radii::default(),
            controls: Controls::default(),
            elevation: Elevation::from_ink(Color32::from_rgb(0x0C, 0x0C, 0x0C), 1.0),
            motion: MotionTokens::default(),
        }
    }

    /// The dark theme.
    pub fn dark() -> Self {
        Self {
            mode: Mode::Dark,
            colors: Palette::dark(),
            elevation: Elevation::from_ink(Color32::BLACK, 3.2),
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

    /// Use `accent` as the primary colour.
    ///
    /// beau's default accent is ink (monochrome). Pass your brand colour here
    /// and every accent-derived token (hover, on-accent text, accent text, focus
    /// ring, selection) is recomputed with WCAG contrast guarantees.
    ///
    /// ```
    /// let theme = beau::Theme::light().with_accent(egui::Color32::from_rgb(0x3B, 0x82, 0xF6));
    /// ```
    pub fn with_accent(mut self, accent: Color32) -> Self {
        self.colors.set_accent(accent);
        self
    }

    /// Whether this is a dark theme.
    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }

    /// Divider and surface-outline stroke: exactly one physical pixel in
    /// [`Palette::border`], so it stays fine on high-DPI screens.
    pub fn divider(&self, ctx: &Context) -> Stroke {
        Stroke::new(
            crate::paint::hairline_width(ctx.pixels_per_point()),
            self.colors.border,
        )
    }

    /// Control outline stroke (inputs, secondary buttons): 1pt, so controls
    /// read as interactive at any density.
    pub fn control_border(&self) -> Stroke {
        Stroke::new(1.0, self.colors.border_strong)
    }

    /// Write the tokens into an [`egui::Style`], so stock egui widgets
    /// (buttons, checkboxes, radios, sliders, combo boxes, drag values, text
    /// edits, scroll bars, tooltips, menus and windows) match beau.
    ///
    /// egui's style has one slot per widget state, shared by every widget, so
    /// a few details can't follow beau exactly. A checked checkbox keeps the
    /// raised fill (only the check mark is ink), and floating surfaces get a
    /// single shadow layer instead of beau's layered elevation. Disabled stock
    /// widgets use egui's faded look rather than beau's neutral one. Pass
    /// [`crate::paint::select_chevrons`] to `ComboBox::icon` for beau's
    /// chevrons.
    pub fn apply_to_style(&self, style: &mut egui::Style) {
        let c = &self.colors;
        let r = |v: u8| CornerRadius::same(v);
        let stroke = |color: Color32| Stroke::new(1.0, color);

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
        v.warn_fg_color = c.text;
        v.error_fg_color = c.danger_text;
        v.panel_fill = c.surface;
        v.window_fill = c.surface_overlay;
        v.window_stroke = stroke(c.border);
        v.window_corner_radius = r(self.radius.lg);
        v.menu_corner_radius = r(self.radius.md);
        v.window_shadow = egui_shadow(&self.elevation.modal);
        v.popup_shadow = egui_shadow(&self.elevation.overlay);
        v.selection.bg_fill = c.selection;
        // Also the focused text-edit border, so it follows the focus ring.
        v.selection.stroke = stroke(c.focus_ring);
        v.text_cursor.stroke = Stroke::new(1.5, c.text);
        v.window_highlight_topmost = false;
        v.collapsing_header_frame = false;
        v.indent_has_left_vline = false;
        v.striped = false;
        v.handle_shape = egui::style::HandleShape::Circle;
        v.slider_trailing_fill = true;
        v.interact_cursor = None;

        // Opaque, so slider rails and handles don't show through each other.
        let hover = c.surface_raised.blend(c.surface_hover);
        let press = c.surface_raised.blend(c.surface_active);
        let w = &mut v.widgets;
        // Most stock widgets are drawn by `crate::stock::StockStyle`; these
        // slots style the rest. `bg_fill` is the inside of radios and slider
        // handles and, when inactive, the slider rail: a soft gray, so the
        // rail shows on raised surfaces, turning white under the pointer.
        // `weak_bg_fill` frames buttons when no StockStyle is registered.
        // `fg_stroke` is radio labels and dots and slider handle outlines.
        for (wv, fill, weak, fg) in [
            (&mut w.inactive, press, c.surface_raised, c.text),
            (&mut w.hovered, c.surface_raised, hover, c.text),
            (&mut w.active, c.surface_raised, press, c.text),
            (&mut w.open, c.surface_raised, press, c.text),
        ] {
            wv.bg_fill = fill;
            wv.weak_bg_fill = weak;
            wv.bg_stroke = stroke(c.border_strong);
            wv.fg_stroke = stroke(fg);
            wv.corner_radius = r(self.radius.md);
            wv.expansion = 0.0;
        }
        w.noninteractive.bg_fill = c.surface;
        w.noninteractive.weak_bg_fill = c.surface;
        // Separators and grid lines.
        w.noninteractive.bg_stroke = stroke(c.border);
        w.noninteractive.fg_stroke = stroke(c.text);
        w.noninteractive.corner_radius = r(self.radius.md);
        w.noninteractive.expansion = 0.0;
        style.visuals = v;

        let s = &mut style.spacing;
        s.item_spacing = vec2(self.space.md, self.space.xs);
        s.button_padding = vec2(self.space.lg, self.space.xs);
        s.interact_size.y = self.controls.md;
        s.window_margin = Margin::same(self.space.xl as i8);
        // Shared by menus and tooltips: menu rows add their own padding.
        s.menu_margin = Margin::symmetric(self.space.sm as i8, self.space.xs as i8);
        s.menu_width = self.controls.md * 7.0;
        s.tooltip_width = self.controls.md * 10.0;
        // Checkboxes and radios: half a control. egui caps the box radius at
        // 30% of this, which lands on `radius.sm`.
        s.icon_width = self.controls.md / 2.0;
        s.icon_width_inner = self.space.md;
        s.icon_spacing = self.space.md;
        s.slider_rail_height = self.space.xs;
        // Default widths, in control heights so density changes carry over.
        s.slider_width = self.controls.md * 6.0;
        s.combo_width = self.controls.md * 6.0;
        s.text_edit_width = self.controls.md * 8.0;
        s.indent = self.space.xl;
        s.scroll = egui::style::ScrollStyle {
            floating: true,
            bar_width: self.space.sm,
            floating_width: self.space.xxs,
            floating_allocated_width: 0.0,
            handle_min_length: self.controls.md,
            bar_inner_margin: self.space.xs,
            bar_outer_margin: self.space.xxs,
            // Ink handle, no track.
            foreground_color: true,
            dormant_background_opacity: 0.0,
            active_background_opacity: 0.0,
            interact_background_opacity: 0.0,
            dormant_handle_opacity: 0.0,
            active_handle_opacity: 0.25,
            interact_handle_opacity: 0.45,
            ..egui::style::ScrollStyle::floating()
        };

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

/// egui draws one shadow per surface: use the widest of beau's layers, which
/// carries most of the elevation. egui can't shrink a shadow, so a negative
/// spread is dropped.
fn egui_shadow(layers: &[ShadowLayer]) -> egui::Shadow {
    let Some(l) = layers.last() else {
        return egui::Shadow::NONE;
    };
    egui::Shadow {
        offset: [l.offset.x as i8, l.offset.y as i8],
        blur: l.blur as u8,
        spread: l.spread.max(0.0) as u8,
        color: l.color,
    }
}

fn theme_id(mode: Mode) -> Id {
    match mode {
        Mode::Light => Id::unique("beau.theme.light"),
        Mode::Dark => Id::unique("beau.theme.dark"),
    }
}

fn egui_theme(mode: Mode) -> egui::Theme {
    match mode {
        Mode::Light => egui::Theme::Light,
        Mode::Dark => egui::Theme::Dark,
    }
}

/// Install `theme` as the theme for its [`Mode`], and map it onto egui's style
/// for that mode.
///
/// beau keeps one theme per mode. Which one is active follows egui's theme
/// preference (see [`set_preference`]), so following the OS light/dark setting
/// works out of the box. Fonts are installed separately by [`crate::install`].
///
/// ```
/// # let ctx = egui::Context::default();
/// let brand = egui::Color32::from_rgb(0x3B, 0x82, 0xF6);
/// beau::set_theme(&ctx, beau::Theme::light().with_accent(brand));
/// beau::set_theme(&ctx, beau::Theme::dark().with_accent(brand));
/// ```
pub fn set_theme(ctx: &Context, theme: Theme) {
    let mode = theme.mode;
    ctx.style_mut_of(egui_theme(mode), |style| theme.apply_to_style(style));
    ctx.data_mut(|d| d.insert_temp(theme_id(mode), Arc::new(theme)));
}

/// Choose light, dark, or follow the OS ([`egui::ThemePreference::System`], the default).
pub fn set_preference(ctx: &Context, preference: egui::ThemePreference) {
    ctx.set_theme(preference);
}

/// The [`Mode`] currently in effect (resolving "follow the OS").
pub fn mode(ctx: &Context) -> Mode {
    match ctx.theme() {
        egui::Theme::Light => Mode::Light,
        egui::Theme::Dark => Mode::Dark,
    }
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

/// The [`Theme`] for the mode currently in effect (beau's defaults if none was
/// installed for that mode).
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let t = beau::theme(ui);
/// let gap = t.space.md;
/// # });
/// ```
pub fn theme(source: &(impl ThemeSource + ?Sized)) -> Arc<Theme> {
    let ctx = source.context();
    theme_for_mode(ctx, mode(ctx))
}

/// The installed [`Theme`] for `mode` (beau's defaults if none was installed).
pub(crate) fn theme_for_mode(ctx: &Context, mode: Mode) -> Arc<Theme> {
    if let Some(t) = ctx.data(|d| d.get_temp::<Arc<Theme>>(theme_id(mode))) {
        return t;
    }
    let t = Arc::new(Theme::for_mode(mode));
    ctx.data_mut(|d| d.insert_temp(theme_id(mode), t.clone()));
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Customised tokens reach egui's style, so stock widgets follow them.
    #[test]
    fn apply_to_style_uses_tokens() {
        let mut theme = Theme::light().with_accent(Color32::from_rgb(0x3B, 0x82, 0xF6));
        theme.radius.md = 8;
        theme.controls.md = 32.0;
        let mut style = egui::Style::default();
        theme.apply_to_style(&mut style);
        let w = &style.visuals.widgets;
        assert_eq!(w.inactive.corner_radius, CornerRadius::same(8));
        assert_eq!(style.spacing.interact_size.y, 32.0);
        assert_eq!(style.spacing.icon_width, 16.0);
        assert_eq!(style.visuals.selection.bg_fill, theme.colors.selection);
        assert_eq!(
            style.visuals.selection.stroke.color,
            theme.colors.focus_ring
        );
    }
}
