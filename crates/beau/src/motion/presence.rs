//! Presence: animate things in *and out*.

use std::collections::HashMap;
use std::hash::Hash;

use egui::{AsIdSalt, Context, Id, Rect, Ui, UiBuilder, vec2};

use super::{Spring, store};

/// Where an element is in its presence lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Becoming visible (`t` rising towards 1).
    Entering,
    /// Fully visible and at rest.
    Present,
    /// Leaving (`t` falling towards 0). Still drawn, but should not be interactive.
    Exiting,
    /// Gone: don't draw anything.
    Gone,
}

/// The state of a present (or leaving) element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Presence {
    /// Visibility progress, `0` = absent, `1` = fully present.
    pub t: f32,
    /// Lifecycle phase.
    pub phase: Phase,
    /// True on the first pass the element exists (useful for autofocus).
    pub appeared: bool,
}

impl Presence {
    /// Whether to draw the element at all.
    pub fn should_draw(&self) -> bool {
        self.phase != Phase::Gone
    }

    /// Whether the element should accept input (not while exiting).
    pub fn is_interactive(&self) -> bool {
        matches!(self.phase, Phase::Entering | Phase::Present)
    }

    const GONE: Self = Self {
        t: 0.0,
        phase: Phase::Gone,
        appeared: false,
    };
}

/// Track the presence of an element keyed by `id`.
///
/// Call this every pass with the element's desired visibility, and keep
/// drawing the element while [`Presence::should_draw`] is true, even after
/// `visible` turns false. That is how it animates out.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let open = false;
/// let id = ui.make_persistent_id("panel");
/// let p = beau::motion::presence(ui.ctx(), id, open, beau::theme(ui).motion.base);
/// if p.should_draw() {
///     ui.scope(|ui| {
///         ui.multiply_opacity(p.t);
///         ui.label("I fade in and out");
///     });
/// }
/// # });
/// ```
pub fn presence(ctx: &Context, id: Id, visible: bool, spring: Spring) -> Presence {
    presence_from(ctx, id, visible, spring, 0.0)
}

fn presence_from(ctx: &Context, id: Id, visible: bool, spring: Spring, initial: f32) -> Presence {
    if !visible && !store::exists(ctx, id) {
        return Presence::GONE;
    }
    let target = if visible { 1.0 } else { 0.0 };
    let s = store::animate(ctx, id, target, spring, Some(initial));
    let phase = match (visible, s.at_rest) {
        (true, false) => Phase::Entering,
        (true, true) => Phase::Present,
        (false, false) => Phase::Exiting,
        (false, true) => {
            store::remove(ctx, id);
            Phase::Gone
        }
    };
    Presence {
        t: s.value,
        phase,
        appeared: s.created && visible,
    }
}

/// One slot in a [`PresenceList`]: a live item, or a ghost of a removed one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Slot<K, T> {
    pub key: K,
    pub item: T,
    /// Height measured the last time the item was laid out.
    pub height: f32,
    /// False for ghosts (items the caller no longer provides).
    pub alive: bool,
}

/// Merge the previous slots with the current items.
///
/// Current items keep the caller's order. Each item that disappeared becomes a
/// ghost, placed right after the nearest earlier item that is still around,
/// so it animates out exactly where it was. Runs in O(n).
pub(crate) fn reconcile<K: Hash + Eq + Clone, T>(
    prev: Vec<Slot<K, T>>,
    current: Vec<(K, T)>,
) -> Vec<Slot<K, T>> {
    let mut heights: HashMap<K, f32> = HashMap::with_capacity(prev.len());
    let current_keys: std::collections::HashSet<K> =
        current.iter().map(|(k, _)| k.clone()).collect();

    // Ghosts grouped by the key they follow (None = list start).
    let mut ghosts_after: HashMap<Option<K>, Vec<Slot<K, T>>> = HashMap::new();
    let mut anchor: Option<K> = None;
    for slot in prev {
        if current_keys.contains(&slot.key) {
            anchor = Some(slot.key.clone());
            heights.insert(slot.key, slot.height);
        } else {
            ghosts_after.entry(anchor.clone()).or_default().push(Slot {
                alive: false,
                ..slot
            });
        }
    }

    let mut out = Vec::with_capacity(current.len() + ghosts_after.len());
    if let Some(g) = ghosts_after.remove(&None) {
        out.extend(g);
    }
    for (key, item) in current {
        let height = heights.get(&key).copied().unwrap_or(0.0);
        let anchor = Some(key.clone());
        out.push(Slot {
            key,
            item,
            height,
            alive: true,
        });
        if let Some(g) = ghosts_after.remove(&anchor) {
            out.extend(g);
        }
    }
    out
}

