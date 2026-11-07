//! beau's look for egui's own widgets.
//!
//! [`crate::install`] maps the theme onto [`egui::Style`] (see
//! [`crate::Theme::apply_to_style`]) and registers [`StockStyle`], an egui
//! [`StyleProvider`] that draws the stock widgets egui lets a theme fully
//! style:
//!
//! - **Buttons**, including selectable labels, drag values, combo boxes and
//!   menu items: a raised secondary surface with a 1pt control border and a
//!   contact shadow, hover and press washes, 500-weight labels, a soft focus
//!   halo for keyboard focus, and ghost rows inside menus and popups.
//! - **Text edits**: a raised field whose border darkens on hover and turns
//!   `focus_ring` with a 35% halo when focused, like [`crate::TextInput`].
//! - **Checkboxes**: a 14pt box at `radius.sm` with an ink check.
//! - **Separators**: physical-pixel hairlines.
//!
//! Radios, sliders, scroll bars, tooltips and windows can only be styled
//! through `Visuals`, so they follow beau's colours and sizes but keep egui's
//! geometry. egui gives a style provider no widget id, so stock widgets change
//! state instantly instead of animating like beau's components.

use std::sync::Arc;

use egui::class::HasClasses as _;
pub use egui::theme::StyleProvider;
use egui::{
    Align2, Color32, Context, CornerRadius, Frame, Margin, Shadow, Stroke, Style, UiKind, vec2,
    widget_style::{
        AtomLayoutStyle, ButtonStyle, CheckboxStyle, SeparatorStyle, StyleArgs, TextEditStyle,
        TextVisuals, WidgetState,
    },
};

use crate::tokens::{Level, Mode, ShadowLayer, Theme};

/// The [`StyleProvider`] that gives stock egui widgets beau's look.
///
/// Registered by [`crate::install`]. It reads the installed [`Theme`] for the
/// widget's mode on every call, so [`crate::set_theme`] and accent changes
/// apply immediately.
///
/// ```
/// # let ctx = egui::Context::default();
/// beau::install(&ctx); // registers StockStyle
/// beau::stock::StockStyle::register(&ctx); // or on its own
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct StockStyle;

impl StockStyle {
    /// Register `StockStyle` for every widget style egui lets a theme provide,
    /// replacing egui's default provider.
    pub fn register(ctx: &Context) {
        ctx.replace_widget_theme::<ButtonStyle>(Self);
        ctx.replace_widget_theme::<TextEditStyle>(Self);
        ctx.replace_widget_theme::<CheckboxStyle>(Self);
        ctx.replace_widget_theme::<SeparatorStyle>(Self);
    }
}

/// The theme matching the style being drawn (a ui can force light or dark).
fn theme_of(ctx: &Context, style: &Style) -> Arc<Theme> {
    let mode = if style.visuals.dark_mode {
        Mode::Dark
    } else {
        Mode::Light
    };
    crate::tokens::theme_for_mode(ctx, mode)
}

/// Whether the widget sits in a menu, combo box list or other popup.
fn in_popup(args: &StyleArgs<'_>) -> bool {
    [UiKind::Menu, UiKind::Popup]
        .into_iter()
        .any(|kind| args.stack.contained_in(kind))
}

/// `Active` means pressed or focused. egui only focuses buttons from the
/// keyboard, so an active widget without a pointer press is keyboard-focused.
fn is_pressed(ctx: &Context) -> bool {
    ctx.input(|i| i.pointer.any_down() || i.pointer.any_released())
}

fn shadow(layer: Option<&ShadowLayer>) -> Shadow {
    layer.map_or(Shadow::NONE, |l| Shadow {
        offset: [l.offset.x as i8, l.offset.y as i8],
        blur: l.blur as u8,
        spread: l.spread.max(0.0) as u8,
        color: l.color,
    })
}

/// beau's focus treatment for frames: a solid 3pt halo at 35% of `focus_ring`.
fn halo(t: &Theme) -> Shadow {
    Shadow {
        offset: [0, 0],
        blur: 0,
        spread: 3,
        color: t.colors.focus_ring.gamma_multiply(0.35),
    }
}

