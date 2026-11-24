use egui::{
    Context, CornerRadius, Id, Key, LayerId, Margin, Modifiers, Order, Rect, Response, Ui,
    UiBuilder, accesskit::Role,
};

use super::floating::{self, Floating, SurfaceStyle};
use crate::{
    behavior::{Align, FocusScope, Placement, Side},
    motion::{self, Presence},
    theme,
    tokens::Level,
};

/// Floating content anchored to a trigger, opened by clicking it.
///
/// * Clicking the trigger (or pressing Enter or Space on it) toggles it.
/// * It sits below the trigger, flipping above it when there is no room,
///   and fades in from it.
/// * Escape or a click outside closes it. Content closes it with
///   [`Ui::close`], e.g. after a choice is made.
/// * While open, it is modal: Tab stays inside it and the page behind
///   doesn't react to the pointer, so the first click outside only closes
///   it. Closing it returns keyboard focus to where it was.
///
/// Popovers nest: a popover opened from inside another keeps the outer one
/// open, and Escape closes the innermost first.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::{Button, Popover};
/// let trigger = ui.add(Button::new("Share"));
/// Popover::new(&trigger).show(ui.ctx(), |ui| {
///     ui.label("Anyone with the link can view.");
///     if ui.add(Button::new("Copy link")).clicked() {
///         ui.close();
///     }
/// });
/// # });
/// ```
#[must_use = "call `.show(ctx, |ui| ...)`"]
pub struct Popover<'o> {
    id: Id,
    anchor: Rect,
    trigger: Option<Id>,
    toggle: bool,
    open: Option<&'o mut bool>,
    side: Side,
    align: Align,
    width: Option<f32>,
    min_width: f32,
    padding: Option<f32>,
    role: Role,
}

/// What [`Popover::show`] returns while the popover is drawn.
pub struct PopoverResponse<R> {
    /// What the content closure returned.
    pub inner: R,
    /// Response for the popover surface.
    pub response: Response,
    /// Presence state (`t`, phase, and whether it just appeared).
    pub presence: Presence,
}

impl<'o> Popover<'o> {
    /// A popover for `trigger`, which opens and closes it.
    ///
    /// Its open state is kept in egui's memory unless bound with
    /// [`Popover::open`].
    pub fn new(trigger: &Response) -> Self {
        let mut popover = Self::anchored(trigger.id.with("beau.popover"), trigger.rect);
        popover.trigger = Some(trigger.id);
        popover.toggle = trigger.clicked();
        popover
    }

    /// A popover next to `anchor` with no trigger, e.g. at the pointer.
    /// Open it with [`Popover::open`] or [`Popover::set_open`].
    pub fn anchored(id: Id, anchor: Rect) -> Self {
        Self {
            id,
            anchor,
            trigger: None,
            toggle: false,
            open: None,
            side: Side::Bottom,
            align: Align::Start,
            width: None,
            min_width: 0.0,
            padding: None,
            role: Role::Dialog,
        }
    }

    /// Bind the open state. Clicking the trigger toggles it; dismissing the
    /// popover sets it to `false`.
    pub fn open(mut self, open: &'o mut bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Open or close the popover with this `id` (for popovers whose state is
    /// kept in memory). A trigger's popover id is `trigger.id.with("beau.popover")`.
    pub fn set_open(ctx: &Context, id: Id, open: bool) {
        ctx.data_mut(|d| d.insert_temp(id.with("beau.open"), open));
    }

    /// The preferred side of the anchor ([`Side::Bottom`] by default).
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Alignment along that side ([`Align::Start`] by default).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Exact width in points, including padding. By default the popover
    /// fits its content.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Minimum width in points, including padding.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Space between the edge and the content (`space.lg` by default).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// The popover's id, e.g. for [`Popover::set_open`].
    pub fn id(&self) -> Id {
        self.id
    }

    /// Show the popover when it is open. Returns `None` once it is fully
    /// closed.
    pub fn show<R>(
        self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<PopoverResponse<R>> {
        let t = theme(ctx);
        let c = &t.colors;
        let state_id = self.id.with("beau.open");
        let layer = LayerId::new(Order::Foreground, self.id);

        let was_open = match &self.open {
            Some(open) => **open,
            None => ctx.data(|d| d.get_temp(state_id)).unwrap_or(false),
        };
        let mut open = was_open != self.toggle;
        if was_open && open {
            // A press outside the content closes it, unless it landed on
            // something above, such as a popover opened from this one.
            let content = Floating::last_rect(ctx, self.id);
            let pressed_outside = ctx
                .input(|i| i.pointer.any_pressed().then(|| i.pointer.interact_pos()))
                .flatten()
                .is_some_and(|pos| {
                    ctx.layer_id_at(pos) == Some(layer) && !content.is_some_and(|r| r.contains(pos))
                });
            let on_top = ctx.memory(|m| m.top_modal_layer()) == Some(layer);
            let escape = on_top && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
            if pressed_outside || escape {
                open = false;
            }
        }
        if let Some(trigger) = self.trigger {
            ctx.accesskit_node_builder(trigger, |node| node.set_expanded(open));
        }

        let presence = motion::presence(ctx, self.id, open, t.motion.base);
        let interactive = presence.is_interactive();
        // Tab stays inside while open; closing hands focus back.
        FocusScope::new(self.id).update(ctx, layer, interactive);
        let mut result = None;
        if presence.should_draw() {
            let floating = Floating {
                id: self.id,
                anchor: self.anchor,
                placement: Placement {
                    side: self.side,
                    align: self.align,
                    gap: t.space.xs,
                    margin: t.space.md,
                },
                order: layer.order,
                interactable: interactive,
                backdrop: true,
                drift: t.space.xs,
                role: Some(self.role),
            };
            let padding = self.padding.unwrap_or(t.space.lg);
            let style = SurfaceStyle {
                fill: c.surface_overlay,
                stroke: t.divider(ctx),
                radius: CornerRadius::same(t.radius.md),
                shadow: t.elevation.get(Level::Overlay),
                padding: Margin::same(padding.round() as i8),
                width: self.width,
                min_width: self.min_width,
            };
            let shown = floating.show(ctx, presence.t, |ui| {
                ui.scope_builder(UiBuilder::new().closable(), |ui| {
                    let inner = floating::surface(ui, &style, add_contents);
                    (inner, ui.should_close())
                })
                .inner
            });
            let (inner, close) = shown.inner;
            if close {
                open = false;
            }
            result = Some(PopoverResponse {
                inner,
                response: shown.response,
                presence,
            });
        }

        match self.open {
            Some(bound) => *bound = open,
            None => ctx.data_mut(|d| {
                d.insert_temp(state_id, open);
            }),
        }
        result
    }
}
