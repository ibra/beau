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
mod switch;
mod text_input;

pub use button::{Button, Size, Variant};
pub use card::Card;
pub use checkbox::Checkbox;
pub use dialog::{Dialog, DialogResponse};
pub use icon::Icon;
pub use kbd::Kbd;
pub use switch::Switch;
pub use text_input::TextInput;

use egui::{Context, Id};

/// True if `id` was not shown in the previous pass, i.e. it just appeared.
///
/// Used for autofocus. Will move to the `behavior` layer in milestone 2.
pub(crate) fn just_appeared(ctx: &Context, id: Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let key = id.with("beau.seen");
    let last = ctx.data_mut(|d| {
        let last = d.get_temp::<u64>(key);
        d.insert_temp(key, pass);
        last
    });
    last.is_none_or(|l| l + 1 < pass)
}
