use egui::{Rect, Vec2, pos2};

/// The side of the anchor that floating content sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// Above the anchor.
    Top,
    /// Below the anchor (default).
    #[default]
    Bottom,
    /// Left of the anchor.
    Left,
    /// Right of the anchor.
    Right,
}

impl Side {
    /// The side across the anchor.
    pub fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

/// How floating content lines up with the anchor along the side's edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Left edges match (or top edges, for [`Side::Left`] and [`Side::Right`]).
    #[default]
    Start,
    /// Centres match.
    Center,
    /// Right edges match (or bottom edges).
    End,
}

/// Where floating content would like to go, relative to its anchor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Placement {
    /// Preferred side.
    pub side: Side,
    /// Alignment along that side.
    pub align: Align,
    /// Space between the anchor and the content, in points.
    pub gap: f32,
    /// Minimum space kept between the content and the edge of the bounds.
    pub margin: f32,
}

/// The result of [`place`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// Where to put the content.
    pub rect: Rect,
    /// The side actually used: the opposite of the preferred side if it
    /// flipped. Use it to pick the direction of an entry animation.
    pub side: Side,
}

/// Position floating content of `size` next to `anchor`, inside `bounds`
/// (usually the screen).
///
/// 1. Place it on the preferred side with the preferred alignment.
/// 2. **Flip** to the opposite side if it doesn't fit there and the opposite
///    side has more room.
/// 3. **Clamp** it into `bounds` (inset by the margin), sliding along the
///    anchor's edge. Content larger than the bounds is pinned to the top-left.
///
/// ```
/// use beau::behavior::{Placement, Side, place};
/// use egui::{Rect, pos2, vec2};
///
/// let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
/// let button = Rect::from_min_size(pos2(40.0, 560.0), vec2(80.0, 28.0));
/// let menu = place(button, vec2(160.0, 200.0), screen, Placement { gap: 4.0, ..Default::default() });
/// // No room below the button, so the menu opens above it.
/// assert_eq!(menu.side, Side::Top);
/// assert_eq!(menu.rect.bottom(), 556.0);
/// ```
pub fn place(anchor: Rect, size: Vec2, bounds: Rect, placement: Placement) -> Placed {
    let Placement {
        side,
        align,
        gap,
        margin,
    } = placement;
    let inner = bounds.shrink(margin);

    // Room for the content on a side, beyond the gap.
    let room = |side: Side| match side {
        Side::Top => anchor.top() - gap - inner.top(),
        Side::Bottom => inner.bottom() - anchor.bottom() - gap,
        Side::Left => anchor.left() - gap - inner.left(),
        Side::Right => inner.right() - anchor.right() - gap,
    };
    let needed = if side.is_vertical() { size.y } else { size.x };
    let side = if room(side) < needed && room(side.opposite()) > room(side) {
        side.opposite()
    } else {
        side
    };

    let along = |start: f32, end: f32, len: f32| match align {
        Align::Start => start,
        Align::Center => (start + end - len) / 2.0,
        Align::End => end - len,
    };
    let min = match side {
        Side::Top => pos2(
            along(anchor.left(), anchor.right(), size.x),
            anchor.top() - gap - size.y,
        ),
        Side::Bottom => pos2(
            along(anchor.left(), anchor.right(), size.x),
            anchor.bottom() + gap,
        ),
        Side::Left => pos2(
            anchor.left() - gap - size.x,
            along(anchor.top(), anchor.bottom(), size.y),
        ),
        Side::Right => pos2(
            anchor.right() + gap,
            along(anchor.top(), anchor.bottom(), size.y),
        ),
    };

    // `max` then `min` pins oversized content to the top-left edge.
    let clamp = |v: f32, lo: f32, hi: f32| v.min(hi).max(lo);
    let min = pos2(
        clamp(min.x, inner.left(), inner.right() - size.x),
        clamp(min.y, inner.top(), inner.bottom() - size.y),
    );
    Placed {
        rect: Rect::from_min_size(min, size),
        side,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    fn screen() -> Rect {
        Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))
    }

    fn anchor(x: f32, y: f32) -> Rect {
        Rect::from_min_size(pos2(x, y), vec2(80.0, 28.0))
    }

    fn below(gap: f32) -> Placement {
        Placement {
            gap,
            ..Default::default()
        }
    }

    #[test]
    fn prefers_the_requested_side_and_alignment() {
        let p = place(
            anchor(100.0, 100.0),
            vec2(160.0, 200.0),
            screen(),
            below(4.0),
        );
        assert_eq!(p.side, Side::Bottom);
        assert_eq!(p.rect.min, pos2(100.0, 132.0));

        let centred = Placement {
            side: Side::Top,
            align: Align::Center,
            gap: 4.0,
            margin: 0.0,
        };
        let p = place(anchor(300.0, 400.0), vec2(160.0, 40.0), screen(), centred);
        assert_eq!(p.side, Side::Top);
        assert_eq!(p.rect.min, pos2(260.0, 356.0));

        let end = Placement {
            side: Side::Right,
            align: Align::End,
            gap: 8.0,
            margin: 0.0,
        };
        let p = place(anchor(300.0, 400.0), vec2(100.0, 60.0), screen(), end);
        assert_eq!(p.rect.min, pos2(388.0, 368.0));
    }

    #[test]
    fn flips_when_the_preferred_side_is_too_small() {
        let p = place(
            anchor(100.0, 560.0),
            vec2(160.0, 200.0),
            screen(),
            below(4.0),
        );
        assert_eq!(p.side, Side::Top);
        assert_eq!(p.rect.bottom(), 556.0);

        let left = Placement {
            side: Side::Left,
            ..Default::default()
        };
        let p = place(anchor(10.0, 100.0), vec2(160.0, 40.0), screen(), left);
        assert_eq!(p.side, Side::Right);
        assert_eq!(p.rect.left(), 90.0);
    }

    #[test]
    fn stays_put_when_the_opposite_side_is_smaller() {
        // 500 tall content: 372 of room below, 200 above. Neither fits, so it
        // keeps the roomier preferred side and is clamped.
        let p = place(
            anchor(100.0, 200.0),
            vec2(160.0, 500.0),
            screen(),
            below(0.0),
        );
        assert_eq!(p.side, Side::Bottom);
        assert_eq!(p.rect.bottom(), 600.0);
    }

    #[test]
    fn clamps_to_the_bounds_with_a_margin() {
        let placement = Placement {
            margin: 8.0,
            gap: 4.0,
            ..Default::default()
        };
        // Would run off the right edge.
        let p = place(
            anchor(700.0, 100.0),
            vec2(160.0, 100.0),
            screen(),
            placement,
        );
        assert_eq!(p.rect.right(), 792.0);
        // Would run off the left edge.
        let end = Placement {
            align: Align::End,
            ..placement
        };
        let p = place(anchor(20.0, 100.0), vec2(160.0, 100.0), screen(), end);
        assert_eq!(p.rect.left(), 8.0);
    }

    #[test]
    fn oversized_content_is_pinned_to_the_top_left() {
        let p = place(
            anchor(100.0, 100.0),
            vec2(900.0, 700.0),
            screen(),
            below(4.0),
        );
        assert_eq!(p.rect.min, pos2(0.0, 0.0));
    }
}
