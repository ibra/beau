//! # beau: beautiful UI for egui
//!
//! beau gives egui apps a polished, recognizable look with very
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
//! 3. [`paint`]: shadows, gradients, hairlines, focus rings and icon strokes.
//! 4. [`motion`]: springs, easing and presence (animate in *and* out).
//! 5. [`components`]: widgets built on the above, [`icons`]: the bundled
//!    icon set, and [`stock`]: beau's look for egui's own widgets.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod components;
pub mod icons;
pub mod motion;
pub mod paint;
pub mod stock;
pub mod tokens;
pub mod typography;

pub use components::{Button, Card, Dialog, Icon, Size, TextInput, Variant};
pub use tokens::{Mode, Theme, mode, set_preference, set_theme, theme};

/// Install beau on a context: fonts, the light and dark themes, and beau's
/// look for stock egui widgets ([`stock::StockStyle`]).
///
/// The active theme follows the OS light/dark setting; override it with
/// [`set_preference`]. Call once at startup, e.g. in your `eframe` app's
/// constructor.
pub fn install(ctx: &egui::Context) {
    typography::install_fonts(ctx);
    set_theme(ctx, Theme::light());
    set_theme(ctx, Theme::dark());
    stock::StockStyle::register(ctx);
}

/// Install beau with the same accent colour in light and dark mode.
///
/// ```no_run
/// # fn setup(ctx: &egui::Context) {
/// beau::install_with_accent(ctx, egui::Color32::from_rgb(0x3B, 0x82, 0xF6));
/// # }
/// ```
pub fn install_with_accent(ctx: &egui::Context, accent: egui::Color32) {
    typography::install_fonts(ctx);
    set_theme(ctx, Theme::light().with_accent(accent));
    set_theme(ctx, Theme::dark().with_accent(accent));
    stock::StockStyle::register(ctx);
}
