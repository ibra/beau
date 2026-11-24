use eframe::egui::{
    self, Align, Key, Layout, Margin, Modifiers, Sense, Ui, UiBuilder, WidgetInfo, vec2,
};

use beau::behavior::ListNav;
use beau::motion::PresenceList;
use beau::tokens::TextRole;
use beau::{Button, Card, Checkbox, Dialog, Select, Size, Switch, TextInput, Tooltip, icons};

use crate::Options;
use crate::data::{self, Filter, Project, Status, Task};
use crate::glyphs;

pub struct TrackerApp {
    tasks: Vec<Task>,
    projects: Vec<Project>,
    filter: Filter,
    /// The project the list is narrowed to, combined with `filter`.
    project: Option<usize>,
    /// The task whose row has keyboard focus.
    selected: Option<u32>,
    /// Give this task's row keyboard focus when it is next shown.
    focus_task: Option<u32>,
    /// Tasks that just changed status: each stays in its old group for a
    /// moment, so the new status shows before the row leaves.
    lingering: Vec<Linger>,
    /// Deleted tasks and where they were, most recent last (for undo).
    deleted: Vec<(usize, Task)>,
    /// The `--stock` page, shown instead of the tracker.
    stock: Option<crate::stock::StockPage>,
    dialog_open: bool,
    /// The task the dialog edits; `None` creates a new one.
    editing: Option<u32>,
    draft_title: String,
    draft_description: String,
    draft_project: usize,
    draft_status: Status,
    /// Keep the dialog open after creating a task.
    create_another: bool,
    /// Put keyboard focus back in the title field on the next frame.
    refocus_title: bool,
    #[cfg(not(target_arch = "wasm32"))]
    screenshot: Option<crate::screenshot::ScreenshotJob>,
}

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: Options) -> Self {
        let mut app = Self {
            tasks: if opts.empty {
                Vec::new()
            } else {
                data::seed_tasks()
            },
            projects: data::projects(),
            filter: Filter::Active,
            project: None,
            selected: None,
            focus_task: None,
            lingering: Vec::new(),
            deleted: Vec::new(),
            stock: None,
            dialog_open: opts.dialog,
            editing: None,
            draft_title: String::new(),
            draft_description: String::new(),
            draft_project: 0,
            draft_status: Status::Todo,
            create_another: false,
            refocus_title: false,
            #[cfg(not(target_arch = "wasm32"))]
            screenshot: opts.screenshot.map(crate::screenshot::ScreenshotJob::new),
        };
        let ctx = &cc.egui_ctx;
        if !opts.no_beau {
            beau::install(ctx);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let demo = app.screenshot.is_some();
        #[cfg(target_arch = "wasm32")]
        let demo = false;
        if opts.stock || opts.no_beau {
            app.stock = Some(crate::stock::StockPage::new(!opts.no_beau, demo));
        }
        // Follow the OS by default. Screenshots are deterministic: light unless --dark.
        #[cfg(not(target_arch = "wasm32"))]
        let fixed = opts.dark || demo;
        #[cfg(target_arch = "wasm32")]
        let fixed = opts.dark;
        if fixed {
            app.set_dark(ctx, opts.dark);
        }
        app
    }

    fn is_dark(ctx: &egui::Context) -> bool {
        ctx.theme() == egui::Theme::Dark
    }

    fn set_dark(&mut self, ctx: &egui::Context, dark: bool) {
        let pref = if dark {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        };
        beau::set_preference(ctx, pref);
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        // A focused row or button doesn't type, so shortcuts stay live.
        if ctx.text_edit_focused() || self.dialog_open {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::N)) {
            self.open_new_task();
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::D)) {
            self.set_dark(ctx, !Self::is_dark(ctx));
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z)) {
            self.undo_delete();
        }
    }

    // ------------------------------------------------------------------ tasks

    /// Whether `task` belongs to the current filter and project.
    fn in_view(&self, task: &Task) -> bool {
        self.filter.matches(task) && self.project.is_none_or(|p| task.project == p)
    }

    /// The rows of the list, top to bottom: a header per status, then its tasks.
    fn rows(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        for status in Status::ALL {
            let tasks: Vec<&Task> = self
                .tasks
                .iter()
                .filter(
                    |task| match self.lingering.iter().find(|l| l.number == task.number) {
                        Some(linger) => linger.group == status,
                        None => task.status == status && self.in_view(task),
                    },
                )
                .collect();
            if tasks.is_empty() {
                continue;
            }
            rows.push(Row::Header(status, tasks.len()));
            rows.extend(tasks.into_iter().cloned().map(Row::Task));
        }
        rows
    }

    /// The task to focus when `number` leaves the list: the next one, or
    /// the previous one if it was last.
    fn neighbour(&self, number: u32) -> Option<u32> {
        let numbers: Vec<u32> = self
            .rows()
            .iter()
            .filter_map(|row| match row {
                Row::Task(task) => Some(task.number),
                Row::Header(..) => None,
            })
            .collect();
        let i = numbers.iter().position(|n| *n == number)?;
        numbers
            .get(i + 1)
            .or(i.checked_sub(1).and_then(|i| numbers.get(i)))
            .copied()
    }

    /// Keep keyboard focus in the list when the focused row is about to go.
    fn move_focus_off(&mut self, number: u32) {
        if self.selected == Some(number) {
            self.focus_task = self.neighbour(number);
        }
    }

    fn toggle_done(&mut self, number: u32, now: f64) {
        let Some(task) = self.tasks.iter_mut().find(|t| t.number == number) else {
            return;
        };
        let old = task.status;
        task.status = if old == Status::Done {
            Status::Todo
        } else {
            Status::Done
        };
        // Toggled again while lingering: it stays in the group it is shown in.
        let group = match self.lingering.iter().position(|l| l.number == number) {
            Some(i) => self.lingering.remove(i).group,
            None => old,
        };
        self.lingering.push(Linger {
            number,
            group,
            until: now + LINGER_SECONDS,
        });
    }

    /// Let rows whose moment has passed leave (or settle, if they still belong).
    fn expire_lingering(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        while let Some(i) = self.lingering.iter().position(|l| l.until <= now) {
            let number = self.lingering[i].number;
            let stays = self
                .tasks
                .iter()
                .any(|t| t.number == number && self.in_view(t));
            if !stays {
                self.move_focus_off(number);
            }
            self.lingering.remove(i);
        }
        if let Some(next) = self.lingering.iter().map(|l| l.until).reduce(f64::min) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64((next - now).max(0.0)));
        }
    }

    fn delete_task(&mut self, number: u32) {
        let Some(i) = self.tasks.iter().position(|t| t.number == number) else {
            return;
        };
        self.move_focus_off(number);
        self.lingering.retain(|l| l.number != number);
        let task = self.tasks.remove(i);
        self.deleted.push((i, task));
    }

    fn undo_delete(&mut self) {
        let Some((i, task)) = self.deleted.pop() else {
            return;
        };
        self.focus_task = Some(task.number);
        self.tasks.insert(i.min(self.tasks.len()), task);
    }

    fn open_new_task(&mut self) {
        if self.editing.take().is_some() {
            self.draft_title.clear();
            self.draft_description.clear();
        }
        // A fresh draft starts in the project being viewed, as a to-do.
        if self.draft_title.is_empty() && self.draft_description.is_empty() {
            self.draft_project = self.project.unwrap_or(0);
            self.draft_status = Status::Todo;
        }
        self.dialog_open = true;
    }

    fn open_edit_task(&mut self, number: u32) {
        let Some(task) = self.tasks.iter().find(|t| t.number == number) else {
            return;
        };
        self.draft_title = task.title.clone();
        self.draft_description = task.description.clone();
        self.draft_project = task.project;
        self.draft_status = task.status;
        self.editing = Some(number);
        self.dialog_open = true;
    }

    // ---------------------------------------------------------------- sidebar

    fn sidebar(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
        ui.spacing_mut().item_spacing.y = 1.0;

        ui.horizontal(|ui| {
            // Ghost rows pad their labels by `space.lg`; the logo shares that
            // left edge, so the whole sidebar reads down one axis.
            ui.add_space(t.space.lg);
            glyphs::logo(ui);
            ui.add_space(2.0);
            ui.label(t.text("Beau Labs", TextRole::BodyStrong));
        });
        ui.add_space(t.space.xl);

        for f in Filter::ALL {
            // Counts say what a click will show: both lists count within
            // the other's selection.
            let count = self
                .tasks
                .iter()
                .filter(|task| f.matches(task) && self.project.is_none_or(|p| task.project == p))
                .count();
            let icon = match f {
                Filter::MyIssues => icons::USER,
                Filter::Active => icons::CIRCLE_DOT,
                Filter::Backlog => icons::CIRCLE_DASHED,
                Filter::Done => icons::CIRCLE_CHECK,
            };
            let mut b = Button::new(f.label())
                .icon(icon)
                .ghost()
                .full_width()
                .selected(self.filter == f);
            if count > 0 {
                b = b.trailing(count.to_string());
            }
            if ui.add(b).clicked() {
                self.filter = f;
            }
        }

        ui.add_space(t.space.xl);
        section_label(ui, "Projects");
        for (i, p) in self.projects.iter().enumerate() {
            let count = self
                .tasks
                .iter()
                .filter(|task| task.project == i && self.filter.matches(task))
                .count();
            let selected = self.project == Some(i);
            let mut b = Button::new(p.name)
                .ghost()
                .full_width()
                .dot(p.color)
                .selected(selected);
            if count > 0 {
                b = b.trailing(count.to_string());
            }
            if ui.add(b).clicked() {
                self.project = (!selected).then_some(i);
            }
        }

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            let mut dark = Self::is_dark(ui.ctx());
            egui::Frame::NONE
                .inner_margin(Margin::symmetric(t.space.lg as i8, t.space.xs as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = t.space.sm;
                        ui.add(icons::MOON.color(t.colors.text_muted));
                        ui.label(
                            t.text("Dark mode", TextRole::Body)
                                .color(t.colors.text_muted),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let switch = Switch::new(&mut dark).accessible_name("Dark mode");
                            if ui.add(switch).changed() {
                                self.set_dark(ui.ctx(), dark);
                            }
                        });
                    });
                });
        });
    }

    // -------------------------------------------------------------- task list

    fn header(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
        let count = self.tasks.iter().filter(|task| self.in_view(task)).count();
        egui::Frame::NONE
            .inner_margin(Margin::symmetric(24, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t.text(self.filter.label(), TextRole::Heading));
                    if let Some(project) = self.project {
                        let name = format!("· {}", self.projects[project].name);
                        ui.label(t.text(name, TextRole::Heading).color(t.colors.text_subtle));
                    }
                    ui.add_space(t.space.xxs);
                    if count > 0 {
                        ui.label(
                            t.text(count.to_string(), TextRole::Body)
                                .color(t.colors.text_subtle),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                Button::new("New task")
                                    .icon(icons::PLUS)
                                    .primary()
                                    .trailing("N"),
                            )
                            .clicked()
                        {
                            self.open_new_task();
                        }
                    });
                });
            });
        let rect = ui.max_rect();
        let y = ui.cursor().top();
        ui.painter().hline(rect.x_range(), y, t.divider(ui.ctx()));
    }

    fn task_list(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
        let rows = self.rows();
        let empty = rows.is_empty();
        let mut actions = Vec::new();
        let mut selected = None;
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut nav =
                    ListNav::new(ui.ctx(), ui.make_persistent_id("tasks")).idle_keys(true);
                let mut cx = RowCx {
                    nav: &mut nav,
                    projects: &self.projects,
                    focus: self.focus_task.take(),
                    actions: &mut actions,
                    selected: &mut selected,
                };
                // One list per view: switching the filter swaps the rows
                // without animating the difference.
                PresenceList::new(("tasks", self.filter, self.project)).show(
                    ui,
                    rows,
                    Row::key,
                    |ui, row, presence| match row {
                        Row::Header(status, count) => group_header(ui, *status, *count),
                        Row::Task(task) => task_row(ui, task, presence.is_interactive(), &mut cx),
                    },
                );
                nav.finish(ui);
                if empty {
                    self.empty_state(ui);
                }
                ui.add_space(t.space.xxl);
            });
        self.selected = selected;
        let now = ui.input(|i| i.time);
        for action in actions {
            match action {
                Action::Toggle(number) => self.toggle_done(number, now),
                Action::Delete(number) => self.delete_task(number),
                Action::Edit(number) => self.open_edit_task(number),
            }
        }
    }

    /// Shown when the current filter matches nothing.
    fn empty_state(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
        let width = 360.0_f32.min(ui.available_width() - 2.0 * t.space.xxl);
        let title = match self.project {
            Some(p) => self.filter.empty_title_in(self.projects[p].name),
            None => self.filter.empty_title().to_owned(),
        };
        // Fades in, so it doesn't pop while the last row is still leaving.
        let id = ui.make_persistent_id(("empty", self.filter, self.project));
        let shown = beau::motion::value_from(ui.ctx(), id, 0.0, 1.0, t.motion.slow);
        ui.multiply_opacity(shown);
        ui.add_space(t.space.xxxl * 2.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(width);
            Card::new()
                .padding(Margin::same(t.space.xxl as i8))
                .show(ui, |ui| {
                    ui.set_width(width - 2.0 * t.space.xxl);
                    ui.spacing_mut().item_spacing.y = t.space.xs;
                    ui.vertical(|ui| {
                        ui.label(t.text(title, TextRole::Title));
                        ui.label(
                            t.text(self.filter.empty_hint(), TextRole::Body)
                                .color(t.colors.text_muted),
                        );
                        ui.add_space(t.space.lg);
                        if ui.add(Button::new("New task").trailing("N")).clicked() {
                            self.open_new_task();
                        }
                    });
                });
        });
    }

    // ----------------------------------------------------------------- dialog

    /// The dialog that creates a task, or edits `self.editing`.
    fn task_dialog(&mut self, ctx: &egui::Context) {
        let mut submit = false;
        let mut cancel = false;
        let mut open = self.dialog_open;
        let editing = self
            .editing
            .and_then(|n| self.tasks.iter().find(|t| t.number == n));
        let (title, action) = match editing {
            Some(_) => ("Edit task", "Save changes"),
            None => ("New task", "Create task"),
        };
        let is_new = editing.is_none();
        let mut dialog = Dialog::new("task").title(title);
        if let Some(task) = editing {
            dialog = dialog.description(format!("BEA-{}", task.number));
        }
        dialog.open(&mut open).show(ctx, |ui| {
            let t = beau::theme(ui);
            let title = ui.add(
                TextInput::new(&mut self.draft_title)
                    .label("Title")
                    .placeholder("What needs doing?")
                    .autofocus(),
            );
            if std::mem::take(&mut self.refocus_title) {
                title.request_focus();
            }
            if title.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                submit = true;
            }
            ui.add_space(t.space.lg);
            ui.add(
                TextInput::new(&mut self.draft_description)
                    .label("Description")
                    .placeholder("Add some detail…")
                    .multiline(3),
            );
            ui.add_space(t.space.lg);
            ui.columns(2, |cols| {
                let mut project = Select::new("project", &mut self.draft_project).label("Project");
                for (i, p) in self.projects.iter().enumerate() {
                    project = project.option_dot(i, p.name, p.color);
                }
                cols[0].add(project);
                let mut status = Select::new("status", &mut self.draft_status).label("Status");
                for s in [
                    Status::Backlog,
                    Status::Todo,
                    Status::InProgress,
                    Status::Done,
                ] {
                    status = status.option_icon(s, s.label(), status_icon(s));
                }
                cols[1].add(status);
            });
            if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Enter)) {
                submit = true;
            }
            Dialog::footer(ui, |ui| {
                let can_submit = !self.draft_title.trim().is_empty();
                let b = Button::new(action).primary().trailing("⌘↵");
                if ui.add_enabled(can_submit, b).clicked() {
                    submit = true;
                }
                if ui.add(Button::new("Cancel").ghost()).clicked() {
                    cancel = true;
                }
                if is_new {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(Checkbox::new(&mut self.create_another).label("Create another"));
                    });
                }
            });
        });
        self.dialog_open = open && !cancel;
        if submit && self.dialog_open {
            self.submit_task();
        }
    }

    fn submit_task(&mut self) {
        let title = self.draft_title.trim().to_owned();
        if title.is_empty() {
            return;
        }
        let description = std::mem::take(&mut self.draft_description);
        self.draft_title.clear();
        if let Some(number) = self.editing {
            if let Some(task) = self.tasks.iter_mut().find(|t| t.number == number) {
                task.title = title;
                task.description = description;
                task.project = self.draft_project;
                task.status = self.draft_status;
            }
            self.dialog_open = false;
            return;
        }
        let number = self.tasks.iter().map(|t| t.number).max().unwrap_or(0) + 1;
        self.tasks.push(Task {
            number,
            title,
            description,
            status: self.draft_status,
            priority: data::Priority::None,
            project: self.draft_project,
            due: None,
            assignee: Some("IH"),
        });
        // Show the new task if the current view would hide it.
        let task = self.tasks.last().expect("just pushed");
        if !self.filter.matches(task) {
            self.filter = match task.status {
                Status::Backlog => Filter::Backlog,
                Status::Done => Filter::Done,
                Status::Todo | Status::InProgress => Filter::Active,
            };
        }
        if self.project.is_some_and(|p| p != task.project) {
            self.project = None;
        }
        self.dialog_open = self.create_another;
        self.refocus_title = self.create_another;
    }
}

