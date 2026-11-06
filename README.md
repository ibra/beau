# beau

> **On AI use.** beau is built with substantial help from AI (Anthropic's
> Claude), which writes much of the code. I've had a deep role throughout:
> authoring and directing the work, making the design and engineering
> decisions, reviewing what gets built, and setting the taste.

Beautiful UI for [egui](https://github.com/emilk/egui).

beau gives native and web Rust apps a polished, recognizable look with very
little effort. Its visual identity is **Graphite**: dense, precise and calm,
built from ink, hairlines and quick, critically damped motion. Colour belongs
to your app, not the library. See [`DESIGN.md`](DESIGN.md).

![Tracker, light](docs/screenshots/m1-light.png)

> **Status:** pre-release. APIs will change.

## Quick start

```rust
// In your eframe app's constructor:
beau::install(&cc.egui_ctx);

// In your UI:
if ui.add(beau::Button::new("Create").primary()).clicked() { /* ... */ }
```

The default accent is ink (monochrome). To use your brand colour, pass it in;
hover, focus ring, selection and on-accent text are derived with WCAG
contrast guarantees:

```rust
let theme = beau::Theme::light().with_accent(egui::Color32::from_rgb(0x3B, 0x82, 0xF6));
beau::install_with(&cc.egui_ctx, theme);
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
