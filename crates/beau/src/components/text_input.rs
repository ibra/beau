use egui::{
    CornerRadius, FontId, Frame, Margin, Response, RichText, Shape, Stroke, TextEdit, Ui, Widget,
};

use crate::{motion, paint, theme, tokens::Weight, typography};

/// A text field with an optional label, placeholder and animated focus ring.
///
/// `ui.add(TextInput::new(..))` returns the response of the text field itself,
/// so `changed()`, `lost_focus()` etc. work as with [`egui::TextEdit`].
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let mut title = String::new();
/// ui.add(
///     beau::TextInput::new(&mut title)
///         .label("Title")
///         .placeholder("What needs doing?"),
/// );
/// # });
/// ```
#[must_use = "add it with `ui.add(...)`"]
pub struct TextInput<'t> {
    text: &'t mut String,
    label: Option<String>,
    placeholder: Option<String>,
    rows: Option<usize>,
    autofocus: bool,
    id_salt: Option<egui::IdSalt>,
}

impl<'t> TextInput<'t> {
    /// A single-line input editing `text`.
    pub fn new(text: &'t mut String) -> Self {
        Self {
            text,
            label: None,
            placeholder: None,
            rows: None,
            autofocus: false,
            id_salt: None,
        }
    }

    /// A label shown above the field (also its accessible name).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Hint text shown while the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Make the field multi-line with the given visible row count.
    pub fn multiline(mut self, rows: usize) -> Self {
        self.rows = Some(rows.max(1));
        self
    }

    /// Focus the field when it first appears.
    pub fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    /// A stable id salt, needed if the field's position in the ui can change.
    pub fn id_salt(mut self, salt: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(egui::IdSalt::new(salt));
        self
    }
}

impl Widget for TextInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme(ui);
        let c = &t.colors;
        let body = t.type_scale.body;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = t.space.sm;
            let label = self.label.map(|l| {
                let font = FontId::new(12.0, typography::family(Weight::Medium));
                ui.label(RichText::new(l).font(font).color(c.text_muted))
            });

            let where_bg = ui.painter().add(Shape::Noop);
            let width = ui.available_width();
            let pad_x = t.space.lg - 2.0;
            // Single-line fields are exactly one control tall: the TextEdit
            // itself fills the height and centres its text.
            let pad_y = if self.rows.is_some() {
                t.space.sm + 1.0
            } else {
                0.0
            };

            let mut edit = match self.rows {
                Some(rows) => TextEdit::multiline(self.text).desired_rows(rows),
                None => TextEdit::singleline(self.text),
            }
            .frame(Frame::NONE)
            .margin(Margin::ZERO)
            .font(body.font_id())
            .text_color(c.text)
            .desired_width(f32::INFINITY);
            if let Some(salt) = self.id_salt {
                edit = edit.id_salt(salt);
            }
            if let Some(p) = self.placeholder {
                edit = edit.hint_text(RichText::new(p).color(c.text_subtle));
            }
            if self.rows.is_none() {
                edit = edit
                    .min_size(egui::vec2(0.0, t.controls.md))
                    .align(egui::Align2::LEFT_CENTER);
            }

            let field = Frame::NONE
                .inner_margin(Margin::symmetric(pad_x as i8, pad_y.round() as i8))
                .show(ui, |ui| {
                    ui.set_width(width - 2.0 * pad_x);
                    match &label {
                        Some(l) => ui.label_inputs_by(l.id, |ui| ui.add(edit)),
                        None => ui.add(edit),
                    }
                });
            let response = field.inner;
            let rect = field.response.rect;

            // Hover-only: a click-sensing area on top would steal focus from the field.
            let frame_hit = ui.interact(rect, response.id.with("beau.frame"), egui::Sense::hover());

            // Skip the invisible sizing pass of a new Area: a focus request
            // made there is dropped, which made autofocus timing-dependent.
            if self.autofocus && !ui.is_sizing_pass() && super::just_appeared(ui.ctx(), response.id)
            {
                response.request_focus();
            }

            let ctx = ui.ctx();
            let id = response.id;
            let hovered = response.hovered() || frame_hit.hovered();
            let hover = motion::toggle(ctx, id.with("beau.hover"), hovered, t.motion.fast);
            let focused = response.has_focus();
            let focus = motion::toggle(ctx, id.with("beau.focus"), focused, t.motion.base);

            let radius = CornerRadius::same(t.radius.md);
            let border = c
                .border_strong
                .lerp_to_gamma(c.text_subtle, hover * 0.6)
                .lerp_to_gamma(c.focus_ring, focus);
            ui.painter().set(
                where_bg,
                Shape::Vec(vec![
                    paint::focus_halo(rect, radius, c.focus_ring, focus),
                    paint::surface(rect, radius, c.surface_raised, Stroke::new(1.0, border)),
                ]),
            );
            response
        })
        .inner
    }
}
