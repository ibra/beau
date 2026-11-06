# beau

Beautiful UI for [egui](https://github.com/emilk/egui).

beau gives native and web Rust apps a polished, recognizable look with very
little effort. Its visual identity is **Graphite**: dense, precise and calm,
with electric lime on graphite, hairline construction and quick,
critically damped motion. See [`DESIGN.md`](DESIGN.md).

![Tracker, light](docs/screenshots/m1-light.png)

> **Status:** pre-release (milestone 1 of 5). APIs will change.

## Quick start

```rust
// In your eframe app's constructor:
beau::install(&cc.egui_ctx);

// In your UI:
if ui.add(beau::Button::new("Create").primary()).clicked() { /* ... */ }
```

## What's inside

| layer | |
|---|---|
| `tokens` | the design system as data: palette, type, spacing, radii, elevation, motion |
| `typography` | bundled Inter (variable, OFL) and the type scale |
| `paint` | layered shadows, gradients, hairlines, focus rings |
| `motion` | analytic springs, easing, and **presence**: animate in *and out* |
| `components` | `Button`, `TextInput`, `Card`, `Dialog`, `Kbd` |

## Reference app

```sh
cargo run -p tracker                     # native
cd examples/tracker && trunk serve       # web
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option. The bundled Inter font is licensed under the
[SIL Open Font License 1.1](crates/beau/assets/fonts/OFL.txt).
