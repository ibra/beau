/// A damped spring, described the way designers think about it.
///
/// * `response`: roughly how long (seconds) one oscillation would take, which in
///   practice is how quick the spring feels.
/// * `damping_ratio`: `1.0` is critically damped (fastest without overshoot),
///   `< 1.0` overshoots, `> 1.0` is sluggish.
///
/// [`Spring::step`] solves the spring *analytically*, so the result is exact for
/// any `dt`. Animations look identical at 30, 60 or 144 fps and never drift.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Perceived duration in seconds.
    pub response: f32,
    /// Damping ratio (`1.0` = critically damped).
    pub damping_ratio: f32,
}

/// Displacement below which a spring is considered at rest.
const REST_DISPLACEMENT: f32 = 5e-4;
/// Velocity below which a spring is considered at rest.
const REST_VELOCITY: f32 = 5e-3;

impl Spring {
    /// A spring with the given response and damping ratio.
    pub const fn new(response: f32, damping_ratio: f32) -> Self {
        Self {
            response,
            damping_ratio,
        }
    }

    /// A critically damped spring (no overshoot), beau's default.
    pub const fn critical(response: f32) -> Self {
        Self::new(response, 1.0)
    }

    /// Advance `value`/`velocity` towards `target` by `dt` seconds.
    ///
    /// Returns `true` once the spring has come to rest; the value is then
    /// snapped exactly onto `target`.
    pub fn step(&self, value: &mut f32, velocity: &mut f32, target: f32, dt: f32) -> bool {
        if self.response <= 0.0 {
            *value = target;
            *velocity = 0.0;
            return true;
        }
        let (d, v) = self.solve(*value - target, *velocity, dt.max(0.0));
        *value = target + d;
        *velocity = v;
        let at_rest = d.abs() < REST_DISPLACEMENT && v.abs() < REST_VELOCITY;
        if at_rest {
            *value = target;
            *velocity = 0.0;
        }
        at_rest
    }

    /// Closed-form damped harmonic oscillator.
    ///
    /// Given displacement `d0` from the target and velocity `v0`, return the
    /// displacement and velocity after `t` seconds.
    fn solve(&self, d0: f32, v0: f32, t: f32) -> (f32, f32) {
        let w0 = std::f32::consts::TAU / self.response;
        let zeta = self.damping_ratio.max(0.0);

        if (zeta - 1.0).abs() < 1e-4 {
            // Critically damped: d = e^{-w t} (A + B t).
            let b = v0 + w0 * d0;
            let e = (-w0 * t).exp();
            let d = e * (d0 + b * t);
            let v = e * (v0 - w0 * b * t);
            (d, v)
        } else if zeta < 1.0 {
            // Under-damped.
            let wd = w0 * (1.0 - zeta * zeta).sqrt();
            let a = d0;
            let b = (v0 + zeta * w0 * d0) / wd;
            let e = (-zeta * w0 * t).exp();
            let (s, c) = (wd * t).sin_cos();
            let d = e * (a * c + b * s);
            let v = e * (-zeta * w0 * (a * c + b * s) + (-a * wd * s + b * wd * c));
            (d, v)
        } else {
            // Over-damped.
            let root = (zeta * zeta - 1.0).sqrt();
            let r1 = -w0 * (zeta - root);
            let r2 = -w0 * (zeta + root);
            let c2 = (v0 - r1 * d0) / (r2 - r1);
            let c1 = d0 - c2;
            let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(spring: Spring, fps: f32, seconds: f32) -> f32 {
        let (mut x, mut v) = (0.0, 0.0);
        let dt = 1.0 / fps;
        let steps = (seconds * fps).round() as usize;
        for _ in 0..steps {
            spring.step(&mut x, &mut v, 1.0, dt);
        }
        x
    }

    #[test]
    fn frame_rate_independent() {
        for spring in [
            Spring::critical(0.18),
            Spring::new(0.3, 0.6),
            Spring::new(0.3, 1.6),
        ] {
            let a = run(spring, 30.0, 0.1);
            let b = run(spring, 60.0, 0.1);
            let c = run(spring, 120.0, 0.1);
            assert!((a - b).abs() < 1e-4, "{spring:?}: {a} vs {b}");
            assert!((b - c).abs() < 1e-4, "{spring:?}: {b} vs {c}");
        }
    }

    #[test]
    fn settles_exactly_on_target() {
        let spring = Spring::critical(0.18);
        let (mut x, mut v) = (0.0, 0.0);
        let mut rested = false;
        for _ in 0..120 {
            rested = spring.step(&mut x, &mut v, 1.0, 1.0 / 60.0);
            if rested {
                break;
            }
        }
        assert!(rested);
        assert_eq!(x, 1.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn critical_never_overshoots() {
        let spring = Spring::critical(0.2);
        let (mut x, mut v) = (0.0, 0.0);
        for _ in 0..200 {
            spring.step(&mut x, &mut v, 1.0, 1.0 / 144.0);
            assert!(x <= 1.0 + 1e-6);
        }
    }

    #[test]
    fn underdamped_overshoots() {
        let spring = Spring::new(0.3, 0.4);
        let (mut x, mut v) = (0.0, 0.0);
        let mut max = 0.0f32;
        for _ in 0..200 {
            spring.step(&mut x, &mut v, 1.0, 1.0 / 60.0);
            max = max.max(x);
        }
        assert!(max > 1.05);
    }
}
