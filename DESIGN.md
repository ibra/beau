# Design

beau is dense, precise and calm. It aims to look and feel right in anyone's app: true-neutral grays that sit next to any brand colour, ink as the default primary, quiet structure and quick motion. Every visual change is reviewed against this document.

## Principles
1. **Ink, not colour.** The primary action is solid ink: `#0C0C0C` in light mode, `#EDEDED` in dark. The library imposes no brand colour; colour belongs to the app, via `Theme::with_accent(color)`, which derives every accent token with contrast guarantees.
2. **Hairline construction.** Structure comes from quiet tonal steps between `bg`, `surface` and `surface_raised`, plus fine lines. Dividers and surface outlines are exactly **one physical pixel** (`Theme::divider`). Control outlines (inputs, secondary buttons) are **1pt** (`Theme::control_border`), so they read as interactive. Use dividers instead of boxes, and spacing instead of dividers where possible.
3. **Shadows only on things that float**: popovers, dialogs and menus. They are tight and layered (a contact shadow plus an ambient one), never a big fuzzy glow.
4. **Consistent radii:** 4 / 6 / 10px. Crisp, but not severe.
5. **Density.** 13px base type, 28px controls (24 / 28 / 32), 36px list rows. Digits are tabular everywhere, so numbers, IDs and dates align.
6. **Keyboard is first-class and visible.** Shortcuts appear as muted trailing hints (`New task  N`, `esc`). The focus ring is 2px with a 1px gap, in `focus_ring` (ink by default). Text inputs are the exception: their border turns `focus_ring` and a soft 35% halo replaces the gap ring, which looks less heavy around large fields.
7. **Motion is quick and critically damped**, about 120–200ms. Things fade and drift 4–6px; nothing bounces or flies.
8. **Light and dark follow the OS** by default (`beau::install`). Apps can force a mode with `beau::set_preference`.

## Tokens

### Colour

| token | light | dark | use |
|---|---|---|---|
| `bg` | `#F4F4F4` | `#0C0C0C` | app canvas, sidebar |
| `surface` | `#FBFBFB` | `#121212` | main content area |
| `surface_raised` | `#FFFFFF` | `#181818` | cards, inputs, secondary buttons |
| `surface_overlay` | `#FFFFFF` | `#1C1C1C` | dialogs, popovers |
| `surface_hover` | ink 4.5% | white 5% | hover wash (translucent) |
| `surface_active` | ink 8% | white 9% | pressed / selected wash |
| `border` | `#E5E5E5` | `#262626` | hairlines |
| `border_strong` | `#D6D6D6` | `#333333` | input / control outlines |
| `text` | `#0C0C0C` | `#EDEDED` | primary text |
| `text_muted` | `#616161` | `#A0A0A0` | secondary text |
| `text_subtle` | `#8F8F8F` | `#6B6B6B` | hints, IDs, placeholders |
| `accent` | `#0C0C0C` | `#EDEDED` | primary fill (default: ink) |
| `text_on_accent` | white | `#0C0C0C` | derived: whichever contrasts more |
| `accent_hover` | derived | derived | one step lighter (dark accents) or darker |
| `accent_text` | derived | derived | accent pushed towards `text` until ≥4.5:1 on `surface` |
| `danger` | `#E5484D` | `#E5484D` | destructive fill |
| `danger_hover` | `#D93D42` | `#F0585D` | |
| `danger_text` | `#CE2C31` | `#FF6369` | destructive text |
| `focus_ring` | derived | derived | accent pushed towards `text` until ≥3:1 on `surface` |
| `selection` | derived | derived | accent wash (18% light, 32% dark) |
| `backdrop` | ink 28% | black 55% | dialog scrim |

Neutrals are untinted grays, so they don't fight warm or cool accents.

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

Large sizes use Inter's *display* optical size, which gives tighter, sharper headings. Digits come from a 9 KB tabular-figures subset placed first in every font family (egui can't toggle OpenType `tnum`).

### Spacing, radii, controls
- **Spacing:** xxs 2 · xs 4 · sm 6 · md 8 · lg 12 · xl 16 · xxl 24 · xxxl 32.
- **Radii:** sm 4 · md 6 · lg 10 · full.
- **Controls:** sm 24 · md 28 · lg 32. Row 36.

