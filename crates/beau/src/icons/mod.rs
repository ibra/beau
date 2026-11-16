//! The bundled icon set: 75 icons from [Lucide](https://lucide.dev), drawn as
//! vector strokes.
//!
//! Each constant is an [`Icon`], which is a widget:
//!
//! ```
//! # egui::__run_test_ui(|ui| {
//! use beau::{Button, icons};
//! ui.add(icons::SEARCH);
//! ui.add(Button::new("New task").icon(icons::PLUS));
//! # });
//! ```
//!
//! Constants are named after the Lucide icon (`trash-2` is [`TRASH_2`]), and
//! [`ALL`] lists every icon for galleries and pickers.
//!
//! Lucide is licensed under the ISC licence, and parts of it under the MIT
//! licence (icons derived from Feather); see `assets/icons/LICENSE`.

use crate::components::Icon;

mod set;

pub use set::*;

#[cfg(test)]
mod tests {
    use super::ALL;

    #[test]
    fn names_are_unique() {
        let mut names: Vec<&str> = ALL.iter().map(|(name, _)| *name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), ALL.len());
    }
}