fn margin(x: f32, y: f32) -> Margin {
    Margin::symmetric(x as i8, y as i8)
}

impl StyleProvider<ButtonStyle> for StockStyle {
    fn style(&mut self, args: &StyleArgs<'_>) -> ButtonStyle {
        let t = theme_of(args.ctx, args.style);
        let c = &t.colors;
        let classes = args.classes;
        let state = args.state;
        let small = classes.has_class(&egui::Button::CLASS_SMALL);
        let selected = classes.has_class(&egui::Button::CLASS_SELECTED);
        let open = classes.has_class(&egui::ComboBox::CLASS_OPEN);
        // Value fields read as fields (regular weight), not as buttons.
        let field =
            classes.has_class(&egui::ComboBox::CLASS) || classes.has_class(&egui::DragValue::CLASS);
        let pressed = state == WidgetState::Active && is_pressed(args.ctx);
        let focused = (state == WidgetState::Active && !pressed) || open;

        let framed = classes.has_class(&egui::Button::CLASS_FRAME)
            || (!classes.has_class(&egui::Button::CLASS_NO_FRAME)
                && args.style.visuals.button_frame);
        let ghost = !framed || in_popup(args);
        let hide_when_inactive = classes.has_class(&egui::Button::CLASS_HIDE_FRAME_WHEN_INACTIVE)
            && state == WidgetState::Inactive;

        // Menu rows are compact, like small buttons.
        let (height, pad) = if small || in_popup(args) {
            (t.controls.sm, margin(t.space.md, 0.0))
        } else {
            (t.controls.md, margin(t.space.lg, 0.0))
        };

        let mut frame = Frame::new().inner_margin(pad);
        let mut text = c.text;
        if ghost {
            // Rows in menus and lists: no frame until hovered or selected.
            let fill = if selected || pressed {
                c.surface_active
            } else if matches!(state, WidgetState::Hovered | WidgetState::Active) {
                c.surface_hover
            } else {
                Color32::TRANSPARENT
            };
            frame = frame
                .fill(fill)
                .corner_radius(CornerRadius::same(t.radius.sm));
            if focused && !in_popup(args) {
                frame = frame.shadow(halo(&t));
            }
            if !selected && state == WidgetState::Inactive && !in_popup(args) {
                text = c.text_muted;
            }
        } else if !hide_when_inactive {
            let raised = c.surface_raised;
            let fill = if pressed || selected {
                raised.blend(c.surface_active)
            } else if state == WidgetState::Hovered || focused {
                raised.blend(c.surface_hover)
            } else {
                raised
            };
            let border = if focused {
                c.focus_ring
            } else {
                c.border_strong
            };
            frame = frame
                .fill(fill)
                .stroke(Stroke::new(1.0, border))
                .corner_radius(CornerRadius::same(t.radius.md))
                .shadow(if focused {
                    halo(&t)
                } else if pressed {
                    Shadow::NONE
                } else {
                    shadow(t.elevation.get(Level::Raised).first())
                });
        }
        if state == WidgetState::Noninteractive {
            text = c.text_subtle;
        }

        ButtonStyle {
            atom_layout: AtomLayoutStyle {
                min_size: if small && !in_popup(args) {
                    vec2(0.0, 0.0)
                } else {
                    vec2(0.0, height)
                },
                gap: t.space.sm,
                frame: frame.apply_stroke_and_expansion_without_layout_shift(frame.stroke, 0.0),
                text_style: TextVisuals::new(
                    args.style,
                    if field {
                        t.type_scale.body.font_id()
                    } else {
                        t.type_scale.body_strong.font_id()
                    },
                    text,
                ),
                image_tint: if classes.has_class(&egui::Button::CLASS_IMAGE_TINT_FOLLOWS_TEXT_COLOR)
                {
                    text
                } else {
                    Color32::WHITE
                },
                ..Default::default()
            },
        }
    }
}

