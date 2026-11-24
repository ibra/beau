use egui::{
    Color32, CornerRadius, FontId, Key, Modifiers, Response, RichText, ScrollArea, Sense, Stroke,
    Ui, Widget, WidgetInfo, accesskit::Role, pos2, vec2,
};

use super::Popover;
use crate::{
    Icon,
    behavior::{self, ListNav},
    icons, motion, paint, theme,
    tokens::Weight,
    typography,
};

/// What sits before an option's label.
#[derive(Clone, Copy)]
pub(crate) enum Lead {
    None,
    Dot(Color32),
    Icon(Icon),
}

impl Lead {
    /// Paint it centred in a slot of `size` at `center`.
    pub(crate) fn paint(
        self,
        painter: &egui::Painter,
        center: egui::Pos2,
        size: f32,
        ink: Color32,
    ) {
        match self {
            Self::None => {}
            Self::Dot(color) => {
                painter.circle_filled(center, 3.5, color);
            }
            Self::Icon(icon) => {
                let color = icon.explicit_color().unwrap_or(ink);
                icon.paint_at(painter, center, icon.explicit_size().unwrap_or(size), color);
            }
        }
    }

    fn is_some(self) -> bool {
        !matches!(self, Self::None)
    }
}

struct SelectOption<T> {
    value: T,
    label: String,
    lead: Lead,
}

/// A dropdown for choosing one value from a list.
///
/// The trigger shows the chosen option; clicking it opens the options in a
/// [`Popover`] at least as wide as the trigger, with the chosen one checked.
///
/// **Keyboard:** Tab to the trigger; Enter, Space, Up or Down opens the list
/// with the chosen option focused; Up, Down, Home and End move; Enter or
/// Space chooses and closes; Escape closes without choosing. Focus then
/// returns to the trigger.
///
/// `ui.add(Select::new(..))` returns the trigger's response, which is
/// [`Response::changed`] on the pass a new value is chosen.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// use beau::components::Select;
/// let mut priority = 2;
/// ui.add(
///     Select::new("priority", &mut priority)
///         .label("Priority")
///         .option(1, "Low")
///         .option(2, "Medium")
///         .option(3, "High"),
/// );
/// # });
/// ```
#[must_use = "add it with `ui.add(...)`"]
pub struct Select<'v, T> {
    id_salt: egui::IdSalt,
    value: &'v mut T,
    options: Vec<SelectOption<T>>,
    label: Option<String>,
    placeholder: String,
    width: Option<f32>,
}

impl<'v, T: PartialEq + Clone> Select<'v, T> {
    /// A select editing `value`. `id_salt` must be unique within the
    /// parent `Ui`.
    pub fn new(id_salt: impl egui::AsIdSalt, value: &'v mut T) -> Self {
        Self {
            id_salt: egui::IdSalt::new(id_salt),
            value,
            options: Vec::new(),
            label: None,
            placeholder: "Select…".to_owned(),
            width: None,
        }
    }

    /// Add an option.
    pub fn option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push(SelectOption {
            value,
            label: label.into(),
            lead: Lead::None,
        });
        self
    }

    /// Add an option with a coloured dot before its label (projects,
    /// statuses).
    pub fn option_dot(mut self, value: T, label: impl Into<String>, color: Color32) -> Self {
        self.options.push(SelectOption {
            value,
            label: label.into(),
            lead: Lead::Dot(color),
        });
        self
    }

    /// Add an option with an icon before its label.
    pub fn option_icon(mut self, value: T, label: impl Into<String>, icon: Icon) -> Self {
        self.options.push(SelectOption {
            value,
            label: label.into(),
            lead: Lead::Icon(icon),
        });
        self
    }

    /// A label shown above the trigger (also its accessible name).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Text shown when the value matches no option ("Select…" by default).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Trigger width in points. By default it fills the available width,
    /// like [`crate::TextInput`].
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl<T: PartialEq + Clone> Widget for Select<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme(ui);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = t.space.sm;
            let label = self.label.as_ref().map(|l| {
                let font = FontId::new(12.0, typography::family(Weight::Medium));
                ui.label(RichText::new(l).font(font).color(t.colors.text_muted))
            });
            let id = ui.make_persistent_id(self.id_salt);
            let width = self.width.unwrap_or_else(|| ui.available_width());
            let (rect, _) = ui.allocate_exact_size(vec2(width, t.controls.md), Sense::hover());
            let mut response = ui.interact(rect, id, Sense::click());
            if let Some(label) = &label {
                response = response.labelled_by(label.id);
            }
            self.show(ui, response)
        })
        .inner
    }
}

