use egui::{
    Align2, Color32, CornerRadius, FontId, Response, Sense, Stroke, Ui, Widget, WidgetInfo,
    accesskit::Role, pos2, vec2,
};

use crate::{
    motion,
    paint::{self, LinearGradient},
    theme,
    tokens::Weight,
    typography,
};

/// Visual emphasis of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// The single most important action on screen (lime).
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
/// # });
/// ```
#[must_use = "add it with `ui.add(...)`"]
pub struct Button {
    text: String,
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
            trailing: None,
            dot: None,
            variant: Variant::Secondary,
            size: Size::Md,
            full_width: false,
            align_left: false,
            selected: false,
        }
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

        let painter = ui.painter().clone();
        let galley = painter.layout_no_wrap(self.text.clone(), font, Color32::PLACEHOLDER);
        let trailing = self
            .trailing
            .as_ref()
            .map(|s| painter.layout_no_wrap(s.clone(), trailing_font, Color32::PLACEHOLDER));

        let gap = t.space.md;
        let dot_w = if self.dot.is_some() { 8.0 + gap } else { 0.0 };
        let content_w =
            dot_w + galley.size().x + trailing.as_ref().map_or(0.0, |g| gap + g.size().x);
        let width = if self.full_width {
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

        let ink = c.text_on_accent;
        let look = match self.variant {
            Variant::Primary => Look {
                fill: c.accent,
                fill_hover: c.accent_hover,
                border: c.accent.lerp_to_gamma(ink, 0.18),
                text: c.text_on_accent,
                trailing: c.text_on_accent.gamma_multiply(0.55),
                gradient: true,
            },
            Variant::Danger => Look {
                fill: c.danger,
                fill_hover: c.danger_hover,
                border: c.danger.lerp_to_gamma(Color32::BLACK, 0.15),
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

        let radius = CornerRadius::same(t.radius.md);
        let fill = look.fill.lerp_to_gamma(look.fill_hover, hover);
        // Pressing sinks the button by one physical pixel and darkens it slightly.
        let sink = press * paint::hairline_width(ctx.pixels_per_point());
        let body = rect.translate(vec2(0.0, sink));
        let fill = fill.lerp_to_gamma(Color32::BLACK, 0.06 * press);

        let mut shapes = Vec::with_capacity(4);
        shapes.push(paint::focus_ring(rect, radius, c.focus_ring, focus));
        if look.gradient {
            let top = fill.lerp_to_gamma(Color32::WHITE, 0.12 * (1.0 - press));
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
        let text_pos = pos2(start_x + dot_w, body.center().y - galley.size().y / 2.0);
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
