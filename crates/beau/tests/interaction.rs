//! Interaction tests: components driven with pointer and keyboard input
//! through `egui_kittest`. They need no renderer.

use egui::{Key, pos2};
use egui_kittest::{
    Harness,
    kittest::{NodeT as _, Queryable as _},
};

/// A 200pt slider over 0..=100 in whole steps.
fn slider_harness() -> Harness<'static, u32> {
    let mut harness = Harness::builder()
        // Enough steps for the springs to settle after each input.
        .with_max_steps(64)
        .with_context_setup(beau::install)
        .build_ui_state(
            |ui, value: &mut u32| {
                ui.add(
                    beau::Slider::new(value, 0..=100)
                        .width(200.0)
                        .accessible_name("Volume"),
                );
            },
            50,
        );
    harness.run();
    harness
}

#[test]
fn slider_arrow_keys_step_and_home_end_jump() {
    let mut harness = slider_harness();
    harness.get_by_label("Volume").focus();
    harness.run();

    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), 51);

    harness.key_press(Key::ArrowLeft);
    harness.key_press(Key::ArrowLeft);
    harness.run();
    assert_eq!(*harness.state(), 49);

    harness.key_press(Key::End);
    harness.run();
    assert_eq!(*harness.state(), 100);
    // Clamped at the end.
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(*harness.state(), 100);

    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(*harness.state(), 0);
}

#[test]
fn slider_drag_follows_the_pointer_and_clamps() {
    let mut harness = slider_harness();
    let rail = harness.get_by_label("Volume").rect();
    let y = rail.center().y;

    // Grab the knob in the middle and pull it a quarter of the rail right.
    harness.drag_at(pos2(rail.center().x, y));
    harness.run();
    harness.hover_at(pos2(rail.center().x + rail.width() / 4.0, y));
    harness.run();
    let dragged = *harness.state();
    assert!((70..=80).contains(&dragged), "dragged to {dragged}");

    // Past the right end: clamped to the maximum.
    harness.hover_at(pos2(rail.right() + 80.0, y));
    harness.run();
    assert_eq!(*harness.state(), 100);

    // Released: moving the pointer no longer changes the value.
    harness.drop_at(pos2(rail.right() + 80.0, y));
    harness.run();
    harness.hover_at(pos2(rail.left(), y));
    harness.run();
    assert_eq!(*harness.state(), 100);

    // A click on the rail jumps there.
    harness.drag_at(pos2(rail.left() + 1.0, y));
    harness.run();
    harness.drop_at(pos2(rail.left() + 1.0, y));
    harness.run();
    assert_eq!(*harness.state(), 0);
}

/// A page with a "Share" button that opens a popover, and a "Page" button
/// behind it. The state counts clicks on "Page".
fn popover_harness() -> Harness<'static, u32> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(480.0, 320.0))
        .with_max_steps(64)
        .with_context_setup(beau::install)
        .build_ui_state(
            |ui, page_clicks: &mut u32| {
                if ui.button("Page").clicked() {
                    *page_clicks += 1;
                }
                let trigger = ui.add(beau::Button::new("Share"));
                beau::Popover::new(&trigger).show(ui.ctx(), |ui| {
                    let _ = ui.button("Inside");
                    if ui.button("Done").clicked() {
                        ui.close();
                    }
                    let nested = ui.button("More");
                    beau::Popover::new(&nested).show(ui.ctx(), |ui| {
                        let _ = ui.button("Nested");
                    });
                });
            },
            0,
        );
    harness.run();
    harness
}

fn is_shown(harness: &Harness<'_, u32>, label: &str) -> bool {
    harness.query_by_label(label).is_some()
}

#[test]
fn popover_toggles_from_its_trigger_and_closes_on_escape() {
    let mut harness = popover_harness();
    assert!(!is_shown(&harness, "Inside"));

    harness.get_by_label("Share").click();
    harness.run();
    assert!(is_shown(&harness, "Inside"));

    harness.key_press(Key::Escape);
    harness.run();
    assert!(!is_shown(&harness, "Inside"), "Escape closes it");
}

#[test]
fn popover_closes_on_outside_click_without_clicking_the_page() {
    let mut harness = popover_harness();
    harness.get_by_label("Share").click();
    harness.run();

    // A click inside keeps it open.
    harness.get_by_label("Inside").click();
    harness.run();
    assert!(is_shown(&harness, "Inside"));

    // The first click outside only closes it: the page doesn't react.
    harness.get_by_label("Page").click();
    harness.run();
    assert!(!is_shown(&harness, "Inside"));
    assert_eq!(*harness.state(), 0);
    harness.get_by_label("Page").click();
    harness.run();
    assert_eq!(*harness.state(), 1);
}

#[test]
fn popover_content_closes_it_with_ui_close() {
    let mut harness = popover_harness();
    harness.get_by_label("Share").click();
    harness.run();
    harness.get_by_label("Done").click();
    harness.run();
    assert!(!is_shown(&harness, "Inside"));
}

#[test]
fn nested_popovers_close_innermost_first() {
    let mut harness = popover_harness();
    harness.get_by_label("Share").click();
    harness.run();
    harness.get_by_label("More").click();
    harness.run();
    assert!(is_shown(&harness, "Nested"));

    // A click in the outer popover closes only the nested one.
    harness.get_by_label("Inside").click();
    harness.run();
    assert!(!is_shown(&harness, "Nested"));
    assert!(is_shown(&harness, "Inside"));

    harness.get_by_label("More").click();
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!is_shown(&harness, "Nested"));
    assert!(is_shown(&harness, "Inside"), "one Escape closes one level");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!is_shown(&harness, "Inside"));
}

#[test]
fn popover_returns_focus_to_its_trigger() {
    let mut harness = popover_harness();
    harness.get_by_label("Share").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(is_shown(&harness, "Inside"));

    // Tab stays inside the popover.
    for _ in 0..5 {
        harness.key_press(Key::Tab);
        harness.run();
        let focused = harness.get_by(|node| node.is_focused());
        let label = focused.accesskit_node().label().unwrap_or_default();
        assert!(
            ["Inside", "Done", "More"].contains(&label.as_str()),
            "focus escaped to {label:?}"
        );
    }

    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Share").accesskit_node().is_focused());
}

#[test]
fn tooltip_shows_after_a_delay_and_hides_when_pressed() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 200.0))
        .with_context_setup(beau::install)
        .build_ui(|ui| {
            ui.add_space(60.0);
            let reset = ui.add(beau::Button::new("Reset"));
            beau::Tooltip::new("Restore defaults").show(ui, &reset);
        });
    harness.run();
    let tip = |h: &Harness<'_>| h.query_by_label("Restore defaults").is_some();

    harness.get_by_label("Reset").hover();
    harness.step();
    assert!(!tip(&harness), "not before the delay");
    for _ in 0..40 {
        harness.step();
    }
    assert!(tip(&harness), "shown after resting on the widget");

    harness.get_by_label("Reset").click();
    for _ in 0..30 {
        harness.step();
    }
    assert!(!tip(&harness), "pressing the widget hides it");
}
