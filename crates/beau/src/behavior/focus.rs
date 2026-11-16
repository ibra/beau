use egui::{Context, Id, LayerId, Response, Ui};

/// True if `id` was not shown in the previous pass, i.e. it just appeared.
///
/// Call it once per pass for a given `id`: the call itself records that the
/// id was shown.
pub fn just_appeared(ctx: &Context, id: Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let key = id.with("beau.seen");
    let last = ctx.data_mut(|d| {
        let last = d.get_temp::<u64>(key);
        d.insert_temp(key, pass);
        last
    });
    last.is_none_or(|l| l + 1 < pass)
}

/// Autofocus: give `response` keyboard focus when it first appears. Returns
/// true on the pass where it asked for focus.
///
/// It skips the invisible sizing pass of a new [`egui::Area`], where a focus
/// request would be dropped.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let response = ui.button("Continue");
/// beau::behavior::focus_on_appear(ui, &response);
/// # });
/// ```
pub fn focus_on_appear(ui: &Ui, response: &Response) -> bool {
    if ui.is_sizing_pass() || !just_appeared(ui.ctx(), response.id) {
        return false;
    }
    response.request_focus();
    true
}

/// Keeps keyboard focus inside a floating container while it is open, and
/// hands focus back when it closes.
///
/// While active, Tab and Shift+Tab cycle through the widgets on the
/// container's layer (and layers above it, such as its own menus) and never
/// reach the page behind. When it becomes inactive, focus returns to the
/// widget that had it before, usually the button that opened the container.
///
/// Call [`FocusScope::update`] every pass the container is shown, before
/// adding its contents:
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// # let open = true;
/// let area = egui::Area::new(ui.make_persistent_id("panel"));
/// beau::behavior::FocusScope::new(ui.make_persistent_id("panel-focus"))
///     .update(ui.ctx(), area.layer(), open);
/// area.show(ui.ctx(), |ui| {
///     let first = ui.button("First");
///     beau::behavior::focus_on_appear(ui, &first);
/// });
/// # });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct FocusScope {
    id: Id,
}

#[derive(Clone, Copy, Default)]
struct ScopeState {
    active: bool,
    /// The widget focused when the scope became active.
    restore: Option<Id>,
}

impl FocusScope {
    /// A scope with a stable id, e.g. the container's own id.
    pub fn new(id: Id) -> Self {
        Self {
            id: id.with("beau.focus_scope"),
        }
    }

    /// Trap focus on `layer` while `active`, and restore the earlier focus on
    /// the pass where `active` turns false.
    ///
    /// Pass `active = false` while the container is animating out, so focus
    /// goes back as soon as it starts closing.
    pub fn update(&self, ctx: &Context, layer: LayerId, active: bool) {
        let was: ScopeState = ctx.data(|d| d.get_temp(self.id)).unwrap_or_default();
        let mut now = ScopeState { active, ..was };
        if active {
            if !was.active {
                now.restore = ctx.memory(|m| m.focused());
            }
            ctx.memory_mut(|m| m.set_modal_layer(layer));
        } else if was.active
            && let Some(id) = now.restore.take()
        {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        ctx.data_mut(|d| d.insert_temp(self.id, now));
    }
}
