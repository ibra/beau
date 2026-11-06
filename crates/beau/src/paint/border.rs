use egui::{Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind, epaint::RectShape};

/// The width, in points, of exactly one physical pixel.
pub fn hairline_width(pixels_per_point: f32) -> f32 {
    1.0 / pixels_per_point.max(f32::EPSILON)
}

/// Snap a rect's edges to the physical pixel grid, so 1px strokes stay crisp.
pub fn snap_rect(rect: Rect, pixels_per_point: f32) -> Rect {
    let s = |v: f32| (v * pixels_per_point).round() / pixels_per_point;
    Rect::from_min_max(
        egui::pos2(s(rect.min.x), s(rect.min.y)),
        egui::pos2(s(rect.max.x), s(rect.max.y)),
    )
}

/// A filled rounded rect with a crisp *inside* hairline.
///
/// Inside strokes never change a surface's footprint, so a bordered and an
/// unbordered surface line up exactly.
pub fn surface(rect: Rect, radius: CornerRadius, fill: Color32, border: Stroke) -> Shape {
    Shape::Rect(RectShape::new(
        rect,
        radius,
        fill,
        border,
        StrokeKind::Inside,
    ))
}
