//! Motion: springs, easing curves and presence.
//!
//! All animation state lives in egui's temp memory, keyed by [`egui::Id`].
//! A repaint is requested only while something is still moving, so an idle app
//! does no work.
//!
//! * [`value`]: animate any scalar towards a target (hover, press, focus).
//! * [`presence()`]: animate an element in *and out*.
//! * [`PresenceList`]: keyed rows that animate out after the caller removes them.

pub mod easing;
mod presence;
mod spring;
mod store;

use egui::{Context, Id};

pub use presence::{Phase, Presence, PresenceList, presence};
pub use spring::Spring;
pub use store::live_entries;

/// Animate a scalar towards `target` with a spring and return its current value.
///
/// The first time an `id` is seen, the value starts at `target` (no
/// animation). Later target changes animate smoothly, even mid-flight.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let response = ui.label("hover me");
/// let t = beau::motion::value(
///     ui.ctx(),
///     response.id.with("hover"),
///     if response.hovered() { 1.0 } else { 0.0 },
///     beau::theme(ui).motion.fast,
/// );
/// # });
/// ```
pub fn value(ctx: &Context, id: Id, target: f32, spring: Spring) -> f32 {
    store::animate(ctx, id, target, spring, None).value
}

/// Like [`value`], but a new `id` starts at `initial` and animates to `target`.
pub fn value_from(ctx: &Context, id: Id, initial: f32, target: f32, spring: Spring) -> f32 {
    store::animate(ctx, id, target, spring, Some(initial)).value
}

/// [`value`] for booleans: animates between `0.0` and `1.0`.
pub fn toggle(ctx: &Context, id: Id, on: bool, spring: Spring) -> f32 {
    value(ctx, id, if on { 1.0 } else { 0.0 }, spring)
}
