# beau design: Graphite

Graphite is dense, precise and calm: a professional instrument, not a toy. It should be recognizable from a single screenshot. Every screenshot gets checked against this file.

## Signatures (what makes it *beau*)
1. **Electric lime on graphite.** The accent `#C6F432` is a *fill* paired with near-black ink `#0B0C0E` in both modes. Lime is never used as text on light backgrounds; light mode uses `accent_text` (`#4D6B00`) instead. Text selection is a lime wash.
2. **Hairline construction.** Structure comes from crisp, pixel-snapped 1px *inside* strokes and quiet tonal steps between `bg`, `surface` and `surface_raised`. Use dividers instead of boxes.
3. **Shadows only on things that float**: popovers, dialogs and menus. They are tight and layered (a contact shadow plus an ambient one), never a big fuzzy glow.
4. **Small, consistent radii:** 3 / 5 / 8px, and nothing in between.
5. **Density.** 13px base type, 28px controls (24 / 28 / 32), 32px rows.
6. **Keyboard is first-class and visible.** Shortcuts appear as muted trailing hints (`New task  N`, `esc`). The focus ring is 2px with a 1px gap, so it reads on both lime and graphite.
7. **Motion is quick and critically damped**, about 120–200ms. Things fade and drift 4–6px; nothing bounces or flies.

## Tokens

### Colour

| token | light | dark | use |
|---|---|---|---|
| `bg` | `#F3F3F4` | `#0B0C0E` | app canvas, sidebar |
| `surface` | `#FBFBFB` | `#111215` | main content area |
| `surface_raised` | `#FFFFFF` | `#17181C` | cards, inputs, secondary buttons |
| `surface_overlay` | `#FFFFFF` | `#1B1C21` | dialogs, popovers |
| `surface_hover` | ink 4.5% | white 5% | hover wash (translucent) |
| `surface_active` | ink 8% | white 9% | pressed / selected wash |
| `border` | `#E4E4E7` | `#25272C` | hairlines |
| `border_strong` | `#D2D3D7` | `#33353B` | input / control outlines |
| `text` | `#0B0C0E` | `#EDEEF0` | primary text |
| `text_muted` | `#5D6068` | `#9C9EA5` | secondary text |
| `text_subtle` | `#8E9199` | `#686B73` | hints, IDs, placeholders |
| `text_on_accent` | `#0B0C0E` | `#0B0C0E` | ink on lime |
| `accent` | `#C6F432` | `#C6F432` | primary fill |
| `accent_hover` | `#B9E826` | `#D3FB5A` | primary hover |
| `accent_text` | `#4D6B00` | `#C6F432` | accent used as text or icon |
| `danger` | `#E5484D` | `#E5484D` | destructive fill |
| `danger_hover` | `#D93D42` | `#F0585D` | |
| `danger_text` | `#CE2C31` | `#FF6369` | destructive text |
| `focus_ring` | `#6F9C00` | `#C6F432` | light uses darker lime for ≥3:1 contrast |
| `backdrop` | ink 28% | black 55% | dialog scrim |

### Type (Inter variable, Latin subset; weights 400 / 500 / 600)

| role | size / line | weight | tracking |
|---|---|---|---|
| caption | 11 / 16 | 500 | +0.1 |
| small | 12 / 16 | 400 | 0 |
| body | 13 / 20 | 400 | 0 |
| body_strong | 13 / 20 | 500 | 0 |
| title | 15 / 22 | 600 | −0.1 |
| heading | 18 / 24 | 600 (opsz 28) | −0.25 |
| display | 24 / 30 | 600 (opsz 32) | −0.5 |

Large sizes use Inter's *display* optical size, which gives tighter, sharper headings.

### Spacing, radii, controls
- **Spacing:** xxs 2 · xs 4 · sm 6 · md 8 · lg 12 · xl 16 · xxl 24 · xxxl 32.
- **Radii:** sm 3 · md 5 · lg 8 · full.
- **Controls:** sm 24 · md 28 · lg 32. Row 32.

### Elevation (layered shadows)
- **flat:** no shadow.
- **raised:** used by cards. A hairline only, plus a 1px contact shadow at 4%.
- **overlay:** used by popovers. A contact shadow (y1, blur 2, 6%) plus an ambient shadow (y6, blur 16, 8%).
- **modal:** used by dialogs. Contact (y1, blur 2, 8%), mid (y8, blur 24, 12%) and ambient (y24, blur 56, 14%). Dark mode uses black at higher alpha.

## Motion
- **Springs** are given as `response` (seconds) and `damping_ratio`. The default ratio is 1.0 (critically damped).
  - `fast` 0.12s: hover and press.
  - `base` 0.18s: focus, dialog, presence.
  - `slow` 0.28s: layout shifts.
- They are solved analytically, so they are frame-rate independent. `dt` is clamped to 1/30s after idle.
- **Repaint** is requested only while something is unsettled. At rest the app draws nothing.
- **Reduced motion** snaps every value to its target.

## Presence (animate in *and* out)
- `motion::presence(ctx, id, visible, spec)` returns `Presence { t, phase }`, where `phase` is `Entering`, `Present`, `Exiting` or `Gone`. The caller keeps drawing until the phase is `Gone`. While exiting, the element is non-interactive.
- `motion::PresenceList` covers keyed lists, where the caller removes an item from its data. The list keeps a clone of each departed item as a *ghost* at its old position. The ghost's height collapses to 0 (so neighbours slide up) and its opacity fades out; then the clone is dropped. New keys grow in from 0 height.
- All state lives in egui temp memory and is garbage-collected two passes after it was last touched.

## Do / don't
- **Do** use tonal steps and hairlines for structure. **Don't** box everything.
- **Do** keep lime for the single most important action on screen. **Don't** use it as decoration.
- **Don't** put lime text on light backgrounds.
- **Don't** add shadows to inline content.
- **Do** align numbers and IDs in muted `small` text.
- **Don't** use radii outside the 3 / 5 / 8 scale.