#[derive(Clone)]
struct ListState<K, T> {
    slots: Vec<Slot<K, T>>,
    /// The pass the list was last shown in.
    pass: u64,
}

impl<K, T> Default for ListState<K, T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            pass: 0,
        }
    }
}

/// A vertical list whose rows animate in when added and *out* when removed.
///
/// When an item disappears from `items`, the list keeps a clone of it (a
/// *ghost*) at its old position and keeps drawing it while its height
/// collapses and it fades out. Then the clone is dropped. That's why `T` must be
/// `Clone`: it's the price of drawing something the caller no longer has.
///
/// Rows are stacked with no item spacing; give rows their own padding.
///
/// A row's widgets keep their ids for as long as its key is in the list,
/// wherever the row moves, so focus and animations survive changes above it.
///
/// Rows that are there when the list first appears don't animate in, and
/// neither do the rows of a list that comes back after being hidden. To
/// switch a list to different contents without animating the difference
/// (another filter, another folder), give each view its own `id_salt`.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let tasks = vec![(1, "Write docs"), (2, "Ship it")];
/// beau::motion::PresenceList::new("tasks").show(
///     ui,
///     tasks.iter().copied(),
///     |(id, _)| *id,
///     |ui, (_, title), _presence| {
///         ui.label(*title);
///     },
/// );
/// # });
/// ```
pub struct PresenceList {
    id_salt: egui::IdSalt,
    spring: Option<Spring>,
}

