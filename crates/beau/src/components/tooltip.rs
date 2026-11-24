use egui::{
    Align, CornerRadius, FontId, Id, Key, Layout, Margin, Order, Response, RichText, Stroke, Ui,
};

use super::floating::{self, Floating, SurfaceStyle};
use crate::{
    behavior::{self, Placement, Side},
    motion, theme,
    tokens::Weight,
    typography,
};

/// How long the pointer rests on a widget before its tooltip shows, in seconds.
const DELAY: f64 = 0.5;
/// After a tooltip hides, others show at once for this long, so moving
/// along a toolbar doesn't wait at every button.
const SKIP_DELAY: f64 = 0.3;

/// A short label that explains a widget, shown after the pointer rests on it
/// or when it gets keyboard focus.
///
/// It is drawn inverted (ink on light, light on dark) so it reads as a label
/// rather than as floating UI, sits above the widget, and fades in from it.
/// It hides when the widget is pressed, on Escape, and as soon as a popover
/// or dialog covers the widget. The text also becomes the widget's
/// description for screen readers.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::{Button, Tooltip, icons};
/// let delete = ui.add(Button::icon_only(icons::TRASH_2, "Delete").ghost());
/// Tooltip::new("Delete task").shortcut("Del").show(ui, &delete);
/// # });
/// ```
#[must_use = "call `.show(ui, &response)`"]
pub struct Tooltip {
    text: String,
    shortcut: Option<String>,
    side: Side,
}

#[derive(Clone, Copy, Default)]
struct TipState {
    /// When the pointer arrived on the widget.
    hover_since: Option<f64>,
    /// Pressed or dismissed: stay hidden until the pointer leaves.
    suppressed: bool,
    /// The widget got focus from the keyboard.
    keyboard: bool,
}

/// State shared by all tooltips.
#[derive(Clone, Copy, Default)]
struct Shared {
    /// When a tooltip was last visible.
    last_visible: Option<f64>,
    /// The pass in which Tab was last pressed.
    last_tab_pass: Option<u64>,
}

impl Tooltip {
    /// A tooltip with this text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            shortcut: None,
            side: Side::Top,
        }
    }

    /// A keyboard shortcut, shown muted after the text.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// The preferred side of the widget ([`Side::Top`] by default). It flips
    /// when there is no room.
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Show the tooltip for `response` when it is due. Call it every pass,
    /// right after adding the widget. Returns true while it is drawn.
    pub fn show(self, ui: &Ui, response: &Response) -> bool {
        let ctx = ui.ctx();
        let t = theme(ui);
        let id = response.id.with("beau.tooltip");
        let shared_id = Id::unique("beau.tooltip.shared");
        let now = ctx.input(|i| i.time);
        let pass = ctx.cumulative_pass_nr();

        let description = match &self.shortcut {
            Some(s) => format!("{} ({s})", self.text),
            None => self.text.clone(),
        };
        ctx.accesskit_node_builder(response.id, |node| node.set_description(description));

        let mut shared: Shared = ctx.data(|d| d.get_temp(shared_id)).unwrap_or_default();
        if ctx.input(|i| i.key_pressed(Key::Tab)) {
            shared.last_tab_pass = Some(pass);
        }
        let mut s: TipState = ctx.data(|d| d.get_temp(id)).unwrap_or_default();

        // Not while something modal (a popover, a dialog) covers the widget.
        let reachable = ui.memory(|m| m.allows_interaction(response.layer_id));
        let hovered = reachable && response.contains_pointer();
        if response.is_pointer_button_down_on() || response.clicked() {
            s.suppressed = true;
        }
        if hovered {
            if s.hover_since.is_none() {
                let warm = shared
                    .last_visible
                    .is_some_and(|last| now - last < SKIP_DELAY);
                s.hover_since = Some(if warm { now - DELAY } else { now });
            }
        } else {
            s.hover_since = None;
            s.suppressed = false;
        }
        // Focus from Tab shows the tooltip; focus handed back when a popover
        // closes, or given by code, does not.
        if response.gained_focus() && shared.last_tab_pass.is_some_and(|p| pass <= p + 2) {
            s.keyboard = true;
        }
        if !response.has_focus() || !reachable {
            s.keyboard = false;
        }

        let due = s.hover_since.is_some_and(|since| now - since >= DELAY);
        if let Some(since) = s.hover_since
            && !due
        {
            ctx.request_repaint_after_secs((since + DELAY - now) as f32);
        }
        let visible = !s.suppressed && (due || s.keyboard) && ui.is_rect_visible(response.rect);
        if visible && ctx.input(|i| i.key_pressed(Key::Escape)) {
            s.suppressed = true;
            s.keyboard = false;
        }
        let visible = visible && !s.suppressed;
        if visible {
            shared.last_visible = Some(now);
        }
        ctx.data_mut(|d| {
            d.insert_temp(id, s);
            d.insert_temp(shared_id, shared);
        });

        let presence = motion::presence(ctx, id, visible, t.motion.fast);
        if !presence.should_draw() {
            return false;
        }
        let c = &t.colors;
        // Inverted: the text colour as the fill.
        let fill = c.text;
        let ink = c.surface_raised;
        let floating = Floating {
            id,
            anchor: response.rect,
            placement: Placement {
                side: self.side,
                align: behavior::Align::Center,
                gap: t.space.sm,
                margin: t.space.md,
            },
            order: Order::Tooltip,
            interactable: false,
            backdrop: false,
            drift: t.space.xs,
            role: None,
        };
        let style = SurfaceStyle {
            fill,
            stroke: Stroke::NONE,
            radius: CornerRadius::same(t.radius.sm),
            shadow: &[],
            padding: Margin::symmetric(t.space.md as i8, t.space.xs as i8),
            width: None,
            min_width: 0.0,
        };
        let label = FontId::new(12.0, typography::family(Weight::Medium));
        let hint = FontId::new(12.0, typography::family(Weight::Regular));
        let max_width = t.controls.md * 10.0;
        floating.show(ctx, presence.t, |ui| {
            floating::surface(ui, &style, |ui| {
                ui.set_max_width(max_width);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = t.space.md;
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    let text = RichText::new(&self.text)
                        .font(label)
                        .line_height(Some(16.0))
                        .color(ink);
                    ui.label(text);
                    if let Some(shortcut) = &self.shortcut {
                        let muted = ink.gamma_multiply(0.6);
                        ui.label(RichText::new(shortcut).font(hint).color(muted));
                    }
                });
            });
        });
        true
    }
}
