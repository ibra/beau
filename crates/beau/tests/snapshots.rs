//! Visual snapshot tests: every component, in light and dark mode.
//!
//! Each test renders a small scene headlessly with `egui_kittest` (wgpu),
//! lets motion settle, and compares the result with the PNG committed in
//! `tests/snapshots/`. On a mismatch the test fails and writes
//! `<name>.new.png` and `<name>.diff.png` next to it.
//!
//! Update the committed images after an intended visual change with
//! `UPDATE_SNAPSHOTS=1 cargo test -p beau --test snapshots`, then review the
//! diff in git. See CONTRIBUTING.md › Snapshot tests.

use beau::{Button, Card, Dialog, Size, TextInput, components::Kbd, tokens::TextRole};
use egui::{Color32, Theme, Ui, Vec2, accesskit::Role, vec2};
use egui_kittest::{Harness, SnapshotOptions, SnapshotResults, kittest::Queryable};

/// beau clamps `dt` to 1/30s, so step at that rate for motion to advance.
const STEP_DT: f32 = 1.0 / 30.0;
/// Enough frames at [`STEP_DT`] for the slowest spring (0.28s) to settle.
const MAX_STEPS: u64 = 120;
/// Render at 2x, so physical-pixel hairlines differ from 1pt strokes.
const PIXELS_PER_POINT: f32 = 2.0;
/// Per-pixel tolerance for scenes with large soft shadows (see [`dialog`]).
const DIALOG_THRESHOLD: f32 = 5.0;

/// Build a harness that draws `scene` on the `surface` colour with `xl`
/// padding. `size: None` fits the viewport to the scene.
fn harness<'a>(
    theme: Theme,
    size: Option<Vec2>,
    mut scene: impl FnMut(&mut Ui) + 'a,
) -> Harness<'a> {
    let mut builder = Harness::builder()
        .with_pixels_per_point(PIXELS_PER_POINT)
        .with_theme(theme)
        .with_step_dt(STEP_DT)
        .with_max_steps(MAX_STEPS)
        .with_context_setup(beau::install)
        .wgpu();
    builder = match size {
        Some(size) => builder.with_size(size),
        None => builder.with_fit_contents(),
    };
    builder.build_ui(move |ui| {
        let t = beau::theme(ui);
        // The harness paints no background of its own.
        ui.painter()
            .rect_filled(ui.ctx().content_rect(), 0.0, t.colors.surface);
        egui::Frame::new()
            .inner_margin(t.space.xl)
            .show(ui, |ui| scene(ui));
    })
}

fn suffix(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
    }
}

