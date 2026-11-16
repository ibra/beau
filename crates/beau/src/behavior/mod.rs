//! Unstyled behaviour: keyboard focus and positioning.
//!
//! Nothing here draws or reads the theme. Components combine these with
//! [`crate::paint`] and [`crate::motion`].

mod focus;
mod placement;

pub use focus::{FocusScope, focus_on_appear, just_appeared};
pub use placement::{Align, Placed, Placement, Side, place};
