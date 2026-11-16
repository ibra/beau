use egui::{
    Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, Ui, Widget, WidgetInfo,
    accesskit::Role, pos2, vec2,
};

use crate::{
    Icon, motion,
    paint::{self, LinearGradient},
    theme,
    tokens::Weight,
    typography,
};

/// Visual emphasis of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// The single most important action on screen (accent fill; ink by default).
    Primary,
    /// A regular action.
    #[default]
    Secondary,
    /// Low-emphasis action, no chrome until hovered.
    Ghost,
    /// Destructive action.
    Danger,
}

/// Control size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// 24px tall.
    Sm,
    /// 28px tall (default).
    #[default]
    Md,
    /// 32px tall.
    Lg,
}

/// A button.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::{Button, Size};
/// if ui.add(Button::new("Create").primary().trailing("⌘↵")).clicked() {
///     // ...
/// }
/// ui.add(Button::new("Cancel").ghost().size(Size::Sm));
/// ui.add(Button::new("Delete").danger());
/// ui.add(Button::new("New task").icon(beau::icons::PLUS));
/// ui.add(Button::icon_only(beau::icons::SEARCH, "Search").ghost());
/// # });
/// ```
#[must_use = "add it with `ui.add(...)`"]
pub struct Button {
    text: String,
    icon: Option<Icon>,
    icon_only: bool,
    trailing: Option<String>,
    dot: Option<Color32>,
    variant: Variant,
    size: Size,
    full_width: bool,
    align_left: bool,
    selected: bool,
}

impl Button {
    /// A secondary button with a label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            icon_only: false,
            trailing: None,
            dot: None,
            variant: Variant::Secondary,
            size: Size::Md,
            full_width: false,
            align_left: false,
            selected: false,
        }
    }

    /// A square button showing only an icon. `label` is not drawn; it names
    /// the button for screen readers.
    pub fn icon_only(icon: Icon, label: impl Into<String>) -> Self {
        let mut button = Self::new(label);
        button.icon = Some(icon);
        button.icon_only = true;
        button
    }

    /// An icon before the label. It takes the label's colour and a size that
    /// suits the button, unless set on the icon itself.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the variant.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    /// [`Variant::Primary`].
    pub fn primary(self) -> Self {
        self.variant(Variant::Primary)
    }

    /// [`Variant::Secondary`].
    pub fn secondary(self) -> Self {
        self.variant(Variant::Secondary)
    }

    /// [`Variant::Ghost`].
    pub fn ghost(self) -> Self {
        self.variant(Variant::Ghost)
    }

    /// [`Variant::Danger`].
    pub fn danger(self) -> Self {
        self.variant(Variant::Danger)
    }

    /// Set the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Muted trailing text, typically a keyboard shortcut hint or a count.
    pub fn trailing(mut self, text: impl Into<String>) -> Self {
        self.trailing = Some(text.into());
        self
    }

    /// A small coloured dot before the label (projects, statuses, presence).
    pub fn dot(mut self, color: Color32) -> Self {
        self.dot = Some(color);
        self
    }

    /// Fill the available width (label aligned left, trailing text right).
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self.align_left = true;
        self
    }

    /// Show the button as selected (for toggles and navigation).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

struct Look {
    fill: Color32,
    fill_hover: Color32,
    border: Color32,
    text: Color32,
    trailing: Color32,
    gradient: bool,
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme(ui);
        let c = &t.colors;
        let (height, pad, font) = match self.size {
            Size::Sm => (
                t.controls.sm,
                t.space.md,
                FontId::new(12.0, typography::family(Weight::Medium)),
            ),
            Size::Md => (
                t.controls.md,
                t.space.lg,
                t.type_scale.body_strong.font_id(),
            ),
            Size::Lg => (t.controls.lg, 14.0, t.type_scale.body_strong.font_id()),
        };
        let trailing_font = FontId::new(12.0, typography::family(Weight::Regular));

        let mut painter = ui.painter().clone();
        let label = if self.icon_only { "" } else { &self.text };
        let galley = painter.layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
        let trailing = self
            .trailing
            .as_ref()
            .map(|s| painter.layout_no_wrap(s.clone(), trailing_font, Color32::PLACEHOLDER));

        let gap = t.space.md;
        let dot_w = if self.dot.is_some() { 8.0 + gap } else { 0.0 };
        let icon_size = self.icon.map(|icon| {
            icon.explicit_size().unwrap_or(match self.size {
                Size::Sm => t.controls.icon - 2.0,
                Size::Md | Size::Lg => t.controls.icon,
            })
        });
        let icon_w = match icon_size {
            Some(size) if self.icon_only => size,
            Some(size) => size + t.space.sm,
            None => 0.0,
        };
        let content_w =
            dot_w + icon_w + galley.size().x + trailing.as_ref().map_or(0.0, |g| gap + g.size().x);
        let width = if self.icon_only {
            height
        } else if self.full_width {
            ui.available_width()
        } else {
            (content_w + 2.0 * pad).max(height)
        };
        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        let enabled = ui.is_enabled();
        response
            .widget_info(|| WidgetInfo::selected(Role::Button, enabled, self.selected, &self.text));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let ctx = ui.ctx();
        let id = response.id;
        let hover = motion::toggle(
            ctx,
            id.with("beau.hover"),
            enabled && response.hovered(),
            t.motion.fast,
        );
        let press = motion::toggle(
            ctx,
            id.with("beau.press"),
            enabled && response.is_pointer_button_down_on(),
            t.motion.fast,
        );
        let focus = motion::toggle(
            ctx,
            id.with("beau.focus"),
            response.has_focus(),
            t.motion.base,
        );

