use egui::accesskit::Role;
use egui::{
    AsIdSalt, Context, EventFilter, FocusDirection, Id, Key, Modifiers, Response, Sense, Ui,
};

/// A move through a list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Previous,
    Next,
    First,
    Last,
}

/// The index `step` leads to from `current` in a list of `len` items.
///
/// With nothing current, `Next` and `First` enter at the top and `Previous`
/// and `Last` at the bottom.
fn target(len: usize, current: Option<usize>, step: Step, wrap: bool) -> Option<usize> {
    let last = len.checked_sub(1)?;
    Some(match (step, current) {
        (Step::First, _) | (Step::Next, None) => 0,
        (Step::Last, _) | (Step::Previous, None) => last,
        (Step::Next, Some(i)) if i >= last => {
            if wrap {
                0
            } else {
                last
            }
        }
        (Step::Next, Some(i)) => i + 1,
        (Step::Previous, Some(0)) => {
            if wrap {
                last
            } else {
                0
            }
        }
        (Step::Previous, Some(i)) => i - 1,
    })
}

/// How long type-ahead keeps what was typed, in seconds.
const TYPEAHEAD_TIMEOUT: f64 = 1.0;

/// The item type-ahead leads to: the first one, after the current item for
/// a fresh single character and from it otherwise, whose text starts with
/// `query`, ignoring case. Typing one character repeatedly cycles through
/// the items that start with it.
fn typeahead_target(
    texts: &[Option<String>],
    current: Option<usize>,
    query: &str,
) -> Option<usize> {
    let query = query.to_lowercase();
    let mut chars = query.chars();
    let first = chars.next()?;
    let repeated = chars.all(|c| c == first);
    let (query, skip_current) = if repeated {
        (first.to_string(), true)
    } else {
        (query, false)
    };
    let n = texts.len();
    let start = match current {
        Some(i) if skip_current => i + 1,
        Some(i) => i,
        None => 0,
    };
    (0..n).map(|k| (start + k) % n).find(|&i| {
        texts[i]
            .as_ref()
            .is_some_and(|t| t.to_lowercase().starts_with(&query))
    })
}

#[derive(Clone, Debug, Default)]
struct NavState {
    /// Key of the item that last had focus: the list's tab stop.
    current: Option<Id>,
    /// Keys of the items shown in the previous pass.
    keys: Vec<Id>,
    /// Type-ahead: what was typed, and when last.
    typed: String,
    typed_at: f64,
}

/// Keyboard navigation for a vertical list: one tab stop, arrow keys inside.
///
/// The list takes a single place in the Tab order (*roving focus*): Tab
/// enters at the item that last had focus, and Tab again leaves the list.
/// Inside, Up and Down move focus between items, Home and End jump to the
/// ends, and the focused item scrolls into view. Clicking an item focuses
/// it. Enter and Space activate the focused item the way egui activates any
/// focused widget: its [`Response::clicked`] is true.
///
/// With [`ListNav::typeahead`], typing moves focus to the next item whose
/// text (given with [`ListNav::item_text`]) starts with what was typed.
///
/// It draws nothing. [`ListNav::item`] returns whether the item has focus, so
/// the caller can paint it as selected. For screen readers the list is a
/// `ListBox` and the items are `ListBoxOption`s, unless set with
/// [`ListNav::roles`].
///
/// Create one per pass, allocate each row with [`ListNav::sense`], report it
/// with [`ListNav::item`], and end with [`ListNav::finish`]:
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::behavior::ListNav;
///
/// let mut nav = ListNav::new(ui.ctx(), ui.make_persistent_id("fruit"));
/// for name in ["Apple", "Banana", "Cherry"] {
///     let (rect, response) =
///         ui.allocate_exact_size(egui::vec2(160.0, 24.0), nav.sense(name));
///     let focused = nav.item(ui, name, &response);
///     if response.clicked() {
///         // Activated with a click, Enter or Space.
///     }
///     let color = if focused { ui.visuals().strong_text_color() } else { ui.visuals().text_color() };
///     ui.painter().text(
///         rect.left_center(),
///         egui::Align2::LEFT_CENTER,
///         name,
///         egui::FontId::default(),
///         color,
///     );
/// }
/// nav.finish(ui);
/// # });
/// ```
#[derive(Debug)]
pub struct ListNav {
    id: Id,
    wrap: bool,
    idle_keys: bool,
    typeahead: bool,
    roles: (Role, Role),
    state: NavState,
    /// Each reported item's type-ahead text, in order.
    texts: Vec<Option<String>>,
    /// Key of the tab stop, if it is still in the list.
    current: Option<Id>,
    /// `(key, widget id)` of each item reported this pass, in order.
    items: Vec<(Id, Id)>,
    focused: Option<usize>,
}

impl ListNav {
    /// Navigation for the list with this stable `id`.
    pub fn new(ctx: &Context, id: Id) -> Self {
        let id = id.with("beau.list_nav");
        let state: NavState = ctx.data(|d| d.get_temp(id)).unwrap_or_default();
        Self {
            id,
            wrap: false,
            idle_keys: false,
            typeahead: false,
            roles: (Role::ListBox, Role::ListBoxOption),
            // A tab stop that is no longer shown falls back to the first item.
            current: state.current.filter(|c| state.keys.contains(c)),
            state,
            texts: Vec::new(),
            items: Vec::new(),
            focused: None,
        }
    }