/// Render `scene` in light and dark mode and snapshot each as `<name>-<mode>`.
/// `interact` runs after the first frame (e.g. to focus a widget).
fn snapshot_both(
    name: &str,
    size: Option<Vec2>,
    scene: impl FnMut(&mut Ui) + Clone,
    interact: impl Fn(&mut Harness<'_>),
) {
    snapshot_both_with(name, size, &SnapshotOptions::default(), scene, interact);
}

/// [`snapshot_both`] with explicit comparison options.
fn snapshot_both_with(
    name: &str,
    size: Option<Vec2>,
    options: &SnapshotOptions,
    scene: impl FnMut(&mut Ui) + Clone,
    interact: impl Fn(&mut Harness<'_>),
) {
    // kittest requires the results of several harnesses in one test to be
    // merged, so a light failure still reports (or updates) dark too.
    let mut results = SnapshotResults::new();
    for theme in [Theme::Light, Theme::Dark] {
        let mut h = harness(theme, size, scene.clone());
        h.step();
        interact(&mut h);
        h.run();
        h.snapshot_options(format!("{name}-{}", suffix(theme)), options);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

#[test]
fn button_variants() {
    snapshot_both(
        "button_variants",
        None,
        |ui| {
            for size in [Size::Sm, Size::Md, Size::Lg] {
                ui.horizontal(|ui| {
                    ui.add(Button::new("Primary").primary().size(size));
                    ui.add(Button::new("Secondary").secondary().size(size));
                    ui.add(Button::new("Ghost").ghost().size(size));
                    ui.add(Button::new("Danger").danger().size(size));
                });
            }
        },
        |_| {},
    );
}

#[test]
fn button_states() {
    snapshot_both(
        "button_states",
        None,
        |ui| {
            ui.horizontal(|ui| {
                ui.add(Button::new("Focused").secondary());
                ui.add(Button::new("Selected").ghost().selected(true));
                ui.add_enabled(false, Button::new("Disabled").primary());
                ui.add_enabled(false, Button::new("Disabled").secondary());
            });
            ui.horizontal(|ui| {
                ui.add(Button::new("New task").secondary().trailing("N"));
                ui.add(
                    Button::new("Urgent")
                        .ghost()
                        .dot(Color32::from_rgb(0xE5, 0x48, 0x4D)),
                );
            });
        },
        |h| h.get_by_label("Focused").focus(),
    );
}

#[test]
fn text_input() {
    snapshot_both(
        "text_input",
        None,
        |ui| {
            ui.set_width(280.0);
            let mut empty = String::new();
            let mut filled = String::from("Ship the snapshot tests");
            let mut focused = String::from("Focused");
            ui.add(
                TextInput::new(&mut empty)
                    .label("Empty")
                    .placeholder("Task title")
                    .id_salt("empty"),
            );
            ui.add(
                TextInput::new(&mut filled)
                    .label("Filled")
                    .id_salt("filled"),
            );
            ui.add(
                TextInput::new(&mut focused)
                    .label("Autofocus")
                    .autofocus()
                    .id_salt("focused"),
            );
        },
        |_| {},
    );
}

#[test]
fn card() {
    snapshot_both(
        "card",
        None,
        |ui| {
            Card::new().show(ui, |ui| {
                let t = beau::theme(ui);
                ui.label(t.text("No tasks yet", TextRole::Title));
                ui.label(
                    t.text("Create a task to get started.", TextRole::Body)
                        .color(t.colors.text_muted),
                );
                ui.add_space(t.space.lg);
                ui.add(Button::new("New task").secondary().trailing("N"));
            });
        },
        |_| {},
    );
}

#[test]
fn kbd() {
    snapshot_both(
        "kbd",
        None,
        |ui| {
            ui.horizontal(|ui| {
                ui.add(Kbd::new("esc"));
                ui.add(Kbd::new("N"));
                ui.add(Kbd::new("⌘K"));
            });
        },
        |_| {},
    );
}

#[test]
fn dialog() {
    snapshot_both_with(
        // A fixed viewport: the dialog is an Area centred on the screen,
        // which fitting to contents can't see.
        "dialog",
        Some(vec2(560.0, 340.0)),
        // The layered modal shadow's soft edges rasterize a few levels apart
        // on different GPUs (macOS Metal vs lavapipe in CI: ~1.7k pixels
        // over the default threshold, none over 5). Content changes are far
        // above this.
        &SnapshotOptions::new().threshold(DIALOG_THRESHOLD),
        |ui| {
            ui.label("Content behind the backdrop");
            let mut open = true;
            Dialog::new("snapshot-dialog")
                .title("Delete task?")
                .description("This can't be undone.")
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    Dialog::footer(ui, |ui| {
                        ui.add(Button::new("Delete").danger());
                        ui.add(Button::new("Cancel").ghost());
                    });
                });
        },
        |_| {},
    );
}

#[test]
fn stock_controls() {
    snapshot_both(
        "stock_controls",
        None,
        |ui| {
            let (mut on, mut off, mut view, mut pts, mut lead) = (true, false, 0, 3, 15.0);
            let mut name = String::from("Weekly planning");
            ui.checkbox(&mut on, "Checked");
            ui.checkbox(&mut off, "Unchecked");
            ui.horizontal(|ui| {
                ui.radio_value(&mut view, 0, "List");
                ui.radio_value(&mut view, 1, "Board");
            });
            ui.add(
                egui::Slider::new(&mut lead, 0.0..=60.0)
                    .fixed_decimals(0)
                    .text("Slider"),
            );
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Combo")
                    .selected_text("Medium")
                    .icon(beau::paint::select_chevrons)
                    .show_ui(ui, |_| {});
                let label = ui.label("Drag");
                ui.add(egui::DragValue::new(&mut pts).suffix(" pts"))
                    .labelled_by(label.id);
            });
            ui.horizontal(|ui| {
                let label = ui.label("Name");
                ui.text_edit_singleline(&mut name).labelled_by(label.id);
            });
            ui.horizontal(|ui| {
                let _ = ui.button("Button");
                let _ = ui.selectable_label(true, "Selected");
                let _ = ui.selectable_label(false, "Unselected");
                let _ = ui.small_button("Small");
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_enabled(false, egui::Checkbox::new(&mut on, "Disabled"));
                ui.add_enabled(false, egui::Button::new("Disabled"));
            });
        },
        |_| {},
    );
}

#[test]
fn stock_states() {
    snapshot_both(
        "stock_states",
        None,
        |ui| {
            let mut a = String::from("Weekly planning");
            let mut b = String::from("Beau Core");
            let mut on = true;
            for (label, text) in [("Focused", &mut a), ("Hovered", &mut b)] {
                ui.horizontal(|ui| {
                    let label = ui.label(label);
                    ui.text_edit_singleline(text).labelled_by(label.id);
                });
            }
            ui.horizontal(|ui| {
                let _ = ui.button("Hovered button");
                ui.checkbox(&mut on, "Checkbox");
            });
        },
        |h| {
            // One focus per scene: the button's is in `stock_button_focus`.
            h.get_by_role_and_label(Role::TextInput, "Focused").focus();
            h.get_by_label("Hovered button").hover();
        },
    );
}

#[test]
fn stock_button_focus() {
    snapshot_both(
        "stock_button_focus",
        None,
        |ui| {
            ui.horizontal(|ui| {
                let _ = ui.button("Focused button");
                let mut on = true;
                ui.checkbox(&mut on, "Checkbox");
            });
        },
        |h| h.get_by_label("Focused button").focus(),
    );
}

#[test]
fn stock_popups() {
    snapshot_both_with(
        // A fixed viewport: popups are Areas, which fitting can't see.
        "stock_popups",
        Some(vec2(360.0, 220.0)),
        // Soft shadows, like the dialog.
        &SnapshotOptions::new().threshold(DIALOG_THRESHOLD),
        |ui| {
            ui.horizontal(|ui| {
                let menu = ui.menu_button("Actions", |ui| {
                    let _ = ui.button("Duplicate");
                    let _ = ui.button("Export…");
                    ui.separator();
                    let _ = ui.button("Archive");
                });
                if ui.ctx().cumulative_pass_nr() == 0 {
                    egui::Popup::open_id(
                        ui.ctx(),
                        egui::Popup::default_response_id(&menu.response),
                    );
                }
                ui.add_space(96.0);
                let reset = ui.button("Reset");
                egui::Tooltip::for_widget(&reset).show(|ui| ui.label("Restore defaults"));
            });
        },
        |h| {
            // The menu opens on the next pass; then hover one of its rows.
            h.step();
            h.get_by_label("Export…").hover();
        },
    );
}
