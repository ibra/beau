use egui::{Rect, Shape, Stroke, Ui, pos2, style::WidgetVisuals};

/// Paint an up/down chevron pair, the indicator for "pick from a list".
///
/// Its signature matches egui's `ComboBox::icon`, so stock combo boxes can
/// use it. The colour comes from the widget's `fg_stroke`.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let mut choice = 0;
/// egui::ComboBox::from_id_salt("priority")
///     .selected_text("Medium")
///     .icon(beau::paint::select_chevrons)
///     .show_ui(ui, |ui| {
///         ui.selectable_value(&mut choice, 0, "Medium");
///     });
/// # });
/// ```
pub fn select_chevrons(ui: &Ui, rect: Rect, visuals: &WidgetVisuals, _open: bool) {
    let c = rect.center();
    // 7pt wide, 3.5pt tall arms, 3pt apart: reads at 13px body size.
    let (half_w, arm, gap) = (3.5, 3.5, 1.5);
    let stroke = Stroke::new(1.25, visuals.fg_stroke.color);
    let up = [
        pos2(c.x - half_w, c.y - gap),
        pos2(c.x, c.y - gap - arm),
        pos2(c.x + half_w, c.y - gap),
    ];
    let down = [
        pos2(c.x - half_w, c.y + gap),
        pos2(c.x, c.y + gap + arm),
        pos2(c.x + half_w, c.y + gap),
    ];
    let p = ui.painter();
    p.add(Shape::line(up.to_vec(), stroke));
    p.add(Shape::line(down.to_vec(), stroke));
}
