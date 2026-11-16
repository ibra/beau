use std::ops::RangeInclusive;

use egui::{
    CornerRadius, EventFilter, Key, Rect, Response, Sense, Shape, Stroke, Ui, Widget, WidgetInfo,
    emath::Numeric, epaint::RectShape, lerp, pos2, remap_clamp, vec2,
};

use crate::{Mode, motion, paint, theme, tokens::Level};

/// Rail thickness, in points.
const RAIL: f32 = 4.0;
/// Knob diameter at rest, in points. It grows by [`KNOB_GROW`] under the pointer.
const KNOB: f32 = 14.0;
const KNOB_GROW: f32 = 2.0;
/// Default rail length, in points.
const WIDTH: f32 = 160.0;

/// A slider for picking a number in a range.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::Slider;
/// let mut volume = 0.4_f32;
/// ui.add(Slider::new(&mut volume, 0.0..=1.0).accessible_name("Volume"));
///
/// // Whole numbers step by one; `show_value` adds an editable number field.
/// let mut minutes = 15_u32;
/// ui.add(Slider::new(&mut minutes, 0..=60).show_value(true).accessible_name("Lead time"));
/// # });
/// ```
///
/// Drag the knob or click the rail. With keyboard focus, Left and Right step
/// the value (by [`Slider::step`], or 1% of the range), and Home and End jump
/// to the ends.
#[must_use = "add it with `ui.add(...)`"]
pub struct Slider<'a> {
    get_set: Box<dyn FnMut(Option<f64>) -> f64 + 'a>,
    range: RangeInclusive<f64>,
    step: Option<f64>,
    integral: bool,
    width: f32,
    show_value: bool,
    name: String,
}

impl<'a> Slider<'a> {
    /// A slider bound to `value`, limited to `range`. Integer types step by
    /// one.
    pub fn new<N: Numeric>(value: &'a mut N, range: RangeInclusive<N>) -> Self {
        Self {
            get_set: Box::new(move |new| {
                if let Some(new) = new {
                    *value = N::from_f64(new);
                }
                value.to_f64()
            }),
            range: range.start().to_f64()..=range.end().to_f64(),
            step: N::INTEGRAL.then_some(1.0),
            integral: N::INTEGRAL,
            width: WIDTH,
            show_value: false,
            name: String::new(),
        }
    }

    /// Snap the value to multiples of `step`, counted from the start of the
    /// range.
    pub fn step(mut self, step: f64) -> Self {
        self.step = (step > 0.0).then_some(step);
        self
    }

    /// Length of the rail in points (160 by default).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Show an editable number field after the rail.
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// A name for screen readers, e.g. the text of the label next to the
    /// slider. It is not drawn.
    pub fn accessible_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    fn snap(&self, value: f64) -> f64 {
        let (min, max) = (*self.range.start(), *self.range.end());
        let value = match self.step {
            Some(step) => min + ((value - min) / step).round() * step,
            None => value,
        };
        value.clamp(min.min(max), max.max(min))
    }
}

impl Widget for Slider<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        if !self.show_value {
            return self.rail(ui);
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme(ui).space.md;
            let mut response = self.rail(ui);
            let mut value = (self.get_set)(None);
            let speed = self
                .step
                .unwrap_or((self.range.end() - self.range.start()).abs() / 100.0);
            let mut field = egui::DragValue::new(&mut value)
                .range(self.range.clone())
                .speed(speed);
            if self.integral {
                field = field.max_decimals(0);
            }
            let field = ui.add(field);
            // egui's number field has no name of its own.
            let (enabled, name) = (ui.is_enabled(), self.name.clone());
            field.widget_info(|| {
                let mut info = WidgetInfo::drag_value(enabled, value);
                info.label = (!name.is_empty()).then(|| name.clone());
                info
            });
            if field.changed() {
                let value = self.snap(value);
                (self.get_set)(Some(value));
                response.mark_changed();
            }
            response | field
        })
        .inner
    }
}

