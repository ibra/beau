use egui::{
    Align, Area, Context, CornerRadius, Frame, Id, Key, Layout, Margin, Modifiers, Order, Rect,
    Response, Sense, Shape, Ui, UiBuilder, emath::TSTransform, pos2, vec2,
};

use super::Kbd;
use crate::{
    motion::{self, Presence},
    paint, theme,
    tokens::{Level, TextRole},
};

/// A modal dialog that animates in *and* out.
///
/// * A backdrop fades in behind it. Clicking the backdrop closes it.
/// * `Escape` closes it.
/// * While closing, it keeps drawing (fading and settling back) but no longer
///   takes input. Then it's gone.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::{Button, Dialog};
/// let mut open = true;
/// Dialog::new("confirm-delete")
///     .title("Delete task?")
///     .description("This can't be undone.")
///     .open(&mut open)
///     .show(ui.ctx(), |ui| {
///         Dialog::footer(ui, |ui| {
///             ui.add(Button::new("Delete").danger());
///             ui.add(Button::new("Cancel").ghost());
///         });
///     });
/// # });
/// ```
#[must_use = "call `.show(ctx, |ui| ...)`"]
pub struct Dialog<'o> {
    id: Id,
    title: Option<String>,
    description: Option<String>,
    open: Option<&'o mut bool>,
    width: f32,
}

/// What [`Dialog::show`] returns while the dialog is drawn.
pub struct DialogResponse<R> {
    /// What the content closure returned.
    pub inner: R,
    /// Response for the dialog surface.
    pub response: Response,
    /// Presence state (`t`, phase, and whether it just appeared).
    pub presence: Presence,
}

impl<'o> Dialog<'o> {
    /// A dialog with a globally unique id.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: Id::unique(("beau.dialog", egui::IdSalt::new(id).value())),
            title: None,
            description: None,
            open: None,
            width: 440.0,
        }
    }

    /// Title shown at the top.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Muted supporting text below the title.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Bind the open state. Set to `false` when dismissed (Escape, backdrop).
    pub fn open(mut self, open: &'o mut bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Dialog width in points (default 440).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// A right-aligned row of actions, laid out right-to-left: add the primary
    /// action first.
    pub fn footer<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let t = theme(ui);
        ui.add_space(t.space.xl);
        // An explicit row height: centring inside the dialog's unbounded
        // height would otherwise place the row at infinity.
        let size = vec2(ui.available_width(), t.controls.md);
        ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = t.space.md;
            add_contents(ui)
        })
        .inner
    }

    /// Show the dialog. Returns `None` once it is fully closed.
    pub fn show<R>(
        self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<DialogResponse<R>> {
        let t = theme(ctx);
        let c = &t.colors;
        let is_open = self.open.as_deref().copied().unwrap_or(true);
        let presence = motion::presence(ctx, self.id, is_open, t.motion.base);
        if !presence.should_draw() {
            return None;
        }
        let k = presence.t.clamp(0.0, 1.0);
        let interactive = presence.is_interactive();

        let screen = ctx.content_rect();
        let size_id = self.id.with("beau.size");
        let last_h: f32 = ctx.data(|d| d.get_temp(size_id)).unwrap_or(260.0);
        let w = self.width.min(screen.width() - 2.0 * t.space.xl);
        let top = screen.top() + ((screen.height() - last_h) * 0.42).max(t.space.xxl);
        let rect = Rect::from_min_size(pos2(screen.center().x - w / 2.0, top), vec2(w, last_h));

        let area = Area::new(self.id)
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .interactable(interactive)
            .role(egui::accesskit::Role::Dialog);
        if interactive {
            let layer = area.layer();
            ctx.memory_mut(|m| m.set_modal_layer(layer));
        }

        let mut close = false;
        let out = area.show(ctx, |ui| {
            // Backdrop.
            let mut backdrop =
                ui.new_child(UiBuilder::new().sense(Sense::click()).max_rect(screen));
            backdrop.set_min_size(screen.size());
            if backdrop.response().clicked() {
                close = true;
            }
            ui.painter()
                .rect_filled(screen, 0.0, c.backdrop.gamma_multiply(k));

            // Settle in: scale up from 97% and drift up 6px.
            let scale = 0.97 + 0.03 * k;
            let pivot = rect.center().to_vec2();
            let transform = TSTransform::from_translation(vec2(0.0, 6.0 * (1.0 - k)))
                * TSTransform::from_translation(pivot)
                * TSTransform::from_scaling(scale)
                * TSTransform::from_translation(-pivot);

            ui.with_visual_transform(transform, |ui| {
                ui.multiply_opacity(k);
                let mut panel = ui.new_child(
                    UiBuilder::new()
                        .max_rect(Rect::from_min_size(rect.min, vec2(w, f32::INFINITY)))
                        .sense(Sense::click_and_drag())
                        .layout(Layout::top_down(Align::Min)),
                );
                let where_bg = panel.painter().add(Shape::Noop);
                let inner = Frame::NONE
                    .inner_margin(Margin::same(t.space.xxl as i8 - 4))
                    .show(&mut panel, |ui| {
                        ui.set_width(w - 2.0 * (t.space.xxl - 4.0));
                        if self.title.is_some() || self.description.is_some() {
                            self.header(ui);
                        }
                        add_contents(ui)
                    });
                let body = Rect::from_min_size(rect.min, vec2(w, inner.response.rect.height()));
                let radius = CornerRadius::same(t.radius.lg);
                let mut shapes: Vec<Shape> =
                    paint::layered_shadow(body, radius, t.elevation.get(Level::Modal)).collect();
                shapes.push(paint::surface(
                    body,
                    radius,
                    c.surface_overlay,
                    t.hairline(),
                ));
                panel.painter().set(where_bg, Shape::Vec(shapes));
                ui.ctx().data_mut(|d| d.insert_temp(size_id, body.height()));
                (inner.inner, panel.response())
            })
            .inner
        });

        if interactive {
            let is_top = ctx.memory(|m| m.top_modal_layer()) == Some(out.response.layer_id);
            if is_top && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                close = true;
            }
        }
        if close && let Some(open) = self.open {
            *open = false;
        }

        let (inner, response) = out.inner;
        Some(DialogResponse {
            inner,
            response,
            presence,
        })
    }

    fn header(&self, ui: &mut Ui) {
        let t = theme(ui);
        ui.horizontal(|ui| {
            if let Some(title) = &self.title {
                ui.label(t.text(title, TextRole::Title));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(Kbd::new("esc"));
            });
        });
        if let Some(d) = &self.description {
            ui.add_space(t.space.xxs);
            ui.label(t.text(d, TextRole::Body).color(t.colors.text_muted));
        }
        ui.add_space(t.space.xl);
    }
}
