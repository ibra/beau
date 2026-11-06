use egui::{CornerRadius, Margin, Rect, Shape, epaint::RectShape};

use crate::tokens::ShadowLayer;

/// Shapes for a layered soft shadow cast by a rounded rect.
///
/// Paint these *before* the surface itself. Each layer is a blurred rect, so a
/// tight contact layer plus a wide ambient layer reads as real elevation.
///
/// ```
/// use beau::{paint, tokens::Level};
/// # egui::__run_test_ui(|ui| {
/// let t = beau::theme(ui);
/// let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(200.0, 120.0));
/// let shadow = paint::layered_shadow(rect, t.radius.lg.into(), t.elevation.get(Level::Modal));
/// ui.painter().extend(shadow);
/// # });
/// ```
pub fn layered_shadow(
    rect: Rect,
    radius: CornerRadius,
    layers: &[ShadowLayer],
) -> impl Iterator<Item = Shape> + '_ {
    layers.iter().map(move |l| {
        let spread = l.spread;
        let r = rect.translate(l.offset).expand(spread);
        let grow = |c: u8| (f32::from(c) + spread).clamp(0.0, 255.0) as u8;
        let cr = CornerRadius {
            nw: grow(radius.nw),
            ne: grow(radius.ne),
            sw: grow(radius.sw),
            se: grow(radius.se),
        };
        Shape::Rect(RectShape::filled(r, cr, l.color).with_blur_width(l.blur))
    })
}

/// How far a layered shadow extends beyond its caster on each side.
///
/// Useful for sizing clip rects and areas so shadows are not cut off.
pub fn shadow_margin(layers: &[ShadowLayer]) -> Margin {
    let mut m = [0.0f32; 4]; // left, right, top, bottom
    for l in layers {
        let reach = l.blur * 0.5 + l.spread;
        m[0] = m[0].max(reach - l.offset.x);
        m[1] = m[1].max(reach + l.offset.x);
        m[2] = m[2].max(reach - l.offset.y);
        m[3] = m[3].max(reach + l.offset.y);
    }
    let c = |v: f32| v.ceil().clamp(0.0, 127.0) as i8;
    Margin {
        left: c(m[0]),
        right: c(m[1]),
        top: c(m[2]),
        bottom: c(m[3]),
    }
}