impl Slider<'_> {
    fn rail(&mut self, ui: &mut Ui) -> Response {
        let t = theme(ui);
        let c = &t.colors;
        let enabled = ui.is_enabled();
        let (min, max) = (*self.range.start(), *self.range.end());

        let size = vec2(self.width.max(KNOB + KNOB_GROW), t.controls.sm);
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        // The knob's centre travels between these two x positions, so at
        // either end its edge is flush with the end of the rail.
        let reach = KNOB / 2.0;
        let travel = (rect.left() + reach)..=(rect.right() - reach);

        let old = (self.get_set)(None);
        let mut value = old;
        if let Some(pointer) = response.interact_pointer_pos() {
            value = remap_clamp(f64::from(pointer.x), travel_f64(&travel), min..=max);
        }
        if response.has_focus() {
            // Keep Left/Right for the slider; without this they move focus.
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    EventFilter {
                        horizontal_arrows: true,
                        ..Default::default()
                    },
                );
            });
            let step = self.step.unwrap_or((max - min).abs() / 100.0) * (max - min).signum();
            let (left, right, home, end) = ui.input(|i| {
                (
                    i.num_presses(Key::ArrowLeft) as f64,
                    i.num_presses(Key::ArrowRight) as f64,
                    i.key_pressed(Key::Home),
                    i.key_pressed(Key::End),
                )
            });
            value += (right - left) * step;
            if home {
                value = min;
            }
            if end {
                value = max;
            }
        }
        let value = self.snap(value);
        if value != old {
            (self.get_set)(Some(value));
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::slider(enabled, value, &self.name));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let ctx = ui.ctx();
        let id = response.id;
        let active = enabled && (response.hovered() || response.dragged());
        let grow = motion::toggle(ctx, id.with("beau.hover"), active, t.motion.fast);
        let focus = motion::toggle(
            ctx,
            id.with("beau.focus"),
            response.has_focus(),
            t.motion.base,
        );

        let mut painter = ui.painter().clone();
        let knob_rest = match t.mode {
            Mode::Light => c.surface_raised,
            Mode::Dark => c.text,
        };
        let (rail, fill, knob, outline) = if enabled {
            (c.border_strong, c.accent, knob_rest, c.border_strong)
        } else {
            // Neutral and fully opaque, like a disabled button.
            painter.multiply_opacity(1.0 / ui.visuals().disabled_alpha().max(0.01));
            let neutral = c.surface_raised.blend(c.surface_active);
            (
                neutral,
                c.text_subtle.gamma_multiply(0.7),
                neutral,
                c.border,
            )
        };

        let fraction = if max == min {
            0.0
        } else {
            ((value - min) / (max - min)) as f32
        };
        let x = lerp(travel, fraction.clamp(0.0, 1.0));
        let cy = rect.center().y;
        let full = CornerRadius::same(t.radius.full);
        let rail_rect = Rect::from_min_max(
            pos2(rect.left(), cy - RAIL / 2.0),
            pos2(rect.right(), cy + RAIL / 2.0),
        );
        painter.add(Shape::Rect(RectShape::filled(rail_rect, full, rail)));
        let filled = Rect::from_min_max(rail_rect.min, pos2(x, rail_rect.bottom()));
        painter.add(Shape::Rect(RectShape::filled(filled, full, fill)));

        let knob_rect = Rect::from_center_size(
            pos2(x, cy),
            vec2(KNOB, KNOB) + vec2(1.0, 1.0) * KNOB_GROW * grow,
        );
        painter.add(paint::focus_ring(knob_rect, full, c.focus_ring, focus));
        if enabled {
            painter.extend(paint::layered_shadow(
                knob_rect,
                full,
                t.elevation.get(Level::Raised),
            ));
        }
        painter.add(paint::surface(
            knob_rect,
            full,
            knob,
            Stroke::new(1.0, outline),
        ));
        response
    }
}

fn travel_f64(travel: &RangeInclusive<f32>) -> RangeInclusive<f64> {
    f64::from(*travel.start())..=f64::from(*travel.end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_to_steps_from_the_range_start() {
        let mut v = 0.0_f32;
        let s = Slider::new(&mut v, 1.0..=10.0).step(2.0);
        assert_eq!(s.snap(4.2), 5.0);
        assert_eq!(s.snap(99.0), 10.0);
        assert_eq!(s.snap(-3.0), 1.0);
    }

    #[test]
    fn integers_step_by_one() {
        let mut v = 0_u8;
        let s = Slider::new(&mut v, 0..=60);
        assert_eq!(s.snap(14.6), 15.0);
    }
}