        // A filled edge only where the fill is light enough to need one.
        let edge = |fill: Color32| {
            if crate::tokens::luminance(fill) < 0.18 {
                fill
            } else {
                fill.lerp_to_gamma(Color32::BLACK, 0.16)
            }
        };
        let look = match self.variant {
            Variant::Primary => Look {
                fill: c.accent,
                fill_hover: c.accent_hover,
                border: edge(c.accent),
                text: c.text_on_accent,
                trailing: c.text_on_accent.gamma_multiply(0.55),
                gradient: true,
            },
            Variant::Danger => Look {
                fill: c.danger,
                fill_hover: c.danger_hover,
                border: edge(c.danger),
                text: c.text_on_danger,
                trailing: c.text_on_danger.gamma_multiply(0.7),
                gradient: true,
            },
            Variant::Secondary => Look {
                fill: c.surface_raised,
                fill_hover: c.surface_raised.blend(c.surface_hover),
                border: c.border_strong,
                text: c.text,
                trailing: c.text_subtle,
                gradient: false,
            },
            Variant::Ghost => {
                let base = if self.selected {
                    c.surface_active
                } else {
                    Color32::TRANSPARENT
                };
                Look {
                    fill: base,
                    fill_hover: if self.selected {
                        c.surface_active
                    } else {
                        c.surface_hover
                    },
                    border: Color32::TRANSPARENT,
                    text: if self.selected {
                        c.text
                    } else {
                        c.text_muted.lerp_to_gamma(c.text, hover)
                    },
                    trailing: c.text_subtle,
                    gradient: false,
                }
            }
        };

        // Disabled: a neutral, full-opacity look that reads as "unavailable",
        // instead of egui's default fade.
        let look = if enabled {
            look
        } else {
            painter.multiply_opacity(1.0 / ui.visuals().disabled_alpha().max(0.01));
            let filled = self.variant != Variant::Ghost;
            Look {
                fill: if filled {
                    c.surface_raised.blend(c.surface_active)
                } else {
                    Color32::TRANSPARENT
                },
                fill_hover: if filled {
                    c.surface_raised.blend(c.surface_active)
                } else {
                    Color32::TRANSPARENT
                },
                border: Color32::TRANSPARENT,
                text: c.text_subtle,
                trailing: c.text_subtle.gamma_multiply(0.7),
                gradient: false,
            }
        };

        let radius = CornerRadius::same(t.radius.md);
        let fill = look.fill.lerp_to_gamma(look.fill_hover, hover);
        // Pressing sinks the button by one physical pixel and darkens it slightly.
        let sink = press * paint::hairline_width(ctx.pixels_per_point());
        let body = rect.translate(vec2(0.0, sink));
        let fill = fill.lerp_to_gamma(Color32::BLACK, 0.06 * press);

        let mut shapes = Vec::with_capacity(4);
        shapes.push(paint::focus_ring(rect, radius, c.focus_ring, focus));
        if look.gradient {
            let top = fill.lerp_to_gamma(Color32::WHITE, 0.08 * (1.0 - press));
            let gradient = LinearGradient::vertical(top, fill);
            shapes.push(paint::gradient_rect(ctx, body, radius, &gradient));
            shapes.push(paint::surface(
                body,
                radius,
                Color32::TRANSPARENT,
                Stroke::new(1.0, look.border),
            ));
        } else if fill != Color32::TRANSPARENT || look.border != Color32::TRANSPARENT {
            shapes.push(paint::surface(
                body,
                radius,
                fill,
                Stroke::new(1.0, look.border),
            ));
        }
        painter.extend(shapes);

        let start_x = if self.align_left {
            body.left() + pad
        } else {
            body.center().x - content_w / 2.0
        };
        if let Some(dot) = self.dot {
            painter.circle_filled(pos2(start_x + 4.0, body.center().y), 3.5, dot);
        }
        if let (Some(icon), Some(size)) = (self.icon, icon_size) {
            let center = pos2(start_x + dot_w + size / 2.0, body.center().y);
            let color = icon.explicit_color().unwrap_or(look.text);
            icon.paint_at(&painter, center, size, color);
        }
        let text_pos = pos2(
            start_x + dot_w + icon_w,
            body.center().y - galley.size().y / 2.0,
        );
        let text_w = galley.size().x;
        painter.galley(text_pos, galley, look.text);

        if let Some(g) = trailing {
            let pos = if self.full_width {
                Align2::RIGHT_CENTER
                    .anchor_size(pos2(body.right() - pad, body.center().y), g.size())
                    .min
            } else {
                pos2(
                    text_pos.x + text_w + gap,
                    body.center().y - g.size().y / 2.0,
                )
            };
            painter.galley(pos, g, look.trailing);
        }

        response
    }
}
