use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

use crate::{
    paint::{self, PathCmd},
    theme,
};

/// An icon from the bundled set ([`crate::icons`]), drawn as vector strokes.
///
/// The size defaults to the theme's `controls.icon` (16) and the colour to
/// the surrounding text colour. The stroke is
/// [`ICON_STROKE`](crate::paint::ICON_STROKE) wide at every size.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::icons;
/// ui.add(icons::INBOX);
/// ui.add(icons::CIRCLE_CHECK.size(20.0).color(egui::Color32::from_rgb(0x30, 0xA4, 0x6C)));
/// # });
/// ```
///
/// Icons are decorative: they add nothing to the accessibility tree. Label
/// the control that contains them instead.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use = "add it with `ui.add(...)`"]
pub struct Icon {
    path: &'static [PathCmd],
    size: Option<f32>,
    color: Option<Color32>,
}

impl Icon {
    pub(crate) const fn from_path(path: &'static [PathCmd]) -> Self {
        Self {
            path,
            size: None,
            color: None,
        }
    }

    /// Set the size: the side of the square the icon fills, in points.
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the colour, instead of following the text colour.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// The size set with [`Icon::size`], if any.
    pub(crate) fn explicit_size(&self) -> Option<f32> {
        self.size
    }

    /// The colour set with [`Icon::color`], if any.
    pub(crate) fn explicit_color(&self) -> Option<Color32> {
        self.color
    }

    /// Paint the icon centred at `center`, for custom-painted rows and
    /// widgets. `size` and `color` are used as given; values set with
    /// [`Icon::size`] and [`Icon::color`] are ignored.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 32.0), egui::Sense::hover());
    /// let color = beau::theme(ui).colors.text_muted;
    /// beau::icons::FILE.paint_at(ui.painter(), rect.left_center() + egui::vec2(12.0, 0.0), 16.0, color);
    /// # });
    /// ```
    pub fn paint_at(&self, painter: &Painter, center: Pos2, size: f32, color: Color32) {
        let ppp = painter.pixels_per_point();
        // Start on a physical pixel, so the grid's straight lines stay sharp.
        let min = center - Vec2::splat(size / 2.0);
        let min = Pos2::new((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
        let rect = Rect::from_min_size(min, Vec2::splat(size));
        painter.extend(paint::stroke_path(
            self.path,
            rect,
            paint::ICON_STROKE,
            color,
            ppp,
        ));
    }
}

impl Widget for Icon {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size.unwrap_or_else(|| theme(ui).controls.icon);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        if ui.is_rect_visible(rect) {
            let color = self.color.unwrap_or_else(|| ui.visuals().text_color());
            self.paint_at(ui.painter(), rect.center(), size, color);
        }
        response
    }
}