### Icons
- **Set:** 75 icons from [Lucide](https://lucide.dev) (`beau::icons`), drawn as vector strokes, not a font or images.
- **Size:** 16 by default (`controls.icon`); 14 in small buttons.
- **Stroke:** 1.5pt at every size, so an icon keeps the weight of the 13px text beside it. Ends and dots are extended to match Lucide's round caps.
- **Colour:** the surrounding text colour. In a button the icon takes the label colour, 6pt before the label. A button's dot is centred in the same 16pt slot, so labels align where dots and icons are stacked.
- **Icon-only buttons** are square (`Button::icon_only`) and carry a label for screen readers.

### Checkbox and Switch
- **Checkbox:** a 14pt box at `radius.sm`. Unchecked is `surface_raised` with a 1pt `border_strong` outline that darkens on hover. Checked fills with the accent (ink by default) and draws a 1.5pt check in `text_on_accent`; the check draws itself in as the fill fades. Indeterminate shows a dash.
- **Switch:** a 28×16pt track with a 12pt knob. Off is a neutral gray, darker than a control border so it shows on every surface; on is the accent with a `text_on_accent` knob. The knob slides on the `base` spring and stretches 3pt while pressed.
- **When to use which:** a switch applies at once (settings); a checkbox is confirmed later (forms, selection).
- **Disabled:** neutral fill, subtle mark, full opacity. **Focus:** the standard ring around the box or track. Labels are part of the click target, 8pt after the control.

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
- **Don't** use outlined pills or chips for metadata. Use a small dot plus muted text.
- **Don't** ship dead controls. A button that does nothing is removed until it works.
- **Do** make disabled controls neutral (surface fill, subtle text, full opacity), not faded.
- **Do** keep signals quiet: urgency is a tinted mark, not a solid red block.
- **Do** separate groups with rhythm (space above a plain label), not bands or lines.
- **Do** align metadata in fixed-width columns so it reads down the list.
- **Do** use tonal steps and hairlines for structure. **Don't** box everything.
- **Do** keep the accent fill for the single most important action on screen. **Don't** use it as decoration.
- **Don't** add shadows to inline content.
- **Do** align numbers and IDs in muted `small` text.
- **Don't** use radii outside the 4 / 6 / 10 scale.
- **Do** give empty views a quiet raised `Card` with a title, a one-line hint and one secondary action.

## Stock egui widgets

`beau::install` restyles egui's own widgets, so apps can adopt beau
gradually. `Theme::apply_to_style` maps tokens onto egui's `Style`, and
`beau::stock::StockStyle` (an egui `StyleProvider`) draws the widgets egui
lets a theme fully style. Everything comes from the theme's tokens.

- **Buttons, drag values and combo boxes** are secondary controls:
  `surface_raised`, a 1pt `border_strong` outline, the raised contact
  shadow, hover and press washes, 500-weight labels (400 for value fields).
  Selected buttons take `surface_active`.
- **Menu and list rows** inside popups are ghost rows, 24pt tall, with a
  `surface_hover` wash at `radius.sm`, inset from the popup edge.
- **Text edits** match `TextInput`: the border darkens on hover, then turns
  `focus_ring` with a 35% halo when focused. Keyboard-focused buttons and
  checkboxes use the same halo.
- **Checkboxes** are 14pt at `radius.sm` with an ink check. **Radios and
  slider rails** are a soft gray (`surface_raised` plus the active wash) that
  turns white under the pointer, so rails show on raised surfaces.
- **Separators** are physical-pixel hairlines. **Scroll bars** float: 2pt,
  growing to 6pt under the pointer, ink at low alpha, no track, hidden at
  rest.
- **Menus and tooltips** use `surface_overlay` with the widest overlay
  shadow layer; windows use the widest modal layer.
- `beau::paint::select_chevrons` gives a `ComboBox` beau's up/down chevrons
  (pass it to `ComboBox::icon`).

Limits, until beau has its own component: stock widgets change state
instantly (egui gives a style provider no widget id to animate); a checked
checkbox keeps a white box; the slider knob is 80% of the control height
with an ink outline; disabled stock widgets fade instead of turning neutral;
floating surfaces get one shadow layer, not beau's layered elevation.

## Customization

beau stays opinionated: few settings, each moving many tokens together so any
combination still looks designed. The accent is configurable today
(`Theme::with_accent`, `beau::install_with_accent`). Roundness, density and
font choice are planned for
[v0.4.0](https://github.com/ibra/beau/milestone/4) (#27, #28, #29).
