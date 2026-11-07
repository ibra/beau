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
use egui::{Color32, Theme, Ui, Vec2, vec2};
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};

/// beau clamps `dt` to 1/30s, so step at that rate for motion to advance.
const STEP_DT: f32 = 1.0 / 30.0;
/// Enough frames at [`STEP_DT`] for the slowest spring (0.28s) to settle.
const MAX_STEPS: u64 = 120;
/// Render at 2x, so physical-pixel hairlines differ from 1pt strokes.
const PIXELS_PER_POINT: f32 = 2.0;

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
    // kittest requires the results of several harnesses in one test to be
    // merged, so a light failure still reports (or updates) dark too.
    let mut results = SnapshotResults::new();
    for theme in [Theme::Light, Theme::Dark] {
        let mut h = harness(theme, size, scene.clone());
        h.step();
        interact(&mut h);
        h.run();
        h.snapshot(format!("{name}-{}", suffix(theme)));
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
    snapshot_both(
        // A fixed viewport: the dialog is an Area centred on the screen,
        // which fitting to contents can't see.
        "dialog",
        Some(vec2(560.0, 340.0)),
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
