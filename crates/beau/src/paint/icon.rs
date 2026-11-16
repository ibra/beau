use egui::{Color32, Pos2, Rect, Shape, Stroke, pos2};

/// Side of the square grid icon paths are drawn on (Lucide's 24 units).
pub(crate) const ICON_GRID: f32 = 24.0;

/// Icon stroke width in points. It stays the same at every icon size, so
/// icons keep the weight of the text next to them.
pub const ICON_STROKE: f32 = 1.5;

/// One command of an icon outline, in [`ICON_GRID`] units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PathCmd {
    /// Start a new subpath.
    Move(f32, f32),
    /// Straight line to a point.
    Line(f32, f32),
    /// Cubic Bézier: two control points, then the end point.
    Cubic(f32, f32, f32, f32, f32, f32),
    /// Close the subpath.
    Close,
}

/// Stroke an icon path scaled to fill `rect`, with round-looking ends.
///
/// Curves are flattened for the given `pixels_per_point`, so icons stay
/// smooth at any size and scale factor.
pub(crate) fn stroke_path(
    path: &[PathCmd],
    rect: Rect,
    width: f32,
    color: Color32,
    pixels_per_point: f32,
) -> Vec<Shape> {
    let scale = rect.width() / ICON_GRID;
    let at = |x: f32, y: f32| pos2(rect.left() + x * scale, rect.top() + y * scale);
    let stroke = Stroke::new(width, color);

    let mut shapes = Vec::new();
    let mut points: Vec<Pos2> = Vec::new();
    let mut flush = |points: &mut Vec<Pos2>, closed: bool| {
        if let Some(shape) = subpath(std::mem::take(points), closed, stroke) {
            shapes.push(shape);
        }
    };
    for cmd in path {
        match *cmd {
            PathCmd::Move(x, y) => {
                flush(&mut points, false);
                points.push(at(x, y));
            }
            PathCmd::Line(x, y) => points.push(at(x, y)),
            PathCmd::Cubic(x1, y1, x2, y2, x, y) => {
                let Some(&p0) = points.last() else { continue };
                let (p1, p2, p3) = (at(x1, y1), at(x2, y2), at(x, y));
                // About one segment per 1.5 physical pixels of curve.
                let length = p0.distance(p1) + p1.distance(p2) + p2.distance(p3);
                let n = ((length * pixels_per_point / 1.5).ceil() as usize).clamp(2, 32);
                for i in 1..=n {
                    let t = i as f32 / n as f32;
                    let u = 1.0 - t;
                    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    points.push(pos2(
                        a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                        a * p0.y + b * p1.y + c * p2.y + d * p3.y,
                    ));
                }
            }
            PathCmd::Close => flush(&mut points, true),
        }
    }
    flush(&mut points, false);
    shapes
}

/// One flattened subpath as a shape. A zero-length subpath is a dot.
fn subpath(mut points: Vec<Pos2>, closed: bool, stroke: Stroke) -> Option<Shape> {
    const EPS: f32 = 1e-3;
    points.dedup_by(|a, b| a.distance(*b) < EPS);
    if closed && points.len() > 1 && points[0].distance(points[points.len() - 1]) < EPS {
        points.pop();
    }
    match points.len() {
        0 => None,
        // Slightly wider than the stroke, so dots hold their own next to lines.
        1 => Some(Shape::circle_filled(
            points[0],
            stroke.width * 0.6,
            stroke.color,
        )),
        _ if closed => Some(Shape::closed_line(points, stroke)),
        n => {
            // egui ends lines flush (butt caps). Extend both ends by half the
            // stroke so they reach as far as the source's round caps.
            let half = stroke.width / 2.0;
            let start = (points[0] - points[1]).normalized() * half;
            let end = (points[n - 1] - points[n - 2]).normalized() * half;
            points[0] += start;
            points[n - 1] += end;
            Some(Shape::line(points, stroke))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    fn rect() -> Rect {
        Rect::from_min_size(pos2(10.0, 10.0), vec2(24.0, 24.0))
    }

    #[test]
    fn zero_length_subpath_is_a_dot() {
        let path = [PathCmd::Move(12.0, 12.0), PathCmd::Line(12.0, 12.0)];
        let shapes = stroke_path(&path, rect(), 2.0, Color32::BLACK, 1.0);
        assert!(matches!(shapes.as_slice(), [Shape::Circle(c)] if c.center == pos2(22.0, 22.0)));
    }

    #[test]
    fn open_lines_get_extended_caps() {
        let path = [PathCmd::Move(4.0, 12.0), PathCmd::Line(20.0, 12.0)];
        let shapes = stroke_path(&path, rect(), 2.0, Color32::BLACK, 1.0);
        let [Shape::Path(p)] = shapes.as_slice() else {
            panic!("expected one path");
        };
        assert!(!p.closed);
        assert_eq!(p.points, vec![pos2(13.0, 22.0), pos2(31.0, 22.0)]);
    }

    #[test]
    fn closed_subpaths_stay_closed_and_scale() {
        let path = [
            PathCmd::Move(0.0, 0.0),
            PathCmd::Line(24.0, 0.0),
            PathCmd::Line(24.0, 24.0),
            PathCmd::Line(0.0, 0.0),
            PathCmd::Close,
        ];
        let half = Rect::from_min_size(pos2(0.0, 0.0), vec2(12.0, 12.0));
        let shapes = stroke_path(&path, half, 1.5, Color32::BLACK, 2.0);
        let [Shape::Path(p)] = shapes.as_slice() else {
            panic!("expected one path");
        };
        assert!(p.closed);
        assert_eq!(
            p.points,
            vec![pos2(0.0, 0.0), pos2(12.0, 0.0), pos2(12.0, 12.0)]
        );
    }
}