impl<T: PartialEq + Clone> Select<'_, T> {
    fn show(self, ui: &mut Ui, mut response: Response) -> Response {
        let t = theme(ui);
        let ctx = ui.ctx().clone();
        let enabled = ui.is_enabled();
        let rect = response.rect;
        let chosen = self.options.iter().position(|o| o.value == *self.value);
        let text = chosen.map_or(self.placeholder.as_str(), |i| &self.options[i].label);
        let name = self.label.clone().unwrap_or_default();
        response.widget_info(|| WidgetInfo::labeled(Role::ComboBox, enabled, &name));
        let value = text.to_owned();
        ctx.accesskit_node_builder(response.id, |node| node.set_value(value));

        // Up and Down open the list too.
        let popover_id = response.id.with("beau.popover");
        if response.has_focus()
            && !Popover::is_open(&ctx, popover_id)
            && ui.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::ArrowDown)
                    | i.consume_key(Modifiers::NONE, Key::ArrowUp)
            })
        {
            Popover::set_open(&ctx, popover_id, true);
        }

        let open = Popover::is_open(&ctx, popover_id) || response.clicked();
        if ui.is_rect_visible(rect) {
            self.paint_trigger(ui, &response, text, chosen, open);
        }

        let mut picked = None;
        let lead_any = self.options.iter().any(|o| o.lead.is_some());
        Popover::new(&response)
            .min_width(rect.width())
            .padding(t.space.xs)
            .role(Role::ListBox)
            .show(&ctx, |ui| {
                // As wide as the widest option, and at least as wide as the trigger.
                let natural = self
                    .options
                    .iter()
                    .map(|o| option_width(ui, o, lead_any))
                    .fold(0.0, f32::max);
                ui.set_width(natural.max(rect.width() - 2.0 * t.space.xs));
                ScrollArea::vertical()
                    .max_height(t.controls.md * 9.5)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        let list_id = popover_id.with("list");
                        // Focus the chosen option when the list opens. Not in
                        // the sizing pass, where a focus request is dropped.
                        let appeared =
                            !ui.is_sizing_pass() && behavior::just_appeared(ui.ctx(), list_id);
                        let mut nav = ListNav::new(ui.ctx(), list_id);
                        for (i, option) in self.options.iter().enumerate() {
                            let chosen = chosen == Some(i);
                            let focus = chosen && appeared;
                            let row = option_row(ui, &mut nav, i, option, chosen, focus, lead_any);
                            if row.clicked() {
                                picked = Some(i);
                            }
                        }
                        nav.finish(ui);
                    });
                if picked.is_some() {
                    ui.close();
                }
            });

        if let Some(i) = picked
            && *self.value != self.options[i].value
        {
            *self.value = self.options[i].value.clone();
            response.mark_changed();
        }
        response
    }

    fn paint_trigger(
        &self,
        ui: &Ui,
        response: &Response,
        text: &str,
        chosen: Option<usize>,
        open: bool,
    ) {
        let t = theme(ui);
        let c = &t.colors;
        let ctx = ui.ctx();
        let enabled = ui.is_enabled();
        let rect = response.rect;
        let id = response.id;
        let hover = motion::toggle(
            ctx,
            id.with("beau.hover"),
            enabled && (response.hovered() || open),
            t.motion.fast,
        );
        let focus = motion::toggle(
            ctx,
            id.with("beau.focus"),
            response.has_focus() && !open,
            t.motion.base,
        );

        let radius = CornerRadius::same(t.radius.md);
        let (fill, border, ink) = if enabled {
            (
                c.surface_raised
                    .lerp_to_gamma(c.surface_raised.blend(c.surface_hover), hover),
                c.border_strong,
                c.text,
            )
        } else {
            (
                c.surface_raised.blend(c.surface_active),
                Color32::TRANSPARENT,
                c.text_subtle,
            )
        };
        let mut painter = ui.painter().clone();
        if !enabled {
            // Neutral and full opacity, like a disabled Button, not faded.
            painter.multiply_opacity(1.0 / ui.visuals().disabled_alpha().max(0.01));
        }
        let p = &painter;
        p.add(paint::focus_ring(rect, radius, c.focus_ring, focus));
        p.add(paint::surface(rect, radius, fill, Stroke::new(1.0, border)));

        let pad = t.space.lg - 2.0;
        let mut x = rect.left() + pad;
        let cy = rect.center().y;
        let slot = t.controls.icon;
        if let Some(lead) = chosen.map(|i| self.options[i].lead)
            && lead.is_some()
        {
            let icon_ink = if enabled { c.text_muted } else { ink };
            lead.paint(p, pos2(x + slot / 2.0, cy), slot, icon_ink);
            x += slot + t.space.sm;
        }
        let chevron = t.controls.icon - 2.0;
        let right = rect.right() - pad - chevron - t.space.sm;
        let color = if chosen.is_some() { ink } else { c.text_subtle };
        let galley = p.layout_no_wrap(text.to_owned(), t.type_scale.body.font_id(), color);
        let clip = egui::Rect::from_x_y_ranges(x..=right, rect.y_range());
        p.with_clip_rect(clip.intersect(p.clip_rect())).galley(
            pos2(x, cy - galley.size().y / 2.0),
            galley,
            color,
        );
        icons::CHEVRONS_UP_DOWN.paint_at(
            p,
            pos2(rect.right() - pad - chevron / 2.0, cy),
            chevron,
            c.text_subtle,
        );
    }
}

