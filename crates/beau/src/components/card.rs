use egui::{Color32, CornerRadius, Frame, InnerResponse, Margin, Shape, Ui};

use crate::{paint, theme, tokens::Level};

/// A surface that groups related content.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::{Card, tokens::Level};
/// Card::new().elevation(Level::Raised).show(ui, |ui| {
///     ui.label("Inside a card");
/// });
/// # });
/// ```
#[must_use = "call `.show(ui, |ui| ...)`"]
pub struct Card {
    level: Level,
    padding: Option<Margin>,
    fill: Option<Color32>,
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    /// A raised card with default padding.
    pub fn new() -> Self {
        Self {
            level: Level::Raised,
            padding: None,
            fill: None,
        }
    }

    /// How high the card floats ([`Level::Flat`] has no shadow).
    pub fn elevation(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Inner padding (defaults to the `xl` spacing token).
    pub fn padding(mut self, padding: impl Into<Margin>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    /// Override the fill colour (defaults to `surface_raised`).
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Show the card with the given contents.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = theme(ui);
        let where_bg = ui.painter().add(Shape::Noop);
        let padding = self
            .padding
            .unwrap_or_else(|| Margin::same(t.space.xl as i8));
        let inner = Frame::NONE.inner_margin(padding).show(ui, add_contents);

        let rect = inner.response.rect;
        let radius = CornerRadius::same(t.radius.lg);
        let mut shapes: Vec<Shape> =
            paint::layered_shadow(rect, radius, t.elevation.get(self.level)).collect();
        shapes.push(paint::surface(
            rect,
            radius,
            self.fill.unwrap_or(t.colors.surface_raised),
            t.divider(ui.ctx()),
        ));
        ui.painter().set(where_bg, Shape::Vec(shapes));
        inner
    }
}