impl StyleProvider<TextEditStyle> for StockStyle {
    fn style(&mut self, args: &StyleArgs<'_>) -> TextEditStyle {
        let t = theme_of(args.ctx, args.style);
        let c = &t.colors;
        let read_only = args.classes.has_class(&egui::TextEdit::CLASS_READ_ONLY);
        let focused = args.state == WidgetState::Active && !read_only;
        let border = match args.state {
            _ if read_only => c.border,
            WidgetState::Active => c.focus_ring,
            WidgetState::Hovered => c.border_strong.lerp_to_gamma(c.text_subtle, 0.6),
            _ => c.border_strong,
        };
        let frame = Frame::new()
            .fill(if read_only {
                Color32::TRANSPARENT
            } else {
                c.surface_raised
            })
            .stroke(Stroke::new(1.0, border))
            .corner_radius(CornerRadius::same(t.radius.md))
            .inner_margin(margin(t.space.lg - 2.0, t.space.xs))
            .shadow(if focused { halo(&t) } else { Shadow::NONE });
        TextEditStyle {
            atom_layout: AtomLayoutStyle {
                frame: frame.apply_stroke_and_expansion_without_layout_shift(frame.stroke, 0.0),
                min_size: vec2(0.0, t.controls.md),
                gap: t.space.sm,
                text_style: TextVisuals::new(args.style, t.type_scale.body.font_id(), c.text),
                align2: Some(Align2::LEFT_TOP),
                ..Default::default()
            },
            hint_text_color: c.text_subtle,
            prefix_suffix_color: c.text_muted,
        }
    }
}

impl StyleProvider<CheckboxStyle> for StockStyle {
    fn style(&mut self, args: &StyleArgs<'_>) -> CheckboxStyle {
        let t = theme_of(args.ctx, args.style);
        let c = &t.colors;
        let state = args.state;
        let pressed = state == WidgetState::Active && is_pressed(args.ctx);
        let focused = state == WidgetState::Active && !pressed;
        let border = match state {
            WidgetState::Hovered | WidgetState::Active => {
                c.border_strong.lerp_to_gamma(c.text_subtle, 0.6)
            }
            _ => c.border_strong,
        };
        let fill = if pressed {
            c.surface_raised.blend(c.surface_active)
        } else {
            c.surface_raised
        };
        let text = if state == WidgetState::Noninteractive {
            c.text_subtle
        } else {
            c.text
        };
        let size = args.style.spacing.icon_width;
        CheckboxStyle {
            atom_layout: AtomLayoutStyle {
                min_size: vec2(0.0, args.style.spacing.interact_size.y),
                gap: t.space.md,
                frame: Frame::new(),
                text_style: TextVisuals::new(args.style, t.type_scale.body.font_id(), text),
                ..Default::default()
            },
            checkbox_size: size,
            check_size: args.style.spacing.icon_width_inner,
            checkbox_frame: Frame::new()
                .fill(fill)
                .stroke(Stroke::new(1.0, border))
                .corner_radius(CornerRadius::same(t.radius.sm))
                .shadow(if focused {
                    halo(&t)
                } else {
                    shadow(t.elevation.get(Level::Raised).first())
                }),
            check_stroke: Stroke::new(1.75, c.text),
        }
    }
}

impl StyleProvider<SeparatorStyle> for StockStyle {
    fn style(&mut self, args: &StyleArgs<'_>) -> SeparatorStyle {
        let t = theme_of(args.ctx, args.style);
        SeparatorStyle {
            spacing: if in_popup(args) {
                t.space.xs
            } else {
                t.space.md
            },
            stroke: t.divider(args.ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{CornerRadius, class::Classes, widget_style::ButtonStyle};

    use crate::Theme;

    /// Token changes flow through to stock widgets.
    #[test]
    fn follows_theme_tokens() {
        let ctx = egui::Context::default();
        crate::install(&ctx);
        let mut theme = Theme::light();
        theme.radius.md = 8;
        theme.controls.md = 32.0;
        crate::set_theme(&ctx, theme);
        crate::set_preference(&ctx, egui::ThemePreference::Light);
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let id = ui.next_auto_id();
            let s: ButtonStyle = ui.widget_style(id, &Classes::default());
            assert_eq!(s.atom_layout.frame.corner_radius, CornerRadius::same(8));
            assert_eq!(s.atom_layout.min_size.y, 32.0);
        });
        output.textures_delta.clear();
    }
}