/// The icon for a status in menus and selects.
fn status_icon(status: Status) -> beau::Icon {
    match status {
        Status::Backlog => icons::CIRCLE_DASHED,
        Status::Todo => icons::CIRCLE,
        Status::InProgress => icons::CIRCLE_DOT,
        Status::Done => icons::CIRCLE_CHECK,
    }
}

/// How long a row stays put after its status changes.
const LINGER_SECONDS: f64 = 0.35;

/// A task shown in the group it just left.
struct Linger {
    number: u32,
    group: Status,
    until: f64,
}

/// One row of the task list.
#[derive(Clone)]
enum Row {
    Header(Status, usize),
    Task(Task),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum RowKey {
    Header(Status),
    Task(u32),
}

impl Row {
    fn key(&self) -> RowKey {
        match self {
            Self::Header(status, _) => RowKey::Header(*status),
            Self::Task(task) => RowKey::Task(task.number),
        }
    }
}

/// Something a row asked for; applied after the list is drawn.
enum Action {
    Toggle(u32),
    Delete(u32),
    Edit(u32),
}

/// What every task row shares.
struct RowCx<'a> {
    nav: &'a mut ListNav,
    projects: &'a [Project],
    /// The task to give keyboard focus.
    focus: Option<u32>,
    actions: &'a mut Vec<Action>,
    /// Set to the task whose row has focus.
    selected: &'a mut Option<u32>,
}