    /// Wrap around: Down on the last item goes to the first, and Up on the
    /// first to the last. Off by default.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Also take Up, Down, Home and End while *no* widget has keyboard focus,
    /// so the main list of a window can be entered without tabbing to it.
    /// Off by default.
    pub fn idle_keys(mut self, idle_keys: bool) -> Self {
        self.idle_keys = idle_keys;
        self
    }

    /// Type-ahead: typing a letter moves focus to the next item that starts
    /// with it, and typing quickly matches longer prefixes. Items take part
    /// when reported with [`ListNav::item_text`]. Off by default.
    pub fn typeahead(mut self, typeahead: bool) -> Self {
        self.typeahead = typeahead;
        self
    }

    /// The accessibility roles of the list and its items (`ListBox` and
    /// `ListBoxOption` by default), e.g. `Menu` and `MenuItem`.
    pub fn roles(mut self, list: Role, item: Role) -> Self {
        self.roles = (list, item);
        self
    }

    /// The [`Sense`] to allocate the item `key` with. Only the list's tab
    /// stop is focusable; every item senses clicks.
    ///
    /// Call it once per item, in order, before [`ListNav::item`].
    pub fn sense(&mut self, key: impl AsIdSalt) -> Sense {
        let key = self.id.with(key);
        let is_stop = match self.current {
            Some(current) => current == key,
            None => self.items.is_empty(),
        };
        if is_stop {
            Sense::click()
        } else {
            Sense::CLICK
        }
    }

    /// Report the item `key` and its `response`. Returns true if the item
    /// has keyboard focus.
    ///
    /// `key` must be unique in the list and stable across passes. Call
    /// [`Response::widget_info`] before this, not after: it would replace
    /// the item's role.
    pub fn item(&mut self, ui: &Ui, key: impl AsIdSalt, response: &Response) -> bool {
        self.report(ui, key, None, response)
    }

    /// [`ListNav::item`] with the text that type-ahead matches against.
    pub fn item_text(
        &mut self,
        ui: &Ui,
        key: impl AsIdSalt,
        text: &str,
        response: &Response,
    ) -> bool {
        self.report(ui, key, Some(text.to_owned()), response)
    }

