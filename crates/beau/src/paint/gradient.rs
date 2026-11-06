use egui::{
    Color32, Context, CornerRadius, Mesh, Rect, Shape, Vec2,
    epaint::{RectShape, Tessellator},
};

/// A linear gradient between colour stops.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Direction of the gradient (need not be normalised). `(0, 1)` is top to bottom.
    pub direction: Vec2,
    /// `(position, colour)` pairs, with positions in `0..=1`, sorted ascending.
    pub stops: Vec<(f32, Color32)>,
}

impl LinearGradient {
    /// Top-to-bottom gradient between two colours.
    pub fn vertical(top: Color32, bottom: Color32) -> Self {
        Self {
            direction: Vec2::DOWN,
            stops: vec![(0.0, top), (1.0, bottom)],
        }
    }

    /// Colour at position `t` in `0..=1`.
    pub fn sample(&self, t: f32) -> Color32 {
        let Some(&(first_t, first)) = self.stops.first() else {
            return Color32::TRANSPARENT;
        };
        if t <= first_t {
            return first;
        }
        for pair in self.stops.windows(2) {
            let (t0, c0) = pair[0];
            let (t1, c1) = pair[1];
            if t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
                return c0.lerp_to_gamma(c1, f);
            }
        }
        self.stops.last().map_or(first, |s| s.1)
    }
}

/// A rounded rect filled with a linear gradient, anti-aliased like any egui shape.
///
/// The rect is tessellated by egui as a plain white fill (so edges get egui's
/// feathering), then each vertex is recoloured by projecting it onto the
/// gradient direction, with its coverage alpha preserved.
pub fn gradient_rect(
    ctx: &Context,
    rect: Rect,
    radius: CornerRadius,
    gradient: &LinearGradient,
) -> Shape {
    let ppp = ctx.pixels_per_point();
    let options = ctx.tessellation_options(|o| *o);
    let mut tessellator = Tessellator::new(ppp, options, [1, 1], vec![]);
    let mut mesh = Mesh::default();
    tessellator.tessellate_rect(&RectShape::filled(rect, radius, Color32::WHITE), &mut mesh);

    let dir = if gradient.direction == Vec2::ZERO {
        Vec2::DOWN
    } else {
        gradient.direction.normalized()
    };
    // Project the rect's corners to find the gradient's extent along `dir`.
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ];
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for c in corners {
        let p = c.to_vec2().dot(dir);
        lo = lo.min(p);
        hi = hi.max(p);
    }
    let span = (hi - lo).max(f32::EPSILON);

    for v in &mut mesh.vertices {
        let coverage = f32::from(v.color.a()) / 255.0;
        let t = ((v.pos.to_vec2().dot(dir) - lo) / span).clamp(0.0, 1.0);
        v.color = gradient.sample(t).gamma_multiply(coverage);
    }
    Shape::mesh(mesh)
}
