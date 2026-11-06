//! Non-colour scales: type, spacing, radii, controls, elevation and motion.

use egui::{Color32, Vec2, vec2};

use crate::motion::Spring;

/// A font weight available in beau's bundled Inter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weight {
    /// 400.
    Regular,
    /// 500.
    Medium,
    /// 600.
    Semibold,
    /// 600 at Inter's display optical size, for headings.
    Display,
}

/// One entry of the type scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSpec {
    /// Font size in points.
    pub size: f32,
    /// Font weight.
    pub weight: Weight,
    /// Line height in points.
    pub line_height: f32,
    /// Extra letter spacing in points (negative tightens).
    pub tracking: f32,
}

/// Named roles in the type scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextRole {
    /// 11px medium: section labels, keyboard hints.
    Caption,
    /// 12px: metadata, IDs, field labels.
    Small,
    /// 13px: default body text.
    Body,
    /// 13px medium: emphasised body text, button labels.
    BodyStrong,
    /// 15px semibold: dialog and card titles.
    Title,
    /// 18px semibold: page headings.
    Heading,
    /// 24px semibold: hero text.
    Display,
}

/// The type scale. See [`TextRole`].
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScale {
    /// See [`TextRole::Caption`].
    pub caption: TextSpec,
    /// See [`TextRole::Small`].
    pub small: TextSpec,
    /// See [`TextRole::Body`].
    pub body: TextSpec,
    /// See [`TextRole::BodyStrong`].
    pub body_strong: TextSpec,
    /// See [`TextRole::Title`].
    pub title: TextSpec,
    /// See [`TextRole::Heading`].
    pub heading: TextSpec,
    /// See [`TextRole::Display`].
    pub display: TextSpec,
}

impl Default for TypeScale {
    /// beau's type scale.
    fn default() -> Self {
        let spec = |size, weight, line_height, tracking| TextSpec {
            size,
            weight,
            line_height,
            tracking,
        };
        Self {
            caption: spec(11.0, Weight::Medium, 16.0, 0.1),
            small: spec(12.0, Weight::Regular, 16.0, 0.0),
            body: spec(13.0, Weight::Regular, 20.0, 0.0),
            body_strong: spec(13.0, Weight::Medium, 20.0, 0.0),
            title: spec(15.0, Weight::Semibold, 22.0, -0.1),
            heading: spec(18.0, Weight::Display, 24.0, -0.25),
            display: spec(24.0, Weight::Display, 30.0, -0.5),
        }
    }
}

impl TypeScale {
    /// Look up the spec for a role.
    pub fn get(&self, role: TextRole) -> TextSpec {
        match role {
            TextRole::Caption => self.caption,
            TextRole::Small => self.small,
            TextRole::Body => self.body,
            TextRole::BodyStrong => self.body_strong,
            TextRole::Title => self.title,
            TextRole::Heading => self.heading,
            TextRole::Display => self.display,
        }
    }
}

/// The spacing scale, in points.
#[derive(Clone, Debug, PartialEq)]
pub struct Space {
    /// 2
    pub xxs: f32,
    /// 4
    pub xs: f32,
    /// 6
    pub sm: f32,
    /// 8
    pub md: f32,
    /// 12
    pub lg: f32,
    /// 16
    pub xl: f32,
    /// 24
    pub xxl: f32,
    /// 32
    pub xxxl: f32,
}

impl Default for Space {
    fn default() -> Self {
        Self {
            xxs: 2.0,
            xs: 4.0,
            sm: 6.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 24.0,
            xxxl: 32.0,
        }
    }
}

/// Corner radii, in points.
#[derive(Clone, Debug, PartialEq)]
pub struct Radii {
    /// 4: checkboxes, keycaps, small marks.
    pub sm: u8,
    /// 6: buttons, inputs.
    pub md: u8,
    /// 10: cards, dialogs, popovers.
    pub lg: u8,
    /// Fully rounded (pills, avatars).
    pub full: u8,
}

impl Default for Radii {
    fn default() -> Self {
        Self {
            sm: 4,
            md: 6,
            lg: 10,
            full: u8::MAX,
        }
    }
}

/// Standard control heights, in points.
#[derive(Clone, Debug, PartialEq)]
pub struct Controls {
    /// Small control height (24).
    pub sm: f32,
    /// Medium control height (28), the default.
    pub md: f32,
    /// Large control height (32).
    pub lg: f32,
    /// List row height (32).
    pub row: f32,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            sm: 24.0,
            md: 28.0,
            lg: 32.0,
            row: 32.0,
        }
    }
}

/// One layer of a layered shadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    /// Offset of the shadow from the caster.
    pub offset: Vec2,
    /// Blur width in points.
    pub blur: f32,
    /// Grow (positive) or shrink (negative) the shadow relative to the caster.
    pub spread: f32,
    /// Shadow colour (usually ink at low alpha).
    pub color: Color32,
}

/// How high a surface floats above the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Level {
    /// On the page: no shadow.
    #[default]
    Flat,
    /// Slightly lifted: cards.
    Raised,
    /// Floating: popovers, menus.
    Overlay,
    /// Above everything: dialogs.
    Modal,
}

/// Layered shadow specs per [`Level`].
#[derive(Clone, Debug, PartialEq)]
pub struct Elevation {
    /// Shadows for [`Level::Raised`].
    pub raised: Vec<ShadowLayer>,
    /// Shadows for [`Level::Overlay`].
    pub overlay: Vec<ShadowLayer>,
    /// Shadows for [`Level::Modal`].
    pub modal: Vec<ShadowLayer>,
}

impl Elevation {
    /// Shadow layers for a level (empty for [`Level::Flat`]).
    pub fn get(&self, level: Level) -> &[ShadowLayer] {
        match level {
            Level::Flat => &[],
            Level::Raised => &self.raised,
            Level::Overlay => &self.overlay,
            Level::Modal => &self.modal,
        }
    }

    /// beau's elevation, cast in `ink`. `strength` scales shadow alpha (dark mode needs more).
    pub fn from_ink(ink: Color32, strength: f32) -> Self {
        let layer = |y: f32, blur: f32, spread: f32, alpha: f32| ShadowLayer {
            offset: vec2(0.0, y),
            blur,
            spread,
            color: ink.gamma_multiply((alpha * strength).min(1.0)),
        };
        Self {
            raised: vec![layer(1.0, 2.0, 0.0, 0.04)],
            overlay: vec![layer(1.0, 2.0, 0.0, 0.06), layer(6.0, 16.0, -2.0, 0.08)],
            modal: vec![
                layer(1.0, 2.0, 0.0, 0.08),
                layer(8.0, 24.0, -4.0, 0.12),
                layer(24.0, 56.0, -8.0, 0.14),
            ],
        }
    }
}

/// Motion timings.
#[derive(Clone, Debug, PartialEq)]
pub struct MotionTokens {
    /// Hover and press feedback.
    pub fast: Spring,
    /// Focus, dialogs, presence.
    pub base: Spring,
    /// Layout shifts.
    pub slow: Spring,
    /// When true every animation snaps to its target.
    pub reduced: bool,
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            fast: Spring::critical(0.12),
            base: Spring::critical(0.18),
            slow: Spring::critical(0.28),
            reduced: false,
        }
    }
}