    fn report(
        &mut self,
        ui: &Ui,
        key: impl AsIdSalt,
        text: Option<String>,
        response: &Response,
    ) -> bool {
        if ui.is_sizing_pass() || !ui.is_enabled() {
            return false;
        }
        let key = self.id.with(key);
        if response.clicked() && !response.has_focus() {
            response.request_focus();
        }
        let focused = response.has_focus();
        if focused {
            self.focused = Some(self.items.len());
            self.current = Some(key);
            // Keep egui from moving focus spatially on Up and Down.
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    EventFilter {
                        vertical_arrows: true,
                        ..Default::default()
                    },
                );
            });
            if response.gained_focus() {
                response.scroll_to_me(None);
            }
        }
        let role = self.roles.1;
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(role);
            node.set_selected(focused);
        });
        self.items.push((key, response.id));
        self.texts.push(text);
        focused
    }

    /// Handle the keys and store the state. Call it after the last item, with
    /// the `Ui` that contains the items. Returns true if a key moved focus.
    pub fn finish(self, ui: &Ui) -> bool {
        let len = self.items.len();
        let list_role = self.roles.0;
        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(list_role);
            node.set_size_of_set(len);
        });
        for (i, (_, widget)) in self.items.iter().enumerate() {
            ui.ctx().accesskit_node_builder(*widget, |node| {
                node.set_position_in_set(i);
                node.set_size_of_set(len);
            });
        }

        let idle = self.idle_keys
            && ui.is_enabled()
            && ui.memory(|m| m.focused().is_none() && m.allows_interaction(ui.layer_id()));
        let step = (self.focused.is_some() || idle)
            .then(|| {
                ui.input_mut(|i| {
                    [
                        (Key::ArrowDown, Step::Next),
                        (Key::ArrowUp, Step::Previous),
                        (Key::Home, Step::First),
                        (Key::End, Step::Last),
                    ]
                    .into_iter()
                    .find_map(|(key, step)| i.consume_key(Modifiers::NONE, key).then_some(step))
                })
            })
            .flatten();
        let mut to = step
            .and_then(|step| target(len, self.focused, step, self.wrap))
            .filter(|to| Some(*to) != self.focused);

        let mut typed = self.state.typed;
        let mut typed_at = self.state.typed_at;
        if self.typeahead && (self.focused.is_some() || idle) {
            let now = ui.input(|i| i.time);
            let text: String = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect()
            });
            if now - typed_at > TYPEAHEAD_TIMEOUT {
                typed.clear();
            }
            // A leading space is Space activating the item, not a search.
            let text = if typed.is_empty() {
                text.trim_start()
            } else {
                &text
            };
            if !text.is_empty() {
                typed.push_str(text);
                typed_at = now;
                if to.is_none() {
                    to = typeahead_target(&self.texts, self.focused, &typed)
                        .filter(|to| Some(*to) != self.focused);
                }
            }
        }

        let mut current = self.current;
        if let Some(to) = to {
            let (key, widget) = self.items[to];
            current = Some(key);
            ui.memory_mut(|m| {
                m.request_focus(widget);
                // An arrow key that arrived before the focus lock took hold
                // must not also move focus spatially.
                m.move_focus(FocusDirection::None);
            });
        }
        let keys = self.items.iter().map(|(k, _)| *k).collect();
        let state = NavState {
            current,
            keys,
            typed,
            typed_at,
        };
        ui.data_mut(|d| d.insert_temp(self.id, state));
        to.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, RawInput, vec2};

    #[test]
    fn typeahead_matches_prefixes_and_cycles() {
        let texts: Vec<Option<String>> = ["Edit", "Duplicate", "Delete", "Archive", "Deploy"]
            .into_iter()
            .map(|t| Some(t.to_owned()))
            .chain([None])
            .collect();
        // A first letter goes to the next item starting with it, wrapping.
        assert_eq!(typeahead_target(&texts, None, "d"), Some(1));
        assert_eq!(typeahead_target(&texts, Some(1), "d"), Some(2));
        assert_eq!(typeahead_target(&texts, Some(4), "d"), Some(1));
        // Repeating the letter keeps cycling.
        assert_eq!(typeahead_target(&texts, Some(1), "dd"), Some(2));
        // A longer prefix searches from the current item, ignoring case.
        assert_eq!(typeahead_target(&texts, Some(2), "DEP"), Some(4));
        assert_eq!(typeahead_target(&texts, Some(2), "del"), Some(2));
        assert_eq!(typeahead_target(&texts, Some(0), "x"), None);
        assert_eq!(typeahead_target(&texts, None, ""), None);
    }

    #[test]
    fn steps() {
        use Step::*;
        assert_eq!(target(0, None, Next, false), None);
        assert_eq!(target(3, None, Next, false), Some(0));
        assert_eq!(target(3, None, Previous, false), Some(2));
        assert_eq!(target(3, Some(1), Next, false), Some(2));
        assert_eq!(target(3, Some(2), Next, false), Some(2));
        assert_eq!(target(3, Some(2), Next, true), Some(0));
        assert_eq!(target(3, Some(0), Previous, false), Some(0));
        assert_eq!(target(3, Some(0), Previous, true), Some(2));
        assert_eq!(target(3, Some(1), First, false), Some(0));
        assert_eq!(target(3, Some(1), Last, false), Some(2));
    }

    /// Run one pass over a list of `n` rows; returns the focused row and
    /// how many rows were focusable.
    fn pass(ctx: &Context, n: usize, keys: &[Key]) -> (Option<usize>, usize) {
        let events = keys
            .iter()
            .map(|&key| Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            })
            .collect();
        let input = RawInput {
            events,
            ..Default::default()
        };
        let mut focused = None;
        let mut stops = 0;
        let mut out = ctx.run_ui(input, |ui| {
            let mut nav = ListNav::new(ui.ctx(), Id::unique("test.list")).idle_keys(true);
            for i in 0..n {
                let sense = nav.sense(i);
                stops += usize::from(sense.is_focusable());
                let (_, response) = ui.allocate_exact_size(vec2(100.0, 20.0), sense);
                nav.item(ui, i, &response);
            }
            nav.finish(ui);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            let mut nav = ListNav::new(ui.ctx(), Id::unique("test.list")).idle_keys(true);
            stops = 0;
            for i in 0..n {
                let sense = nav.sense(i);
                stops += usize::from(sense.is_focusable());
                let (_, response) = ui.allocate_exact_size(vec2(100.0, 20.0), sense);
                if nav.item(ui, i, &response) {
                    focused = Some(i);
                }
            }
            nav.finish(ui);
        });
        out.textures_delta.clear();
        (focused, stops)
    }

    #[test]
    fn arrows_move_one_roving_tab_stop() {
        let ctx = Context::default();
        assert_eq!(pass(&ctx, 4, &[]), (None, 1));
        // Nothing focused: Down enters at the top.
        assert_eq!(pass(&ctx, 4, &[Key::ArrowDown]), (Some(0), 1));
        assert_eq!(pass(&ctx, 4, &[Key::ArrowDown]), (Some(1), 1));
        assert_eq!(pass(&ctx, 4, &[Key::End]), (Some(3), 1));
        assert_eq!(pass(&ctx, 4, &[Key::ArrowDown]), (Some(3), 1));
        assert_eq!(pass(&ctx, 4, &[Key::ArrowUp]), (Some(2), 1));
        assert_eq!(pass(&ctx, 4, &[Key::Home]), (Some(0), 1));
        // The focused row is removed: the stop falls back to the first row.
        assert_eq!(pass(&ctx, 0, &[]), (None, 0));
        assert_eq!(pass(&ctx, 4, &[]).1, 1);
    }
}
