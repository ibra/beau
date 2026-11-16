//! Interaction tests: components driven with pointer and keyboard input
//! through `egui_kittest`. They need no renderer.

use egui::{Key, pos2};
use egui_kittest::{Harness, kittest::Queryable as _};

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