fn section_label(ui: &mut Ui, text: &str) {
    let t = beau::theme(ui);
    egui::Frame::NONE
        .inner_margin(Margin::symmetric(t.space.lg as i8, t.space.xs as i8))
        .show(ui, |ui| {
            ui.label(t.text(text, TextRole::Caption).color(t.colors.text_subtle));
        });
}

fn group_header(ui: &mut Ui, status: Status, count: usize) {
    let t = beau::theme(ui);
    let c = &t.colors;
    // Plain label: structure from rhythm (space above), not from bands or lines.
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::hover());
    let p = ui.painter();
    let cy = rect.bottom() - 16.0;
    // No status glyph: each row already shows one, and a second copy in
    // another column read as misaligned.
    let x = rect.left() + 24.0;
    let label = p.layout_no_wrap(status.label().into(), t.font(TextRole::BodyStrong), c.text);
    let w = label.size().x;
    p.galley(egui::pos2(x, cy - label.size().y / 2.0), label, c.text);
    let n = p.layout_no_wrap(count.to_string(), t.font(TextRole::Body), c.text_subtle);
    p.galley(
        egui::pos2(x + w + 8.0, cy - n.size().y / 2.0),
        n,
        c.text_subtle,
    );
}

/// A task row. `live` is false while the row animates out.
fn task_row(ui: &mut Ui, task: &Task, live: bool, cx: &mut RowCx<'_>) {
    let t = beau::theme(ui);
    let c = &t.colors;
    let number = task.number;
    let project = &cx.projects[task.project];
    let sense = if live {
        cx.nav.sense(number)
    } else {
        Sense::hover()
    };
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), t.controls.row + 4.0), sense);
    let mut focused = false;
    if live {
        response.widget_info(|| WidgetInfo::labeled(egui::Role::Button, true, &task.title));
        if cx.focus == Some(number) {
            response.request_focus();
        }
        focused = cx.nav.item(ui, number, &response);
    }
    if focused {
        *cx.selected = Some(number);
        let key = |key| ui.input_mut(|i| i.consume_key(Modifiers::NONE, key));
        if key(Key::Enter) {
            cx.actions.push(Action::Edit(number));
        } else if key(Key::Space) {
            cx.actions.push(Action::Toggle(number));
        } else if key(Key::Delete) || key(Key::Backspace) {
            cx.actions.push(Action::Delete(number));
        }
    }
    if response.double_clicked() {
        cx.actions.push(Action::Edit(number));
    }

    // The pointer is over the row, even while it is on one of its controls.
    let inside = live && ui.rect_contains_pointer(rect);
    let ctx = ui.ctx().clone();
    let hover = beau::motion::toggle(&ctx, response.id.with("hover"), inside, t.motion.fast);
    let select = beau::motion::toggle(&ctx, response.id.with("focus"), focused, t.motion.fast);
    let actions = beau::motion::toggle(
        &ctx,
        response.id.with("actions"),
        inside || focused,
        t.motion.fast,
    );

    let p = ui.painter().clone();
    if hover > 0.0 {
        p.rect_filled(rect, 0.0, c.surface_hover.gamma_multiply(hover));
    }
    if select > 0.0 {
        p.rect_filled(rect, 0.0, c.surface_active.gamma_multiply(select));
    }
    let cy = rect.center().y;
    let small = t.font(TextRole::Small);
    let mut x = rect.left() + 24.0;
    glyphs::priority(&p, egui::pos2(x + 7.0, cy), task.priority, &t);
    x += 26.0;
    let id = p.layout_no_wrap(format!("BEA-{number}"), small.clone(), c.text_subtle);
    p.galley(egui::pos2(x, cy - id.size().y / 2.0), id, c.text_subtle);
    x += 60.0;

    // The status mark is the row's checkbox: click it to complete the task.
    let mark = egui::pos2(x + 7.0, cy);
    if live {
        let hit = egui::Rect::from_center_size(mark, vec2(22.0, 22.0));
        let done = task.status == Status::Done;
        let toggle = ui.interact(hit, response.id.with("status"), Sense::CLICK);
        toggle.widget_info(|| WidgetInfo::selected(egui::Role::CheckBox, true, done, "Done"));
        let over = beau::motion::toggle(
            &ctx,
            toggle.id.with("hover"),
            toggle.hovered(),
            t.motion.fast,
        );
        if over > 0.0 {
            // A tonal wash that shows on the hover and selected fills alike.
            p.circle_filled(mark, 11.0, c.text.gamma_multiply(0.08 * over));
        }
        if toggle
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            cx.actions.push(Action::Toggle(number));
        }
    }
    glyphs::status(&p, mark, task.status, &t);
    x += 26.0;

    // Right side, laid out right-to-left.
    let mut right = rect.right() - 24.0;
    glyphs::avatar(&p, egui::pos2(right - 11.0, cy), task.assignee, &t);
    right -= 22.0 + 16.0;
    // Fixed-width columns so dates and projects align down the list.
    let due_w = 52.0;
    if let Some(due) = task.due {
        let g = p.layout_no_wrap(due.into(), small.clone(), c.text_muted);
        p.galley(
            egui::pos2(right - g.size().x, cy - g.size().y / 2.0),
            g,
            c.text_muted,
        );
    }
    right -= due_w + 24.0;
    let project_w = 96.0;
    let px = right - project_w;
    p.circle_filled(egui::pos2(px + 3.0, cy), 3.0, project.color);
    let name = p.layout_no_wrap(project.name.into(), small, c.text_muted);
    p.galley(
        egui::pos2(px + 14.0, cy - name.size().y / 2.0),
        name,
        c.text_muted,
    );
    right = px - 24.0;

    // Row actions sit at the end of the title, so the columns never move.
    if live && actions > 0.0 {
        let slot = egui::Rect::from_center_size(
            egui::pos2(right - t.controls.sm / 2.0, cy),
            egui::Vec2::splat(t.controls.sm),
        );
        let mut child = ui.new_child(UiBuilder::new().max_rect(slot));
        child.multiply_opacity(actions);
        let delete = Button::icon_only(icons::TRASH_2, "Delete task")
            .ghost()
            .size(Size::Sm);
        let delete = child.add(delete);
        Tooltip::new("Delete task")
            .shortcut("Del")
            .show(&child, &delete);
        if delete.clicked() {
            cx.actions.push(Action::Delete(number));
        }
    }
    right -= (t.controls.sm + t.space.md) * actions;

    let title_color = if task.status == Status::Done {
        c.text_muted
    } else {
        c.text
    };
    let title = p.layout(
        task.title.clone(),
        t.font(TextRole::Body),
        title_color,
        f32::INFINITY,
    );
    let clip = egui::Rect::from_x_y_ranges(x..=right, rect.y_range());
    p.with_clip_rect(clip)
        .galley(egui::pos2(x, cy - title.size().y / 2.0), title, title_color);
}

impl eframe::App for TrackerApp {
    /// For `--stock` screenshots, park the pointer over the activity list so
    /// its (floating) scroll bar shows.
    #[cfg(not(target_arch = "wasm32"))]
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if self.screenshot.is_none() {
            return;
        }
        if let Some(pos) = self.stock.as_ref().and_then(|p| p.hover_target) {
            raw_input.events.push(egui::Event::PointerMoved(pos));
        }
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shortcuts(ctx);
        self.expire_lingering(ctx);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(job) = &mut self.screenshot {
            job.tick(ctx);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if let Some(page) = &mut self.stock {
            page.ui(ui);
            return;
        }
        let t = beau::theme(ui);
        egui::Panel::left("sidebar")
            .exact_size(240.0)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(t.colors.bg).inner_margin(Margin {
                left: 10,
                right: 10,
                top: 14,
                bottom: 10,
            }))
            .show(ui, |ui| self.sidebar(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(t.colors.surface))
            .show(ui, |ui| {
                self.header(ui);
                self.task_list(ui);
            });
        let ctx = ui.ctx().clone();
        self.task_dialog(&ctx);
    }
}
