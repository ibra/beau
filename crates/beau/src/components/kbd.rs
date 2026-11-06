use egui::{CornerRadius, FontId, Response, Sense, Stroke, Ui, Widget, vec2};

use crate::{paint, theme, tokens::Weight, typography};

/// A keyboard key hint, like `esc` or `⌘K`.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// ui.add(beau::components::Kbd::new("esc"));
/// # });
/// ```
#[must_use = "add it with `ui.add(...)`"]
pub struct Kbd {
    text: String,
}

impl Kbd {
    /// A key hint with the given label.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for Kbd {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme(ui);
        let c = &t.colors;
        let font = FontId::new(11.0, typography::family(Weight::Medium));
        let galley = ui.painter().layout_no_wrap(self.text, font, c.text_subtle);
        let size = vec2((galley.size().x + 2.0 * t.space.sm).max(20.0), 20.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            let p = ui.painter();
            p.add(paint::surface(
                rect,
                CornerRadius::same(t.radius.sm),
                c.surface,
                Stroke::new(1.0, c.border),
            ));
            p.galley(rect.center() - galley.size() / 2.0, galley, c.text_subtle);
        }
        response
    }
}
