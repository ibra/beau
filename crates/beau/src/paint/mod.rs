//! Drawing primitives egui lacks or makes tedious: layered shadows, gradients,
//! crisp hairlines and focus rings.
//!
//! Everything here returns [`egui::Shape`]s (or paints into a [`egui::Painter`])
//! and takes colours and radii as arguments, so it never reaches into the theme.

mod border;
mod focus;
mod gradient;
mod shadow;

pub use border::{hairline_width, snap_rect, surface};
pub use focus::{FOCUS_GAP, FOCUS_WIDTH, focus_ring};
pub use gradient::{LinearGradient, gradient_rect};
pub use shadow::{layered_shadow, shadow_margin};

/// How a surface treats what lies behind it.
///
/// Only [`Backdrop::Opaque`] is implemented today; `Blur` is reserved so that
/// backdrop blur can be added later without API churn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Backdrop {
    /// Solid fill.
    #[default]
    Opaque,
    /// Reserved: frosted glass with the given blur radius. Currently drawn as
    /// [`Backdrop::Opaque`].
    Blur(f32),
}