/// One option in the list: a 28pt row, highlighted while it has keyboard
/// focus, which follows the pointer.
fn option_row<T>(
    ui: &mut Ui,
    nav: &mut ListNav,
    i: usize,
    option: &SelectOption<T>,
    chosen: bool,
    focus: bool,
    lead_any: bool,
) -> Response {
    let t = theme(ui);
    let c = &t.colors;
    let sense = nav.sense(i);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), t.controls.md), sense);
    response.widget_info(|| WidgetInfo::selected(Role::ListBoxOption, true, chosen, &option.label));
    if focus {
        response.request_focus();
    }
    // The highlight follows the pointer as well as the keys.
    if response.hovered() && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO) {
        response.request_focus();
    }
    let focused = nav.item(ui, i, &response);
    let ctx = ui.ctx();
    let wash = motion::toggle(ctx, response.id.with("beau.wash"), focused, t.motion.fast);

    let p = ui.painter().clone();
    if wash > 0.0 {
        p.rect_filled(
            rect,
            CornerRadius::same(t.radius.sm),
            c.surface_hover.gamma_multiply(wash),
        );
    }
    // With the list's padding, labels and checks line up with the trigger's
    // label and chevrons.
    let pad = t.space.sm;
    let slot = t.controls.icon;
    let cy = rect.center().y;
    let mut x = rect.left() + pad;
    if lead_any {
        option
            .lead
            .paint(&p, pos2(x + slot / 2.0, cy), slot, c.text_muted);
        x += slot + t.space.sm;
    }
    let galley = p.layout_no_wrap(option.label.clone(), t.type_scale.body.font_id(), c.text);
    let check = t.controls.icon - 2.0;
    p.galley(pos2(x, cy - galley.size().y / 2.0), galley, c.text);
    if chosen {
        icons::CHECK.paint_at(
            &p,
            pos2(rect.right() - pad - check / 2.0, cy),
            check,
            c.text,
        );
    }
    response
}

/// The width an option's row needs: padding, lead, label, and room for the
/// check on every row, so the widest label never sits under it.
fn option_width<T>(ui: &Ui, option: &SelectOption<T>, lead_any: bool) -> f32 {
    let t = theme(ui);
    let label = ui
        .painter()
        .layout_no_wrap(
            option.label.clone(),
            t.type_scale.body.font_id(),
            Color32::PLACEHOLDER,
        )
        .size()
        .x;
    let lead = if lead_any {
        t.controls.icon + t.space.sm
    } else {
        0.0
    };
    2.0 * t.space.sm + lead + label + t.space.xl + t.controls.icon - 2.0
}
