use egui::{Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind, epaint::RectShape};

/// Gap between a widget's edge and its focus ring, in points.
pub const FOCUS_GAP: f32 = 1.0;
/// Width of the focus ring, in points.
pub const FOCUS_WIDTH: f32 = 2.0;
/// Width of the soft halo around a focused text field, in points.
pub const FOCUS_HALO_WIDTH: f32 = 3.0;
/// Opacity of the halo, as a fraction of `focus_ring`.
pub const FOCUS_HALO_ALPHA: f32 = 0.16;

/// A focus ring around `rect`: a [`FOCUS_GAP`] gap, then a [`FOCUS_WIDTH`] ring.
///
/// `t` in `0..=1` fades the ring in (drive it with [`crate::motion::value`]).
/// Returns [`Shape::Noop`] when `t` is zero.
pub fn focus_ring(rect: Rect, radius: CornerRadius, color: Color32, t: f32) -> Shape {
    if t <= 0.0 {
        return Shape::Noop;
    }
    let grow = FOCUS_GAP;
    let r = rect.expand(grow);
    let g = |c: u8| c.saturating_add(grow as u8);
    let cr = CornerRadius {
        nw: g(radius.nw),
        ne: g(radius.ne),
        sw: g(radius.sw),
        se: g(radius.se),
    };
    let stroke = Stroke::new(FOCUS_WIDTH, color.gamma_multiply(t.clamp(0.0, 1.0)));
    Shape::Rect(RectShape::stroke(r, cr, stroke, StrokeKind::Outside))
}

/// The soft halo around a focused text field: a [`FOCUS_HALO_WIDTH`] band
/// flush against `rect`, at [`FOCUS_HALO_ALPHA`] of `color`.
///
/// Unlike [`focus_ring`] there is no gap, so the field's own border and the
/// halo read as one soft edge rather than a double outline. `t` in `0..=1`
/// fades it in; returns [`Shape::Noop`] when `t` is zero.
pub fn focus_halo(rect: Rect, radius: CornerRadius, color: Color32, t: f32) -> Shape {
    if t <= 0.0 {
        return Shape::Noop;
    }
    let alpha = FOCUS_HALO_ALPHA * t.clamp(0.0, 1.0);
    let stroke = Stroke::new(FOCUS_HALO_WIDTH, color.gamma_multiply(alpha));
    Shape::Rect(RectShape::stroke(rect, radius, stroke, StrokeKind::Outside))
}
