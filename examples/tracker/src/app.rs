use eframe::egui::{self, Align, Key, Layout, Margin, Modifiers, Sense, Ui, vec2};

use beau::tokens::TextRole;
use beau::{Button, Dialog, Size, TextInput};

use crate::Options;
use crate::data::{self, Filter, Project, Status, Task};
use crate::glyphs;

pub struct TrackerApp {
    tasks: Vec<Task>,
    projects: Vec<Project>,
    filter: Filter,
    dark: bool,
    stock: bool,
    new_task_open: bool,
    draft_title: String,
    draft_description: String,
    #[cfg(not(target_arch = "wasm32"))]
    screenshot: Option<crate::screenshot::ScreenshotJob>,
}

fn mode(dark: bool) -> beau::Mode {
    if dark {
        beau::Mode::Dark
    } else {
        beau::Mode::Light
    }
}

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: Options) -> Self {
        let mut app = Self {
            tasks: data::seed_tasks(),
            projects: data::projects(),
            filter: Filter::Active,
            dark: opts.dark,
            stock: opts.stock,
            new_task_open: opts.dialog,
            draft_title: String::new(),
            draft_description: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            screenshot: opts.screenshot.map(crate::screenshot::ScreenshotJob::new),
        };
        if !app.stock {
            beau::typography::install_fonts(&cc.egui_ctx);
        }
        app.set_dark(&cc.egui_ctx, opts.dark);
        app
    }

    fn set_dark(&mut self, ctx: &egui::Context, dark: bool) {
        self.dark = dark;
        if self.stock {
            ctx.set_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
        } else {
            beau::set_theme(ctx, beau::Theme::for_mode(mode(dark)));
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() || self.new_task_open {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::N)) {
            self.new_task_open = true;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::D)) {
            self.set_dark(ctx, !self.dark);
        }
    }

    // ---------------------------------------------------------------- sidebar

    fn sidebar(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
        ui.spacing_mut().item_spacing.y = 1.0;

        ui.horizontal(|ui| {
            glyphs::logo(ui);
            ui.add_space(2.0);
            ui.label(t.text("Beau Labs", TextRole::BodyStrong));
        });
        ui.add_space(t.space.xl);

        for f in Filter::ALL {
            let count = self.tasks.iter().filter(|task| f.matches(task)).count();
            let b = Button::new(f.label())
                .ghost()
                .full_width()
                .selected(self.filter == f)
                .trailing(count.to_string());
            if ui.add(b).clicked() {
                self.filter = f;
            }
        }

        ui.add_space(t.space.xl);
        section_label(ui, "Projects");
        for (i, p) in self.projects.iter().enumerate() {
            let count = self.tasks.iter().filter(|task| task.project == i).count();
            ui.add(
                Button::new(p.name)
                    .ghost()
                    .full_width()
                    .dot(p.color)
                    .trailing(count.to_string()),
            );
        }

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            let label = if self.dark { "Light mode" } else { "Dark mode" };
            if ui
                .add(
                    Button::new(label)
                        .ghost()
                        .size(Size::Sm)
                        .full_width()
                        .trailing("⇧D"),
                )
                .clicked()
            {
                let dark = !self.dark;
                self.set_dark(ui.ctx(), dark);
            }
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
                            .add(Button::new("New task").primary().trailing("N"))
                            .clicked()
                        {
                            self.new_task_open = true;
                        }
                        ui.add(Button::new("Display").secondary());
                    });
                });
            });
        let rect = ui.max_rect();
        let y = ui.cursor().top();
        ui.painter().hline(rect.x_range(), y, t.hairline());
    }

    fn task_list(&mut self, ui: &mut Ui) {
        let t = beau::theme(ui);
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
        self.new_task_open = false;
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
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 0.0, c.bg.gamma_multiply(0.6));
    p.hline(rect.x_range(), rect.bottom() - 0.5, t.hairline());
    let mut x = rect.left() + 24.0;
    glyphs::status(p, egui::pos2(x + 7.0, rect.center().y), status, &t);
    x += 24.0;
    let label = p.layout_no_wrap(status.label().into(), t.font(TextRole::BodyStrong), c.text);
    let w = label.size().x;
    p.galley(
        egui::pos2(x, rect.center().y - label.size().y / 2.0),
        label,
        c.text,
    );
    let n = p.layout_no_wrap(count.to_string(), t.font(TextRole::Body), c.text_subtle);
    p.galley(
        egui::pos2(x + w + 8.0, rect.center().y - n.size().y / 2.0),
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
    p.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        (1.0, c.border.gamma_multiply(0.6)),
    );

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
    glyphs::avatar(p, egui::pos2(right - 10.0, cy), task.assignee, &t);
    right -= 20.0 + 16.0;
    if let Some(due) = task.due {
        let g = p.layout_no_wrap(due.into(), small.clone(), c.text_muted);
        right -= g.size().x;
        p.galley(egui::pos2(right, cy - g.size().y / 2.0), g, c.text_muted);
        right -= 16.0;
    }
    let chip = p.layout_no_wrap(project.name.into(), small, c.text_muted);
    let chip_w = chip.size().x + 8.0 + 6.0 + 16.0;
    let chip_rect =
        egui::Rect::from_center_size(egui::pos2(right - chip_w / 2.0, cy), vec2(chip_w, 22.0));
    p.add(beau::paint::surface(
        chip_rect,
        egui::CornerRadius::same(t.radius.full),
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(1.0, c.border),
    ));
    p.circle_filled(egui::pos2(chip_rect.left() + 10.0, cy), 3.0, project.color);
    p.galley(
        egui::pos2(chip_rect.left() + 18.0, cy - chip.size().y / 2.0),
        chip,
        c.text_muted,
    );
    right = chip_rect.left() - 16.0;

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
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shortcuts(ctx);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(job) = &mut self.screenshot {
            job.tick(ctx);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.stock {
            self.stock_ui(ui);
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

impl TrackerApp {
    /// The same screen with stock egui widgets, for before/after comparisons.
    fn stock_ui(&mut self, ui: &mut Ui) {
        egui::Panel::left("sidebar")
            .exact_size(240.0)
            .show(ui, |ui| {
                ui.heading("Beau Labs");
                for f in Filter::ALL {
                    if ui.selectable_label(self.filter == f, f.label()).clicked() {
                        self.filter = f;
                    }
                }
                ui.separator();
                for p in &self.projects {
                    ui.horizontal(|ui| {
                        ui.colored_label(p.color, "●");
                        ui.label(p.name);
                    });
                }
            });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.filter.label());
                if ui.button("New task").clicked() {
                    self.new_task_open = true;
                }
            });
            ui.separator();
            for task in self.tasks.iter().filter(|task| self.filter.matches(task)) {
                ui.horizontal(|ui| {
                    ui.label(format!("BEA-{}", task.number));
                    ui.label(&task.title);
                });
            }
        });
        let mut open = self.new_task_open;
        egui::Window::new("New task")
            .open(&mut open)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label("Title");
                ui.text_edit_singleline(&mut self.draft_title);
                ui.label("Description");
                ui.text_edit_multiline(&mut self.draft_description);
                ui.horizontal(|ui| {
                    let _ = ui.button("Cancel");
                    let _ = ui.button("Create");
                });
            });
        self.new_task_open = open;
    }
}
