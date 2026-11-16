//! Components built on beau's foundations.
//!
//! Components are thin: they pick tokens, ask [`crate::motion`] for animated
//! values and draw with [`crate::paint`].

mod button;
mod card;
mod checkbox;
mod dialog;
mod icon;
mod kbd;
mod slider;
mod switch;
mod text_input;

pub use button::{Button, Size, Variant};
pub use card::Card;
pub use checkbox::Checkbox;
pub use dialog::{Dialog, DialogResponse};
pub use icon::Icon;
pub use kbd::Kbd;
pub use slider::Slider;
pub use switch::Switch;
pub use text_input::TextInput;
