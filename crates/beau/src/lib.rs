//! # beau: beautiful UI for egui
//!
//! beau gives egui apps a polished, recognizable look ("Graphite") with very
//! little effort. Call [`install`] once, then use the [`components`].
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! beau::install(ctx);
//! # }
//! ```
//!
//! The crate is layered; each layer only uses the ones before it:
//!
//! 1. [`tokens`]: the design system as data ([`Theme`]).
//! 2. [`typography`]: bundled Inter font and the type scale.
//! 3. [`paint`]: shadows, gradients, hairlines and focus rings.
//! 4. [`motion`]: springs, easing and presence (animate in *and* out).
//! 5. [`components`]: widgets built on the above.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod components;
pub mod motion;
pub mod paint;
pub mod tokens;
pub mod typography;

pub use tokens::{Mode, Theme, set_theme, theme};

/// Install beau on a context: fonts plus the Graphite light theme.
///
/// Call once at startup, e.g. in your `eframe` app's constructor.
pub fn install(ctx: &egui::Context) {
    install_with(ctx, Theme::light());
}

/// Install beau with a specific theme.
pub fn install_with(ctx: &egui::Context, theme: Theme) {
    typography::install_fonts(ctx);
    set_theme(ctx, theme);
}