impl PresenceList {
    /// A list identified by `id_salt` (unique within the parent `Ui`).
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: egui::IdSalt::new(id_salt),
            spring: None,
        }
    }

    /// Override the spring (defaults to the theme's `motion.base`).
    pub fn spring(mut self, spring: Spring) -> Self {
        self.spring = Some(spring);
        self
    }

    /// Show the list. `key` must be unique and stable per item.
    pub fn show<K, T>(
        self,
        ui: &mut Ui,
        items: impl IntoIterator<Item = T>,
        key: impl Fn(&T) -> K,
        mut row: impl FnMut(&mut Ui, &T, Presence),
    ) where
        K: Hash + Eq + Clone + std::fmt::Debug + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
    {
        let list_id = ui.scope_id().with_salt(self.id_salt);
        let spring = self.spring.unwrap_or(crate::theme(ui).motion.base);
        let ctx = ui.ctx().clone();

        // A list that was hidden for a while starts afresh: what it showed
        // before is stale, and animating the difference would be noise.
        let pass = ctx.cumulative_pass_nr();
        let prev: Option<ListState<K, T>> = ctx
            .data_mut(|d| d.remove_temp::<ListState<K, T>>(list_id))
            .filter(|prev| prev.pass + 1 >= pass);
        let first_show = prev.is_none();
        let current: Vec<(K, T)> = items.into_iter().map(|t| (key(&t), t)).collect();
        let mut slots = reconcile(prev.map(|p| p.slots).unwrap_or_default(), current);

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let base_opacity = ui.opacity();
            slots.retain_mut(|slot| {
                let row_id = list_id.with(&slot.key);
                // Rows already there when the list first appears don't animate in.
                let initial = if first_show { 1.0 } else { 0.0 };
                let p = presence_from(&ctx, row_id, slot.alive, spring, initial);
                if !p.should_draw() {
                    return false;
                }

                let width = ui.available_width();
                let top_left = ui.cursor().min;
                let clip_h = (p.phase != Phase::Present).then(|| slot.height * p.t.clamp(0.0, 1.0));

                let mut child = ui.new_child(
                    UiBuilder::new()
                        // An explicit id, so the ids of a row's widgets depend
                        // only on its key. Derived from the row's position,
                        // they would change when a row above is removed, and
                        // the row would lose focus and its animations.
                        .scope_id(row_id.with("row"))
                        .max_rect(Rect::from_min_size(top_left, vec2(width, f32::INFINITY))),
                );
                if let Some(h) = clip_h {
                    let clip = child
                        .clip_rect()
                        .intersect(Rect::from_min_size(top_left, vec2(width, h)));
                    child.set_clip_rect(clip);
                }
                if !p.is_interactive() {
                    child.disable();
                }
                child.set_opacity(base_opacity * p.t.clamp(0.0, 1.0));
                row(&mut child, &slot.item, p);

                let measured = child.min_rect().height();
                if slot.alive {
                    slot.height = measured;
                }
                let h = clip_h.unwrap_or(measured);
                ui.advance_cursor_after_rect(Rect::from_min_size(top_left, vec2(width, h)));
                true
            });
        });

        ctx.data_mut(|d| d.insert_temp(list_id, ListState { slots, pass }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(keys: &[(u32, bool)]) -> Vec<Slot<u32, u32>> {
        keys.iter()
            .map(|&(k, alive)| Slot {
                key: k,
                item: k,
                height: 10.0,
                alive,
            })
            .collect()
    }

    fn keys(s: &[Slot<u32, u32>]) -> Vec<(u32, bool)> {
        s.iter().map(|s| (s.key, s.alive)).collect()
    }

    #[test]
    fn removed_item_stays_in_place_as_ghost() {
        let prev = slots(&[(1, true), (2, true), (3, true)]);
        let out = reconcile(prev, vec![(1, 1), (3, 3)]);
        assert_eq!(keys(&out), vec![(1, true), (2, false), (3, true)]);
    }

    #[test]
    fn removed_first_item_ghosts_at_start() {
        let prev = slots(&[(1, true), (2, true)]);
        let out = reconcile(prev, vec![(2, 2)]);
        assert_eq!(keys(&out), vec![(1, false), (2, true)]);
    }

    #[test]
    fn consecutive_removals_keep_order() {
        let prev = slots(&[(1, true), (2, true), (3, true), (4, true)]);
        let out = reconcile(prev, vec![(1, 1), (4, 4), (5, 5)]);
        assert_eq!(
            keys(&out),
            vec![(1, true), (2, false), (3, false), (4, true), (5, true)]
        );
    }

    #[test]
    fn readded_item_revives_and_keeps_height() {
        let prev = slots(&[(1, true), (2, false)]);
        let out = reconcile(prev, vec![(1, 1), (2, 2)]);
        assert_eq!(keys(&out), vec![(1, true), (2, true)]);
        assert_eq!(out[1].height, 10.0);
    }

    #[test]
    fn new_items_start_with_zero_height() {
        let out = reconcile(Vec::<Slot<u32, u32>>::new(), vec![(7, 7)]);
        assert_eq!(out[0].height, 0.0);
        assert!(out[0].alive);
    }

    fn pass(ctx: &Context, time: f64, f: impl FnMut(&mut Ui)) {
        let input = egui::RawInput {
            time: Some(time),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, f);
        out.textures_delta.clear();
    }

    #[test]
    fn presence_lifecycle_and_gc() {
        let ctx = Context::default();
        let id = Id::unique("test.presence");
        let spring = Spring::critical(0.1);
        let mut phases = vec![];
        let mut time = 0.0;

        // Animate in.
        for _ in 0..40 {
            pass(&ctx, time, |ui| {
                phases.push(presence(ui.ctx(), id, true, spring))
            });
            time += 1.0 / 60.0;
        }
        assert!(phases[0].appeared);
        assert_eq!(phases[0].phase, Phase::Entering);
        assert_eq!(phases.last().unwrap().phase, Phase::Present);
        assert_eq!(phases.last().unwrap().t, 1.0);

        // Animate out: still drawn while exiting, then gone and garbage-collected.
        phases.clear();
        for _ in 0..40 {
            pass(&ctx, time, |ui| {
                phases.push(presence(ui.ctx(), id, false, spring))
            });
            time += 1.0 / 60.0;
        }
        assert_eq!(phases[1].phase, Phase::Exiting);
        assert!(!phases[1].is_interactive());
        assert!(phases[1].should_draw());
        assert_eq!(phases.last().unwrap().phase, Phase::Gone);
        assert_eq!(store::live_entries(&ctx), 0);
    }

    #[test]
    fn never_shown_costs_nothing() {
        let ctx = Context::default();
        pass(&ctx, 0.0, |ui| {
            let p = presence(
                ui.ctx(),
                Id::unique("test.never"),
                false,
                Spring::critical(0.1),
            );
            assert_eq!(p.phase, Phase::Gone);
        });
        assert_eq!(store::live_entries(&ctx), 0);
    }
}
