//! `--stock`: a settings page built from stock egui widgets, with a few beau
//! components where beau has them (cards, the primary button, the text
//! field).
//!
//! With `--no-beau` the same page renders without `beau::install`, and the
//! beau components fall back to their nearest stock egui equivalent, for the
//! README's before/after comparison.

use eframe::egui::{self, Align, Layout, Margin, Popup, Response, RichText, Ui};

use beau::tokens::TextRole;
use beau::{Button, Card, TextInput};

const VIEWS: [&str; 3] = ["List", "Board", "Timeline"];
const PRIORITIES: [&str; 4] = ["Urgent", "High", "Medium", "Low"];
const ACTIVITY: [(&str, &str); 14] = [
    ("Moved BEA-12 to In progress", "2m"),
    ("Commented on BEA-9", "18m"),
    ("Created BEA-21", "1h"),
    ("Closed BEA-4", "2h"),
    ("Renamed the project", "3h"),
    ("Assigned BEA-17", "5h"),
    ("Moved BEA-3 to Done", "6h"),
    ("Set a due date on BEA-14", "1d"),
    ("Created BEA-20", "1d"),
    ("Commented on BEA-12", "2d"),
    ("Raised BEA-8 to High", "2d"),
    ("Closed BEA-6", "3d"),
    ("Created BEA-19", "4d"),
    ("Archived Website", "5d"),
];

/// Width of the settings column.
const MAIN_W: f32 = 520.0;
/// Width of the activity column.
const SIDE_W: f32 = 272.0;
/// Space between the two columns, and between sections.
const GAP: f32 = 24.0;
/// Inner padding of a card.
const PAD: f32 = 16.0;
/// Width of the controls column inside a card.
const CONTROL_W: f32 = 224.0;

pub struct StockPage {
    /// Whether beau is installed (false with `--no-beau`).
    beau: bool,
    /// Hold the tooltip and menu open, so screenshots show them.
    demo: bool,
    name: String,
    description: String,
    view: usize,
    priority: usize,
    estimate: u32,
    digest: bool,
    completed: bool,
    lead_minutes: f32,
    saved: bool,
    /// Where `--screenshot` parks the pointer: over the activity list, so its
    /// scroll bar shows.
    pub hover_target: Option<egui::Pos2>,
}

