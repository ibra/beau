# beau

> [!NOTE]
> **AI-assisted development.** Some code in this repository is drafted with LLM
> tools. The architecture and design are set by the maintainer, and no change
> reaches `main` without being read, tested and approved by hand.

[![CI](https://github.com/ibra/beau/actions/workflows/ci.yml/badge.svg)](https://github.com/ibra/beau/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![crates.io](https://img.shields.io/crates/v/beau.svg)](https://crates.io/crates/beau)
[![docs.rs](https://img.shields.io/docsrs/beau)](https://docs.rs/beau)
![Rust 1.95+](https://img.shields.io/badge/rust-1.95%2B-orange.svg)

A design system for [egui](https://github.com/emilk/egui). beau gives native
and web Rust apps a polished, consistent look with very little code: design
tokens, typography, drawing primitives, motion and components that are
designed together.

**[Live demo →](https://ibra.github.io/beau/)**

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/hero-dark.png">
  <img alt="The tracker example, a small task tracker built with beau" src="https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/hero-light.png">
</picture>

![Adding, completing, deleting and restoring tasks from the keyboard, then switching to dark mode](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/hero.gif)

> [!NOTE]
> beau is pre-1.0. APIs may change between minor versions; see the
> [changelog](https://github.com/ibra/beau/blob/main/crates/beau/CHANGELOG.md).

## Features

- **Design tokens.** A semantic palette, type scale, spacing, radii, elevation
  and motion timings for light and dark mode. Light and dark follow the OS.
- **Your colour, safely.** The default primary is ink. Set a brand accent and
  hover, focus, selection and on-accent colours are derived to meet WCAG
  contrast.
- **Typography.** Inter (variable, Latin subset) with real weights, a display
  optical size for headings and tabular figures so numbers align.
- **Drawing primitives.** Layered soft shadows, anti-aliased gradients,
  physical-pixel hairlines and focus rings.
- **Motion.** Frame-rate-independent springs and *presence*: elements and list
  rows animate out after you stop drawing them. Nothing repaints while idle.
- **Components.** `Button`, `TextInput`, `Checkbox`, `Switch`, `Slider`,
  `Card`, `Dialog`, `Popover`, `Tooltip`, `Icon` and `Kbd`, with more on the
  [roadmap](https://github.com/ibra/beau/milestones).
- **Stock widgets too.** `beau::install` restyles egui's own buttons,
  checkboxes, radios, sliders, combo boxes, drag values, text edits, menus,
  tooltips and scroll bars, so existing egui code looks at home next to beau
  components.
- **Native and web.** Everything builds for desktop and WebAssembly.

## Installation

beau is on [crates.io](https://crates.io/crates/beau). It builds on egui
0.37:

```sh
cargo add beau egui@0.37
```

## Usage

Install beau once, then use its components alongside regular egui code:

```rust
// In your eframe app's constructor:
beau::install(&cc.egui_ctx);

// In your UI:
if ui.add(beau::Button::new("Create").primary()).clicked() {
    // ...
}
```

Use your brand colour as the accent:

```rust
beau::install_with_accent(&cc.egui_ctx, egui::Color32::from_rgb(0x3B, 0x82, 0xF6));
```

Force light or dark instead of following the OS:

```rust
beau::set_preference(ctx, egui::ThemePreference::Dark);
```

## Example app

`examples/tracker` is a small task tracker used as the showcase and test bed.
It is deployed to [ibra.github.io/beau](https://ibra.github.io/beau/) on every
push to `main`.

```sh
cargo run -p tracker                      # native
cd examples/tracker && trunk serve        # web
```

| Light | Dark |
| --- | --- |
| ![Dialog, light](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/dialog-light.png) | ![Empty state, dark](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/empty-dark.png) |

## Stock egui vs beau

The same settings page of stock egui widgets, rendered without and with
`beau::install`. The cards, the primary button and the description field are
beau components; without beau they fall back to their egui equivalents.

```sh
cargo run -p tracker -- --no-beau   # plain egui
cargo run -p tracker -- --stock     # with beau
```

| | Stock egui | With beau |
| --- | --- | --- |
| Light | ![Stock egui, light](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/compare-stock-light.png) | ![beau, light](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/compare-beau-light.png) |
| Dark | ![Stock egui, dark](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/compare-stock-dark.png) | ![beau, dark](https://raw.githubusercontent.com/ibra/beau/main/docs/screenshots/compare-beau-dark.png) |

## Design

[`DESIGN.md`](https://github.com/ibra/beau/blob/main/DESIGN.md) describes the
visual language: tokens, motion principles and the rules every component
follows.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](https://github.com/ibra/beau/blob/main/CONTRIBUTING.md) and
the [Code of Conduct](https://github.com/ibra/beau/blob/main/CODE_OF_CONDUCT.md). Report security issues privately as
described in [SECURITY.md](https://github.com/ibra/beau/blob/main/SECURITY.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/ibra/beau/blob/main/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/ibra/beau/blob/main/LICENSE-MIT))

at your option. The bundled Inter font is licensed under the
[SIL Open Font License 1.1](https://github.com/ibra/beau/blob/main/crates/beau/assets/fonts/OFL.txt). The bundled
icons come from [Lucide](https://lucide.dev), licensed under the
[ISC and MIT licences](https://github.com/ibra/beau/blob/main/crates/beau/assets/icons/LICENSE).
