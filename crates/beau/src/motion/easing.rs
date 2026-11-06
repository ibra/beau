//! Easing curves mapping `t` in `0..=1` to eased progress.
//!
//! Springs drive *time*. Easing curves shape how an animated `t` maps onto a
//! property, e.g. a dialog that scales with `out_cubic(t)`.

/// No easing.
pub fn linear(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// Fast start, gentle stop.
pub fn out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Slow start and stop.
pub fn in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Very fast start, long soft tail.
pub fn out_expo(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= 1.0 {
        1.0
    } else {
        1.0 - 2f32.powf(-10.0 * t)
    }
}

/// A CSS-style cubic Bézier timing curve through `(0,0)`, `(x1,y1)`, `(x2,y2)`, `(1,1)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    /// First control point x (`0..=1`).
    pub x1: f32,
    /// First control point y.
    pub y1: f32,
    /// Second control point x (`0..=1`).
    pub x2: f32,
    /// Second control point y.
    pub y2: f32,
}

impl CubicBezier {
    /// The standard "ease out" curve used across beau: `(0.2, 0.0, 0.0, 1.0)`.
    pub const STANDARD: Self = Self {
        x1: 0.2,
        y1: 0.0,
        x2: 0.0,
        y2: 1.0,
    };

    fn coord(a: f32, b: f32, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * t * a + 3.0 * mt * t * t * b + t * t * t
    }

    fn slope(a: f32, b: f32, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * a + 6.0 * mt * t * (b - a) + 3.0 * t * t * (1.0 - b)
    }

    /// Evaluate the curve's y for a given x (progress) in `0..=1`.
    pub fn eval(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        // Newton–Raphson on x(t) = x, falling back to bisection.
        let mut t = x;
        for _ in 0..8 {
            let err = Self::coord(self.x1, self.x2, t) - x;
            if err.abs() < 1e-5 {
                return Self::coord(self.y1, self.y2, t);
            }
            let d = Self::slope(self.x1, self.x2, t);
            if d.abs() < 1e-6 {
                break;
            }
            t = (t - err / d).clamp(0.0, 1.0);
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..24 {
            t = 0.5 * (lo + hi);
            if Self::coord(self.x1, self.x2, t) < x {
                lo = t;
            } else {
                hi = t;
            }
        }
        Self::coord(self.y1, self.y2, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        for f in [linear, out_cubic, in_out_cubic, out_expo] {
            assert!((f(0.0)).abs() < 1e-3);
            assert!((f(1.0) - 1.0).abs() < 1e-6);
        }
        let b = CubicBezier::STANDARD;
        assert!(b.eval(0.0).abs() < 1e-4);
        assert!((b.eval(1.0) - 1.0).abs() < 1e-4);
        assert!(b.eval(0.5) > 0.5, "ease-out is ahead at the midpoint");
    }
}
