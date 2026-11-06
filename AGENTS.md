# AGENTS.md

Guidance for AI coding agents working in this repository. Human contributors
should read [CONTRIBUTING.md](CONTRIBUTING.md); everything there applies to
agents too.

## Project

beau is a Rust library that gives egui apps a polished, consistent look:
design tokens, typography, drawing primitives, motion and components.
[`DESIGN.md`](DESIGN.md) is the source of truth for visual decisions.
`examples/tracker` is the reference app and test bed; every component is
proven there before it is considered done.

## Repository layout

```
crates/beau/            the library
  src/tokens/           Theme, palette, scales (the design system as data)
  src/typography.rs     bundled Inter, type scale
  src/paint/            shadows, gradients, hairlines, focus rings
  src/motion/           springs, easing, presence
  src/components/       widgets
  assets/fonts/         Inter (OFL) and the tabular-digits subset
examples/tracker/       reference app (native and web)
docs/screenshots/       images referenced by the README and PRs
```

## Architecture rules

- **Layering.** Modules depend only on layers below them:
  `tokens → typography → paint → motion → behavior → layout → components`.
- **Foundations first.** If a component needs a lot of custom drawing, add the
  missing primitive to `paint` or `motion` instead.
- **`behavior` is unstyled.** Focus, keyboard navigation and positioning code
  never references colours or radii.
- **egui conventions.** Widgets implement `egui::Widget` or expose
  `.show(ui, |ui| …)`.
- **No `unsafe`**, and no new dependencies without a justification comment in
  `Cargo.toml`. The library depends only on `egui`.
- **Docs.** Every public item has a doc comment; components have a compiling
  example.

## Commands

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo test -p beau
RUSTDOCFLAGS="-D warnings" cargo doc -p beau --no-deps
cargo run -p tracker -- --screenshot out.png [--dark] [--dialog] [--empty]
(cd examples/tracker && trunk build --release)
```

## Workflow

- Work from an issue. Branch from `main`, open a pull request that references
  the issue, and title it as a Conventional Commit (see CONTRIBUTING).
- `main` is protected; never push to it directly and never merge or approve
  your own pull request. The maintainer merges.
- For any visual change, render light and dark screenshots with
  `--screenshot`, review them against `DESIGN.md`, and include them in the PR.
  Call out anything that is merely acceptable rather than well designed.
- Taste and product decisions belong to the maintainer. When a change involves
  one, propose options and ask rather than choosing silently.

## egui 0.37 notes

APIs that differ from older egui versions:

- `eframe::App` has a required `fn ui(&mut self, ui: &mut Ui, frame)` and an
  optional `fn logic(ctx, frame)`; there is no `update`.
- Panels: `egui::Panel::left(id).show(ui, …)` and
  `CentralPanel::default().show(ui, …)` take a `&mut Ui`.
- `Id::new` is deprecated: use `Id::unique(..)` for globals and
  `ui.make_persistent_id(..)` / `id.with(..)` locally. `Ui::id` is
  `Ui::scope_id`.
- Styles are per theme: `ctx.style_mut_of(theme, …)`; `Context::style` is
  `global_style`.
- Variable fonts: `FontTweak { coords: VariationCoords::new([(b"wght", 500.0)]), .. }`.
- Screenshots: `ctx.request_screenshot(|image| …)`.
- Accessibility roles are `egui::accesskit::Role`.
- A new `Area` runs an invisible sizing pass first; focus requests made during
  it are dropped (check `ui.is_sizing_pass()`).
