use egui::{
    Color32, CornerRadius, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Widget, WidgetInfo,
    accesskit::Role, pos2, vec2,
};

use crate::{motion, paint, theme};

/// Side of the box, in points.
const BOX: f32 = 14.0;

/// A checkbox, with an optional label and an indeterminate state.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::Checkbox;
/// let mut notify = true;
/// if ui.add(Checkbox::new(&mut notify).label("Notify me")).changed() {
///     // ...
/// }
/// // A parent of partly checked items: shows a dash until clicked.
/// let mut all = false;
/// ui.add(Checkbox::new(&mut all).label("Select all").indeterminate(true));
/// # });
/// ```
///
/// Clicking the box or the label toggles it, as does Space or Enter when it
/// has keyboard focus.
#[must_use = "add it with `ui.add(...)`"]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: String,
    name: String,
    indeterminate: bool,
}

impl<'a> Checkbox<'a> {
    /// A checkbox bound to `checked`.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            label: String::new(),
            name: String::new(),
            indeterminate: false,
        }
    }

    /// Text after the box. It is part of the click target.
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

    /// Show a dash instead of a check while the value is `false`, for a
    /// parent whose children are only partly checked. Clicking sets the value
    /// to `true`; clear this flag once the children agree.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

impl Widget for Checkbox<'_> {
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
        let height = galley.as_ref().map_or(BOX, |g| g.size().y).max(20.0);
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(BOX + label_w, height), Sense::click());
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let checked = *self.checked;
        let mixed = self.indeterminate && !checked;
        let name = if self.label.is_empty() {
            &self.name
        } else {
            &self.label
        };
        response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, checked, name));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let ctx = ui.ctx();
        let id = response.id;
        let on = motion::toggle(ctx, id.with("beau.on"), checked || mixed, t.motion.fast);
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

        let (fill, border, mark, text) = if enabled {
            let off_border = c
                .border_strong
                .lerp_to_gamma(c.text_subtle, 0.6 * hover.max(press));
            let on_fill = c.accent.lerp_to_gamma(c.accent_hover, hover);
            (
                c.surface_raised
                    .lerp_to_gamma(on_fill, on)
                    .lerp_to_gamma(Color32::BLACK, 0.06 * press),
                off_border.lerp_to_gamma(on_fill, on),
                c.text_on_accent,
                c.text,
            )
        } else {
            // Neutral and fully opaque, like a disabled button.
            painter.multiply_opacity(1.0 / ui.visuals().disabled_alpha().max(0.01));
            let fill = c.surface_raised.blend(c.surface_active);
            (fill, c.border, c.text_subtle, c.text_subtle)
        };

        let ppp = ctx.pixels_per_point();
        let boxed = paint::snap_rect(
            Rect::from_center_size(
                pos2(rect.left() + BOX / 2.0, rect.center().y),
                vec2(BOX, BOX),
            ),
            ppp,
        );
        let radius = CornerRadius::same(t.radius.sm);
        painter.add(paint::focus_ring(boxed, radius, c.focus_ring, focus));
        painter.add(paint::surface(
            boxed,
            radius,
            fill,
            Stroke::new(1.0, border),
        ));

        if on > 0.0 {
            let at = |x: f32, y: f32| boxed.min + vec2(x, y);
            let points = if mixed {
                vec![at(4.0, 7.0), at(10.0, 7.0)]
            } else {
                vec![at(3.5, 7.25), at(6.0, 9.75), at(10.5, 4.5)]
            };
            // The mark draws itself in from its start as the box fills.
            let stroke = Stroke::new(paint::ICON_STROKE, mark.gamma_multiply(on.min(1.0)));
            painter.add(Shape::line(trim(&points, on.clamp(0.0, 1.0)), stroke));
        }

        if let Some(galley) = galley {
            let pos = pos2(
                rect.left() + BOX + t.space.md,
                rect.center().y - galley.size().y / 2.0,
            );
            painter.galley(pos, galley, text);
        }
        response
    }
}

/// The first `fraction` (by length) of a polyline.
fn trim(points: &[Pos2], fraction: f32) -> Vec<Pos2> {
    let total: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
    let mut left = total * fraction;
    let mut out = vec![points[0]];
    for w in points.windows(2) {
        let len = w[0].distance(w[1]);
        if len <= left {
            out.push(w[1]);
            left -= len;
        } else {
            out.push(w[0] + (w[1] - w[0]) * (left / len.max(f32::EPSILON)));
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_cuts_by_length() {
        let line = [pos2(0.0, 0.0), pos2(4.0, 0.0), pos2(4.0, 4.0)];
        assert_eq!(trim(&line, 1.0), line.to_vec());
        assert_eq!(trim(&line, 0.25), vec![pos2(0.0, 0.0), pos2(2.0, 0.0)]);
        assert_eq!(
            trim(&line, 0.75),
            vec![pos2(0.0, 0.0), pos2(4.0, 0.0), pos2(4.0, 2.0)]
        );
    }
}
