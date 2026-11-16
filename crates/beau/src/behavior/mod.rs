//! Unstyled behaviour: keyboard focus, list navigation and positioning.
//!
//! Nothing here draws or reads the theme. Components combine these with
//! [`crate::paint`] and [`crate::motion`].

mod focus;
mod list;
mod placement;

pub use focus::{FocusScope, focus_on_appear, just_appeared};
pub use list::ListNav;
pub use placement::{Align, Placed, Placement, Side, place};
