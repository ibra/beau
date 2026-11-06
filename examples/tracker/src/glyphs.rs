//! Small painted glyphs for the tracker: logo, status, priority, avatar.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2,
};

use beau::Theme;
use beau::tokens::Weight;

use crate::data::{Priority, Status};

/// The workspace logo: a lime tile with an ink "b".
pub fn logo(ui: &mut Ui) {
    let t = beau::theme(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
    let p = ui.painter();
    let gradient = beau::paint::LinearGradient::vertical(
        t.colors.accent.lerp_to_gamma(Color32::WHITE, 0.08),
        t.colors.accent,
    );
    p.add(beau::paint::gradient_rect(
        ui.ctx(),
        rect,
        CornerRadius::same(t.radius.md),
        &gradient,
    ));
    let font = FontId::new(13.0, beau::typography::family(Weight::Semibold));
    p.text(
        rect.center() + vec2(0.0, -0.5),
        egui::Align2::CENTER_CENTER,
        "b",
        font,
        t.colors.text_on_accent,
    );
}

/// Status glyph centred at `c` (14px).
pub fn status(p: &Painter, c: Pos2, status: Status, t: &Theme) {
    let r = 6.0;
    let col = &t.colors;
    match status {
        Status::Backlog => {
            let n = 10;
            for i in 0..n {
                let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
                let a1 = a0 + std::f32::consts::TAU / n as f32 * 0.5;
                let pt = |a: f32| c + vec2(a.cos(), a.sin()) * r;
                p.line_segment([pt(a0), pt(a1)], Stroke::new(1.5, col.text_subtle));
            }
        }
        Status::Todo => {
            p.circle_stroke(c, r, Stroke::new(1.5, col.text_muted));
        }
        Status::InProgress => {
            // Same muted ring as Todo; only the half-fill carries ink, so the
            // shape says "in progress" without outweighing the other statuses.
            p.circle_stroke(c, r, Stroke::new(1.5, col.text_muted));
            let ring = col.text;
            let inner = r - 2.5;
            let mut pts = vec![c];
            for i in 0..=16 {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 / 16.0 * std::f32::consts::PI;
                pts.push(c + vec2(a.cos(), a.sin()) * inner);
            }
            p.add(Shape::convex_polygon(pts, ring, Stroke::NONE));
        }
        Status::Done => {
            p.circle_filled(c, r + 0.75, col.text_muted);
            let s = Stroke::new(1.6, col.surface);
            p.line_segment([c + vec2(-2.8, 0.2), c + vec2(-0.8, 2.2)], s);
            p.line_segment([c + vec2(-0.8, 2.2), c + vec2(3.0, -2.0)], s);
        }
    }
}

/// Priority glyph centred at `c` (14px).
pub fn priority(p: &Painter, c: Pos2, priority: Priority, t: &Theme) {
    let col = &t.colors;
    match priority {
        Priority::None => {
            for dx in [-4.0, 0.0, 4.0] {
                p.circle_filled(c + vec2(dx, 0.0), 1.1, col.text_subtle);
            }
        }
        Priority::Urgent => {
            // Quiet: a tinted tile with a red mark, not a solid block.
            let rect = Rect::from_center_size(c, vec2(14.0, 14.0));
            p.rect_filled(
                rect,
                CornerRadius::same(t.radius.sm),
                col.danger.gamma_multiply(0.14),
            );
            p.text(
                c,
                egui::Align2::CENTER_CENTER,
                "!",
                FontId::new(11.0, beau::typography::family(Weight::Semibold)),
                col.danger_text,
            );
        }
        level => {
            let filled = match level {
                Priority::Low => 1,
                Priority::Medium => 2,
                _ => 3,
            };
            for i in 0..3 {
                let h = 4.0 + 3.0 * i as f32;
                let x = c.x - 5.0 + 4.0 * i as f32;
                let bar = Rect::from_min_max(pos2(x, c.y + 5.0 - h), pos2(x + 2.5, c.y + 5.0));
                let color = if i < filled {
                    col.text_muted
                } else {
                    col.border_strong
                };
                p.rect_filled(bar, CornerRadius::same(1), color);
            }
        }
    }
}

/// Assignee avatar (22px) or an empty slot.
pub fn avatar(p: &Painter, c: Pos2, initials: Option<&str>, t: &Theme) {
    let col = &t.colors;
    match initials {
        Some(text) => {
            p.circle_filled(c, 11.0, col.surface_active);
            let font = FontId::new(10.5, beau::typography::family(Weight::Semibold));
            p.text(c, egui::Align2::CENTER_CENTER, text, font, col.text_muted);
        }
        None => {
            p.circle_stroke(c, 10.5, Stroke::new(1.0, col.border_strong));
        }
    }
}
