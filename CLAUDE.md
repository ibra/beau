# beau

beau is a Rust library that makes egui apps beautiful. It has its own visual identity, **Graphite**; see `DESIGN.md`, which is the visual source of truth. `examples/tracker` is the showcase and test bed: every component earns its place there first.

## Layering rules (non-negotiable)
Modules are built bottom-up, and each may only depend on the layers above it in this list:
`tokens → typography → paint → motion → behavior → layout → components`.

- **Beauty comes from the foundations.** If a component needs lots of custom drawing, a foundation layer is missing. Add it to `paint` or `motion` instead.
- **`behavior`** (focus, keyboard navigation, popover positioning) knows nothing about colours or radii.
- **Feel native to egui.** Widgets use `ui.add(Widget)` or `.show(ui, |ui| …)` builders.
- **Taste over flexibility.** Ship one excellent light and dark theme before adding knobs.
- **No `unsafe`.** The workspace sets `forbid(unsafe_code)`.
- **Minimal dependencies.** Every dependency has a one-line justification comment in its `Cargo.toml`. The library depends on `egui` only.
- **Docs.** Every public item has a doc comment, and each component has a short example.

## Commands
```sh
cargo build --workspace
cargo test -p beau
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings
cargo fmt --all
cargo run -p tracker
cargo run -p tracker -- --screenshot out.png [--dark] [--dialog]   # renders, saves PNG, exits
(cd examples/tracker && trunk serve)                               # web build
```

## Workflow
- Work one milestone at a time (see the roadmap in the original brief). At the end of each milestone, stop, summarize, show screenshots and wait for review.
- After each layer, take light **and** dark screenshots with `--screenshot` and critique them against `DESIGN.md`. Call out anything that is merely "fine": the bar is beautiful, not acceptable.
- Commit only when asked.

## egui 0.37 gotchas (differs from older egui)
- `eframe::App` has `fn ui(&mut self, ui: &mut Ui, frame)` (required) and an optional `fn logic(ctx, frame)`. There is no `update`.
- Panels: `egui::Panel::left(id).show(ui, …)` and `CentralPanel::default().show(ui, …)`, which take a `&mut Ui`, not a ctx.
- `Id::new` is deprecated. Use `Id::unique("beau.x")` for globals and `ui.make_persistent_id(salt)` / `id.with(salt)` for local ids. `Ui::id` is now `Ui::scope_id`.
- `Context::style` is now `global_style`. Use `all_styles_mut` to change both themes.
- Fonts: variable fonts are supported via `FontTweak { coords: VariationCoords::new([(b"wght", 500.0)]) }`.
- Screenshots: `ctx.request_screenshot(|img: Arc<ColorImage>| …)`.
- Accessibility roles are `egui::accesskit::Role` (`WidgetType` is gone).