impl StockPage {
    pub fn new(beau: bool, demo: bool) -> Self {
        Self {
            beau,
            demo,
            name: "Beau Core".to_owned(),
            description: String::new(),
            view: 0,
            priority: 2,
            estimate: 3,
            digest: true,
            completed: false,
            lead_minutes: 15.0,
            saved: false,
            hover_target: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let width = MAIN_W + GAP + SIDE_W;
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(Margin::same(32)))
            .show(ui, |ui| {
                let side = ((ui.available_width() - width) / 2.0).max(0.0);
                ui.horizontal_top(|ui| {
                    ui.add_space(side);
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        self.header(ui);
                        ui.add_space(GAP);
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(MAIN_W);
                                self.settings(ui);
                            });
                            ui.add_space(GAP - ui.spacing().item_spacing.x);
                            ui.vertical(|ui| {
                                ui.set_width(SIDE_W);
                                self.activity(ui);
                            });
                        });
                    });
                });
            });
    }

    // ------------------------------------------------------------ sections

    fn header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if self.beau {
                    let t = beau::theme(ui);
                    ui.label(t.text("Project settings", TextRole::Heading));
                    ui.label(
                        t.text("Stock egui widgets, styled by beau", TextRole::Body)
                            .color(t.colors.text_muted),
                    );
                } else {
                    ui.heading("Project settings");
                    ui.weak("Stock egui widgets");
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let menu = ui.menu_button("Actions", |ui| {
                    let _ = ui.button("Duplicate project");
                    let _ = ui.button("Export as CSV…");
                    let _ = ui.button("Copy link");
                    ui.separator();
                    let _ = ui.button("Archive");
                });
                if self.demo && ui.ctx().cumulative_pass_nr() == 1 {
                    Popup::open_id(ui.ctx(), Popup::default_response_id(&menu.response));
                }
            });
        });
    }

    fn settings(&mut self, ui: &mut Ui) {
        let beau = self.beau;
        section(beau, ui, "General");
        card(beau, ui, |ui| {
            row(
                beau,
                ui,
                "Name",
                "Shown in the sidebar and in links.",
                |ui| ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(CONTROL_W)),
            );
            ui.separator();
            row(
                beau,
                ui,
                "Default view",
                "How tasks open for new members.",
                |ui| {
                    let mut r = ui.radio_value(&mut self.view, 0, VIEWS[0]);
                    for (i, view) in VIEWS.iter().enumerate().skip(1) {
                        r |= ui.radio_value(&mut self.view, i, *view);
                    }
                    r
                },
            );
            ui.separator();
            row(beau, ui, "Priority", "Applied to new tasks.", |ui| {
                let combo = egui::ComboBox::from_id_salt("priority")
                    .selected_text(PRIORITIES[self.priority])
                    .width(CONTROL_W);
                let combo = if self.beau {
                    combo.icon(beau::paint::select_chevrons)
                } else {
                    combo
                };
                combo
                    .show_ui(ui, |ui| {
                        for (i, p) in PRIORITIES.iter().enumerate() {
                            ui.selectable_value(&mut self.priority, i, *p);
                        }
                    })
                    .response
            });
            ui.separator();
            row(
                beau,
                ui,
                "Estimate",
                "Default points for a new task.",
                |ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.estimate)
                            .range(0..=13)
                            .suffix(" pts"),
                    )
                },
            );
            ui.separator();
            ui.add_space(4.0);
            let label = title(beau, ui, "Description");
            if self.beau {
                ui.add(
                    TextInput::new(&mut self.description)
                        .placeholder("What is this project for?")
                        .multiline(2),
                );
            } else {
                ui.add(
                    egui::TextEdit::multiline(&mut self.description)
                        .hint_text("What is this project for?")
                        .desired_rows(2)
                        .desired_width(f32::INFINITY),
                )
                .labelled_by(label.id);
            }
        });

        ui.add_space(GAP);
        section(beau, ui, "Notifications");
        card(beau, ui, |ui| {
            ui.checkbox(&mut self.digest, "Send a daily digest");
            ui.checkbox(&mut self.completed, "Include completed tasks");
            ui.separator();
            row(beau, ui, "Reminder", "Before a task is due.", |ui| {
                // Leave room for the value field next to the rail.
                ui.spacing_mut().slider_width = CONTROL_W - 100.0;
                ui.add(
                    egui::Slider::new(&mut self.lead_minutes, 0.0..=60.0)
                        .step_by(5.0)
                        .fixed_decimals(0)
                        .suffix(" min"),
                )
            });
        });

        ui.add_space(GAP);
        self.footer(ui);
    }

    fn activity(&mut self, ui: &mut Ui) {
        let beau = self.beau;
        section(beau, ui, "Activity");
        let target = card(beau, ui, |ui| {
            let area = egui::ScrollArea::vertical()
                .max_height(400.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    // Room for the floating scroll bar beside the times.
                    ui.set_max_width(ui.available_width() - 2.0 * ui.spacing().scroll.bar_width);
                    for (line, when) in ACTIVITY {
                        ui.horizontal(|ui| {
                            ui.label(line);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                hint(beau, ui, when);
                            });
                        });
                    }
                });
            area.inner_rect.center()
        });
        self.hover_target = Some(target);
    }

    fn footer(&mut self, ui: &mut Ui) {
        let beau = self.beau;
        let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
            let save = if self.beau {
                ui.add(Button::new("Save changes").primary())
            } else {
                ui.button("Save changes")
            };
            if save.clicked() {
                self.saved = true;
            }
            let reset = ui.button("Reset");
            if reset.clicked() {
                *self = Self {
                    hover_target: self.hover_target,
                    ..Self::new(self.beau, false)
                };
            }
            let tip = "Restore this project's defaults";
            if self.demo {
                egui::Tooltip::for_widget(&reset).show(|ui| ui.label(tip));
            } else {
                reset.on_hover_text(tip);
            }
            if self.saved {
                hint(beau, ui, "Saved");
            }
        });
    }
}

// ---------------------------------------------------------------- helpers

fn title(beau: bool, ui: &mut Ui, text: &str) -> Response {
    if beau {
        let t = beau::theme(ui);
        ui.label(t.text(text, TextRole::BodyStrong))
    } else {
        ui.label(RichText::new(text).strong())
    }
}

fn hint(beau: bool, ui: &mut Ui, text: &str) -> Response {
    if beau {
        let t = beau::theme(ui);
        ui.label(t.text(text, TextRole::Small).color(t.colors.text_muted))
    } else {
        ui.weak(text)
    }
}

/// A plain group label above a card: rhythm, not bands or lines.
fn section(beau: bool, ui: &mut Ui, text: &str) {
    if beau {
        let t = beau::theme(ui);
        ui.label(t.text(text, TextRole::Caption).color(t.colors.text_subtle));
    } else {
        ui.label(RichText::new(text).small().weak());
    }
    ui.add_space(2.0);
}

fn card<R>(beau: bool, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let width = ui.available_width();
    let inner = |ui: &mut Ui| {
        ui.set_width(width - 2.0 * PAD);
        add(ui)
    };
    if beau {
        Card::new()
            .padding(Margin::same(PAD as i8))
            .show(ui, inner)
            .inner
    } else {
        egui::Frame::group(ui.style())
            .inner_margin(Margin::same(PAD as i8))
            .show(ui, inner)
            .inner
    }
}

/// One setting: title and hint on the left, the control on the right.
fn row(beau: bool, ui: &mut Ui, name: &str, help: &str, control: impl FnOnce(&mut Ui) -> Response) {
    ui.horizontal(|ui| {
        let label = ui
            .vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let label = title(beau, ui, name);
                hint(beau, ui, help);
                label
            })
            .inner;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(CONTROL_W, ui.available_height()),
                Layout::left_to_right(Align::Center),
                |ui| control(ui).labelled_by(label.id),
            );
        });
    });
}
