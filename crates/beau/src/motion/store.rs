//! Animation state, stored in egui's temp memory.

use egui::{Context, Id, IdMap};

use super::Spring;

/// Largest time step applied in one go. After the app has been idle for a
/// while, the first frame advances at most this much, so nothing visibly jumps.
pub(crate) const MAX_DT: f32 = 1.0 / 30.0;

/// Entries not touched for this many passes are dropped.
const GC_AFTER_PASSES: u64 = 2;

#[derive(Clone, Debug)]
struct Entry {
    value: f32,
    velocity: f32,
    last_time: f64,
    last_pass: u64,
}

#[derive(Clone, Debug, Default)]
struct Store {
    entries: IdMap<Entry>,
    gc_pass: u64,
}

impl Store {
    fn gc(&mut self, pass: u64) {
        if self.gc_pass != pass {
            self.gc_pass = pass;
            self.entries
                .retain(|_, e| e.last_pass + GC_AFTER_PASSES >= pass);
        }
    }
}

fn store_id() -> Id {
    Id::unique("beau.motion.store")
}

/// Result of advancing one animated value.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    pub value: f32,
    pub at_rest: bool,
    /// The entry was created during this call.
    pub created: bool,
}

/// Advance the animation `id` towards `target` and return its current value.
///
/// `initial` is the value a *new* entry starts at (`None` = start at `target`).
/// Requests a repaint while the value is still moving.
pub(crate) fn animate(
    ctx: &Context,
    id: Id,
    target: f32,
    spring: Spring,
    initial: Option<f32>,
) -> Sample {
    let reduced = crate::theme(ctx).motion.reduced;
    let now = ctx.input(|i| i.time);
    let pass = ctx.cumulative_pass_nr();

    let sample = ctx.data_mut(|d| {
        let store = d.get_temp_mut_or_default::<Store>(store_id());
        store.gc(pass);
        let mut created = false;
        let e = store.entries.entry(id).or_insert_with(|| {
            created = true;
            Entry {
                value: initial.unwrap_or(target),
                velocity: 0.0,
                last_time: now,
                last_pass: pass,
            }
        });
        let dt = ((now - e.last_time) as f32).clamp(0.0, MAX_DT);
        e.last_time = now;
        e.last_pass = pass;
        let at_rest = if reduced {
            e.value = target;
            e.velocity = 0.0;
            true
        } else {
            spring.step(&mut e.value, &mut e.velocity, target, dt)
        };
        Sample {
            value: e.value,
            at_rest,
            created,
        }
    });

    if !sample.at_rest {
        ctx.request_repaint();
    }
    sample
}

/// Whether an animation entry exists for `id`.
pub(crate) fn exists(ctx: &Context, id: Id) -> bool {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Store>(store_id())
            .entries
            .contains_key(&id)
    })
}

/// Drop the animation entry for `id`.
pub(crate) fn remove(ctx: &Context, id: Id) {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Store>(store_id())
            .entries
            .remove(&id);
    });
}

/// Number of live animation entries (for tests and debugging).
pub fn live_entries(ctx: &Context) -> usize {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Store>(store_id()).entries.len())
}
