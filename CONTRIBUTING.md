# Contributing to beau

Thanks for your interest in beau. This guide covers how work is planned, how
to set up a development environment, and what a pull request needs before it
can be merged.

By participating you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## How work is planned

- **Issues** track every change. Bugs and feature requests use the issue
  templates; larger ideas start as a feature request so the design can be
  discussed before code is written.
- **Milestones** map to versions (`v0.2.0`, `v0.3.0`, …). The open milestones
  are the roadmap.
- **Labels** classify issues by type (`type: bug`, `type: feature`, …) and area
  (`area: motion`, `area: components`, …). `needs decision` marks issues
  waiting on a maintainer call.

## Development setup

Requirements: the Rust version in `rust-version` (see `Cargo.toml`) or newer,
plus the WebAssembly target and [Trunk](https://trunkrs.dev) for the web build.

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked

cargo run -p tracker                       # reference app, native
(cd examples/tracker && trunk serve)       # reference app, web
```

The tracker can render a screenshot and exit, which is the easiest way to
review visual changes:

```sh
cargo run -p tracker -- --screenshot out.png [--dark] [--dialog] [--empty] [--stock] [--no-beau]
```

## Checks

CI runs the following; please run them locally before opening a pull request.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo test -p beau
RUSTDOCFLAGS="-D warnings" cargo doc -p beau --no-deps
```

## Snapshot tests

`crates/beau/tests/snapshots.rs` renders every component in light and dark
mode and compares the result with the PNGs committed in
`crates/beau/tests/snapshots/`. It runs as part of `cargo test -p beau` and
needs a GPU or a software renderer (CI installs Mesa's lavapipe).

- **A test fails** when the rendering changes. It writes `<name>.new.png`
  (the new rendering) and `<name>.diff.png` (changed pixels) next to the
  committed image. In CI, these are uploaded as the `snapshot-diffs` artifact.
- **If the change is intended**, update the images and review them like code:

  ```sh
  UPDATE_SNAPSHOTS=1 cargo test -p beau --test snapshots
  git diff --stat crates/beau/tests/snapshots
  ```

  Commit the updated PNGs with the change that caused them, and mention them
  in the PR.
- **New components** get a test in the same file: one scene, rendered in both
  modes through `snapshot_both`.

## Pull requests

1. Branch from `main`. `main` is protected: changes land through pull requests
   with passing checks, merged with **squash and merge**.
2. Reference the issue the PR resolves (`Closes #123`).
3. Title the PR as a [Conventional Commit](https://www.conventionalcommits.org):
   the squashed commit uses the PR title, and releases are generated from it.

   | Prefix | Use for | Version effect (pre-1.0) |
   | --- | --- | --- |
   | `feat:` | new capability | minor |
   | `fix:` | bug fix | patch |
   | `perf:` | performance | patch |
   | `docs:`, `refactor:`, `test:`, `chore:`, `ci:` | no user-facing change | none |
   | `feat!:` / `BREAKING CHANGE:` footer | public API change | minor |

   An optional scope names the area, e.g. `feat(components): add Switch`.
4. **Visual changes need screenshots**, light and dark, in the PR description.
   Check them against [`DESIGN.md`](DESIGN.md).

## Code guidelines

- **Layering.** Modules depend only on layers below them:
  `tokens → typography → paint → motion → behavior → layout → components`.
  `behavior` knows nothing about colours or radii. If a component needs a lot
  of custom drawing, the missing piece belongs in `paint` or `motion`.
- **egui conventions.** Widgets implement `egui::Widget` or expose a
  `.show(ui, |ui| …)` builder.
- **No `unsafe`.** The workspace forbids it.
- **Dependencies.** Keep them minimal. Each one needs a one-line justification
  comment in its `Cargo.toml`. The library depends only on `egui`.
- **Documentation.** Every public item has a doc comment; every component has
  a short example that compiles as a doctest.

## Versioning and releases

beau follows [Semantic Versioning](https://semver.org). Before 1.0, a minor
release (`0.x.0`) may contain breaking changes and a patch release (`0.x.y`)
never does. Breaking changes are marked with the `breaking change` label and
called out in the changelog.

Releases are automated with
[release-please](https://github.com/googleapis/release-please): it keeps a
release pull request up to date with the next version and the generated
[changelog](crates/beau/CHANGELOG.md). Merging that pull request tags the
release and publishes the GitHub release notes.

## Licensing

beau is dual-licensed under MIT or Apache-2.0. Unless you explicitly state
otherwise, any contribution you submit is dual-licensed in the same way,
without any additional terms or conditions.
