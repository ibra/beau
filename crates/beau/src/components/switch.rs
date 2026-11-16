use egui::{
    Color32, CornerRadius, Rect, Response, Sense, Shape, Ui, Widget, WidgetInfo, accesskit::Role,
    epaint::RectShape, pos2, vec2,
};

use crate::{Mode, motion, paint, theme};

/// Track size, in points.
const TRACK: egui::Vec2 = vec2(28.0, 16.0);
/// Knob diameter, in points.
const KNOB: f32 = 12.0;

/// An on/off switch that takes effect immediately, with an optional label.
///
/// Use a switch for settings that apply at once, and a
/// [`Checkbox`](crate::Checkbox) for choices confirmed later (forms, bulk
/// selection).
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::Switch;
/// let mut dark = false;
/// if ui.add(Switch::new(&mut dark).label("Dark mode")).changed() {
///     // ...
/// }
/// # });
/// ```
///
/// Clicking the track or the label toggles it, as does Space or Enter when
/// it has keyboard focus.
#[must_use = "add it with `ui.add(...)`"]
pub struct Switch<'a> {
    on: &'a mut bool,
    label: String,
    name: String,
}

impl<'a> Switch<'a> {
    /// A switch bound to `on`.
    pub fn new(on: &'a mut bool) -> Self {
        Self {
            on,
            label: String::new(),
            name: String::new(),
        }
    }

    /// Text after the track. It is part of the click target.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// A name for screen readers when there is no visible [`label`](Self::label),
    /// e.g. when the text sits elsewhere in the row. It is not drawn.
    pub fn accessible_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl Widget for Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme(ui);
        let c = &t.colors;
        let enabled = ui.is_enabled();

        let mut painter = ui.painter().clone();
        let galley = (!self.label.is_empty()).then(|| {
            painter.layout_no_wrap(
                self.label.clone(),
                t.type_scale.body.font_id(),
                Color32::PLACEHOLDER,
            )
        });
        let label_w = galley.as_ref().map_or(0.0, |g| t.space.md + g.size().x);
        let height = galley.as_ref().map_or(TRACK.y, |g| g.size().y).max(20.0);
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(TRACK.x + label_w, height), Sense::click());
        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }
        let is_on = *self.on;
        let name = if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        };
        response.widget_info(|| WidgetInfo::selected(Role::Switch, enabled, is_on, name));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let ctx = ui.ctx();
        let id = response.id;
        let on = motion::toggle(ctx, id.with("beau.on"), is_on, t.motion.base);
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

        let (track, knob, text) = if enabled {
            // The off track must show on every surface, so it is darker than
            // a control border.
            let off = c
                .border_strong
                .lerp_to_gamma(c.text_subtle, 0.45 + 0.2 * hover);
            let on_fill = c.accent.lerp_to_gamma(c.accent_hover, hover);
            let knob_off = match t.mode {
                Mode::Light => c.surface_raised,
                Mode::Dark => c.text,
            };
            (
                off.lerp_to_gamma(on_fill, on.clamp(0.0, 1.0)),
                knob_off.lerp_to_gamma(c.text_on_accent, on.clamp(0.0, 1.0)),
                c.text,
            )
        } else {
            // Neutral and fully opaque, like a disabled button.
            painter.multiply_opacity(1.0 / ui.visuals().disabled_alpha().max(0.01));
            (
                c.surface_raised.blend(c.surface_active),
                c.text_subtle.gamma_multiply(0.7),
                c.text_subtle,
            )
        };

        let track_rect = paint::snap_rect(
            Rect::from_center_size(pos2(rect.left() + TRACK.x / 2.0, rect.center().y), TRACK),
            ctx.pixels_per_point(),
        );
        let radius = CornerRadius::same(t.radius.full);
        painter.add(paint::focus_ring(track_rect, radius, c.focus_ring, focus));
        painter.add(Shape::Rect(RectShape::filled(track_rect, radius, track)));

        // The knob stretches towards the other side while pressed.
        let inset = (TRACK.y - KNOB) / 2.0;
        let width = KNOB + 3.0 * press;
        let travel = TRACK.x - 2.0 * inset - width;
        let left = track_rect.left() + inset + travel * on;
        let knob_rect =
            Rect::from_min_size(pos2(left, track_rect.top() + inset), vec2(width, KNOB));
        painter.add(Shape::Rect(RectShape::filled(knob_rect, radius, knob)));

        if let Some(galley) = galley {
            let pos = pos2(
                rect.left() + TRACK.x + t.space.md,
                rect.center().y - galley.size().y / 2.0,
            );
            painter.galley(pos, galley, text);
        }
        response
    }
}
