//! Plumbing shared by floating components (Tooltip, Popover, Menu, Select):
//! an [`Area`] placed next to an anchor by [`crate::behavior::place`], that
//! fades and drifts in from the anchor, and the surface drawn behind it.

use egui::{
    Align, Area, AreaState, Color32, Context, CornerRadius, Frame, Id, Layout, Margin, Order, Rect,
    Response, Sense, Shape, Stroke, Ui, UiBuilder, Vec2, accesskit::Role, emath::TSTransform, vec2,
};

use crate::{
    behavior::{Placement, Side, place},
    paint,
    tokens::ShadowLayer,
};

/// Content that floats next to an anchor rect.
pub(crate) struct Floating {
    pub id: Id,
    pub anchor: Rect,
    pub placement: Placement,
    pub order: Order,
    pub interactable: bool,
    /// Cover the screen with an invisible backdrop that takes the pointer,
    /// so the page behind doesn't react while this is open.
    pub backdrop: bool,
    /// How far the content drifts in from the anchor, in points.
    pub drift: f32,
    pub role: Option<Role>,
}

/// What [`Floating::show`] returns.
pub(crate) struct Shown<R> {
    pub inner: R,
    /// The area's response.
    pub response: Response,
}

impl Floating {
    /// Where the content with this `id` was last shown, without the
    /// animation transform.
    pub fn last_rect(ctx: &Context, id: Id) -> Option<Rect> {
        ctx.data(|d| d.get_temp(id.with("beau.floating_rect")))
    }

    /// Show the content with presence progress `t` (0 = absent, 1 = present).
    ///
    /// The content is placed with its size from the previous pass; the first
    /// pass is an invisible sizing pass, so it never shows misplaced.
    pub fn show<R>(self, ctx: &Context, t: f32, add: impl FnOnce(&mut Ui) -> R) -> Shown<R> {
        let screen = ctx.content_rect();
        let rect_id = self.id.with("beau.floating_rect");
        let size = if self.backdrop {
            Self::last_rect(ctx, self.id).map(|r| r.size())
        } else {
            AreaState::load(ctx, self.id).and_then(|s| s.size)
        };
        let placed = place(
            self.anchor,
            size.unwrap_or(Vec2::ZERO),
            screen,
            self.placement,
        );
        let k = t.clamp(0.0, 1.0);

        // Settle in from the anchor: drift away from it and scale up from 97%
        // about the edge that faces it.
        let r = placed.rect;
        let (pivot, towards_anchor) = match placed.side {
            Side::Bottom => (r.center_top(), vec2(0.0, -1.0)),
            Side::Top => (r.center_bottom(), vec2(0.0, 1.0)),
            Side::Right => (r.left_center(), vec2(-1.0, 0.0)),
            Side::Left => (r.right_center(), vec2(1.0, 0.0)),
        };
        let pivot = pivot.to_vec2();
        let transform = TSTransform::from_translation(towards_anchor * self.drift * (1.0 - k))
            * TSTransform::from_translation(pivot)
            * TSTransform::from_scaling(0.97 + 0.03 * k)
            * TSTransform::from_translation(-pivot);
        let animated = move |ui: &mut Ui, add: Box<dyn FnOnce(&mut Ui) -> R + '_>| {
            ui.with_visual_transform(transform, |ui| {
                ui.multiply_opacity(k);
                add(ui)
            })
            .inner
        };

        let mut area = Area::new(self.id)
            .order(self.order)
            .constrain(false)
            .interactable(self.interactable);
        if let Some(role) = self.role {
            area = area.role(role);
        }

        if !self.backdrop {
            let out = area
                .fixed_pos(r.min)
                .show(ctx, |ui| animated(ui, Box::new(add)));
            ctx.data_mut(|d| d.insert_temp(rect_id, out.response.rect));
            return Shown {
                inner: out.inner,
                response: out.response,
            };
        }

        let out = area.fixed_pos(screen.min).show(ctx, |ui| {
            // Takes clicks and drags anywhere on screen, without being a tab stop.
            let mut backdrop = ui.new_child(
                UiBuilder::new()
                    .sense(Sense::CLICK | Sense::DRAG)
                    .max_rect(screen),
            );
            backdrop.set_min_size(screen.size());

            let max = vec2(screen.width() - 2.0 * self.placement.margin, f32::INFINITY);
            let mut builder = UiBuilder::new()
                .max_rect(Rect::from_min_size(r.min, max))
                .layout(Layout::top_down(Align::Min));
            if size.is_none() {
                builder = builder.sizing_pass().invisible();
            }
            let mut content = ui.new_child(builder);
            let inner = animated(&mut content, Box::new(add));
            (inner, content.min_rect())
        });
        let (inner, rect) = out.inner;
        ctx.data_mut(|d| d.insert_temp(rect_id, rect));
        if size.is_none() {
            ctx.request_repaint();
        }
        Shown {
            inner,
            response: out.response,
        }
    }
}

/// How a floating surface looks.
pub(crate) struct SurfaceStyle<'a> {
    pub fill: Color32,
    pub stroke: Stroke,
    pub radius: CornerRadius,
    pub shadow: &'a [ShadowLayer],
    pub padding: Margin,
    /// Exact outer width, if set.
    pub width: Option<f32>,
    /// Minimum outer width.
    pub min_width: f32,
}

/// Add `add_contents` with padding, then paint the surface (shadow, fill and
/// hairline) behind it.
pub(crate) fn surface<R>(
    ui: &mut Ui,
    style: &SurfaceStyle<'_>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let where_bg = ui.painter().add(Shape::Noop);
    let pad = style.padding.sum();
    let inner = Frame::NONE.inner_margin(style.padding).show(ui, |ui| {
        if let Some(width) = style.width {
            ui.set_width((width - pad.x).max(0.0));
        }
        ui.set_min_width((style.min_width - pad.x).max(0.0));
        add_contents(ui)
    });
    let rect = inner.response.rect;
    let mut shapes: Vec<Shape> = paint::layered_shadow(rect, style.radius, style.shadow).collect();
    shapes.push(paint::surface(rect, style.radius, style.fill, style.stroke));
    ui.painter().set(where_bg, Shape::Vec(shapes));
    inner.inner
}
