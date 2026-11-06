use eframe::egui::{self, Ui};

use crate::Options;
use crate::data::{self, Filter, Project, Status, Task};

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

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, opts: Options) -> Self {
        if !opts.stock {
            beau::install_with(&cc.egui_ctx, beau::Theme::for_mode(mode(opts.dark)));
        } else {
            cc.egui_ctx.set_theme(if opts.dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
        }
        Self {
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
        }
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

    fn sidebar(&mut self, ui: &mut Ui) {
        ui.heading("Beau Labs");
        ui.separator();
        for f in Filter::ALL {
            if ui.selectable_label(self.filter == f, f.label()).clicked() {
                self.filter = f;
            }
        }
        ui.separator();
        ui.label("Projects");
        for (i, p) in self.projects.iter().enumerate() {
            let count = self.tasks.iter().filter(|t| t.project == i).count();
            ui.horizontal(|ui| {
                ui.colored_label(p.color, "●");
                ui.label(p.name);
                ui.label(count.to_string());
            });
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let label = if self.dark { "Light mode" } else { "Dark mode" };
            if ui.button(label).clicked() {
                let dark = !self.dark;
                self.set_dark(ui.ctx(), dark);
            }
        });
    }

    fn task_list(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(self.filter.label());
            ui.label(self.tasks.len().to_string());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("New task").clicked() {
                    self.new_task_open = true;
                }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for status in Status::ALL {
                let tasks: Vec<&Task> = self.tasks.iter().filter(|t| t.status == status).collect();
                if tasks.is_empty() {
                    continue;
                }
                ui.strong(format!("{}  {}", status.label(), tasks.len()));
                for t in tasks {
                    ui.horizontal(|ui| {
                        ui.label(format!("BEA-{}", t.number));
                        ui.label(&t.title);
                        if let Some(due) = t.due {
                            ui.label(due);
                        }
                    });
                }
            }
        });
    }

    fn new_task_dialog(&mut self, ctx: &egui::Context) {
        if !self.new_task_open {
            return;
        }
        let mut open = true;
        egui::Window::new("New task")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Title");
                ui.text_edit_singleline(&mut self.draft_title);
                ui.label("Description");
                ui.text_edit_multiline(&mut self.draft_description);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.new_task_open = false;
                    }
                    if ui.button("Create").clicked() {
                        self.create_task();
                    }
                });
            });
        if !open {
            self.new_task_open = false;
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
            assignee: None,
        });
        self.draft_title.clear();
        self.new_task_open = false;
    }
}

fn mode(dark: bool) -> beau::Mode {
    if dark {
        beau::Mode::Dark
    } else {
        beau::Mode::Light
    }
}

impl eframe::App for TrackerApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(job) = &mut self.screenshot {
            job.tick(_ctx);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("sidebar")
            .exact_size(232.0)
            .show(ui, |ui| self.sidebar(ui));
        egui::CentralPanel::default().show(ui, |ui| self.task_list(ui));
        let ctx = ui.ctx().clone();
        self.new_task_dialog(&ctx);
    }
}
