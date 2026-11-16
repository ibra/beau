use eframe::egui::{self, Align, Key, Layout, Margin, Modifiers, Sense, Ui, vec2};

use beau::tokens::TextRole;
use beau::{Button, Card, Checkbox, Dialog, Switch, TextInput, icons};

use crate::Options;
use crate::data::{self, Filter, Project, Status, Task};
use crate::glyphs;

pub struct TrackerApp {
    tasks: Vec<Task>,
    projects: Vec<Project>,
    filter: Filter,
    /// The `--stock` page, shown instead of the tracker.
    stock: Option<crate::stock::StockPage>,
    new_task_open: bool,
    draft_title: String,
    draft_description: String,
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
            stock: None,
            new_task_open: opts.dialog,
            draft_title: String::new(),
            draft_description: String::new(),
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
        let fixed = opts.dark || app.screenshot.is_some();
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
        if ctx.egui_wants_keyboard_input() || self.new_task_open {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::N)) {
            self.new_task_open = true;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::D)) {
            self.set_dark(ctx, !Self::is_dark(ctx));
        }
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
            let count = self.tasks.iter().filter(|task| f.matches(task)).count();
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
            let count = self.tasks.iter().filter(|task| task.project == i).count();
            let mut b = Button::new(p.name).ghost().full_width().dot(p.color);
            if count > 0 {
                b = b.trailing(count.to_string());
            }
            ui.add(b);
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
        let count = self
            .tasks
            .iter()
            .filter(|task| self.filter.matches(task))
            .count();
        egui::Frame::NONE
            .inner_margin(Margin::symmetric(24, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t.text(self.filter.label(), TextRole::Heading));
                    ui.add_space(t.space.xxs);
                    ui.label(
                        t.text(count.to_string(), TextRole::Body)
                            .color(t.colors.text_subtle),
                    );
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
                            self.new_task_open = true;
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
        if !self.tasks.iter().any(|task| self.filter.matches(task)) {
            self.empty_state(ui);
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for status in Status::ALL {
                    let tasks: Vec<&Task> = self
                        .tasks
                        .iter()
                        .filter(|task| task.status == status && self.filter.matches(task))
                        .collect();
                    if tasks.is_empty() {
                        continue;
                    }
                    group_header(ui, status, tasks.len());
                    for task in tasks {
                        task_row(ui, task, &self.projects[task.project]);
                    }
                }
                ui.add_space(t.space.xxl);
            });
    }

    /// Shown when the current filter matches nothing.
    fn empty_state(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
        let width = 360.0_f32.min(ui.available_width() - 2.0 * t.space.xxl);
        ui.add_space(t.space.xxxl * 2.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(width);
            Card::new()
                .padding(Margin::same(t.space.xxl as i8))
                .show(ui, |ui| {
                    ui.set_width(width - 2.0 * t.space.xxl);
                    ui.spacing_mut().item_spacing.y = t.space.xs;
                    ui.vertical(|ui| {
                        ui.label(t.text(self.filter.empty_title(), TextRole::Title));
                        ui.label(
                            t.text(self.filter.empty_hint(), TextRole::Body)
                                .color(t.colors.text_muted),
                        );
                        ui.add_space(t.space.lg);
                        if ui.add(Button::new("New task").trailing("N")).clicked() {
                            self.new_task_open = true;
                        }
                    });
                });
        });
    }

    // ----------------------------------------------------------------- dialog

    fn new_task_dialog(&mut self, ctx: &egui::Context) {
        let mut create = false;
        let mut cancel = false;
        let mut open = self.new_task_open;
        Dialog::new("new-task")
            .title("New task")
            .description("Add a task to Beau Core.")
            .open(&mut open)
            .show(ctx, |ui| {
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
                    create = true;
                }
                ui.add_space(t.space.lg);
                ui.add(
                    TextInput::new(&mut self.draft_description)
                        .label("Description")
                        .placeholder("Add some detail…")
                        .multiline(3),
                );
                if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Enter)) {
                    create = true;
                }
                Dialog::footer(ui, |ui| {
                    let can_create = !self.draft_title.trim().is_empty();
                    let b = Button::new("Create task").primary().trailing("⌘↵");
                    if ui.add_enabled(can_create, b).clicked() {
                        create = true;
                    }
                    if ui.add(Button::new("Cancel").ghost()).clicked() {
                        cancel = true;
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(Checkbox::new(&mut self.create_another).label("Create another"));
                    });
                });
            });
        self.new_task_open = open && !cancel;
        if create {
            self.create_task();
        }
    }

    fn create_task(&mut self) {
        let title = self.draft_title.trim();
        if title.is_empty() {
            return;
        }
        let number = self.tasks.iter().map(|t| t.number).max().unwrap_or(0) + 1;
        self.tasks.push(Task {
            number,
            title: title.to_owned(),
            description: std::mem::take(&mut self.draft_description),
            status: Status::Todo,
            priority: data::Priority::None,
            project: 0,
            due: None,
            assignee: Some("IH"),
        });
        self.draft_title.clear();
        self.new_task_open = self.create_another;
        self.refocus_title = self.create_another;
    }
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

fn task_row(ui: &mut Ui, task: &Task, project: &Project) {
    let t = beau::theme(ui);
    let c = &t.colors;
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), t.controls.row + 4.0),
        Sense::click(),
    );
    let hover = beau::motion::toggle(
        ui.ctx(),
        response.id.with("hover"),
        response.hovered(),
        t.motion.fast,
    );
    let p = ui.painter();
    if hover > 0.0 {
        p.rect_filled(rect, 0.0, c.surface_hover.gamma_multiply(hover));
    }
    let cy = rect.center().y;
    let small = t.font(TextRole::Small);
    let mut x = rect.left() + 24.0;
    glyphs::priority(p, egui::pos2(x + 7.0, cy), task.priority, &t);
    x += 26.0;
    let id = p.layout_no_wrap(format!("BEA-{}", task.number), small.clone(), c.text_subtle);
    p.galley(egui::pos2(x, cy - id.size().y / 2.0), id, c.text_subtle);
    x += 60.0;
    glyphs::status(p, egui::pos2(x + 7.0, cy), task.status, &t);
    x += 26.0;

    // Right side, laid out right-to-left.
    let mut right = rect.right() - 24.0;
    glyphs::avatar(p, egui::pos2(right - 11.0, cy), task.assignee, &t);
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
        self.new_task_dialog(&ctx);
    }
}
