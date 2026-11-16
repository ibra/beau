"""Generate src/icons/set.rs from Lucide SVGs.

beau draws icons as stroked vector paths, with no SVG parser at run time. This
script converts each icon named in ../icons.txt into a flat list of move, line,
cubic and close commands on Lucide's 24-unit grid. Arcs, circles and rounded
rectangles become cubic Béziers here, so the library only flattens cubics.

It also copies the source SVGs to ../svg/ and Lucide's licence to ../LICENSE.

Usage (from this directory):
    npm pack lucide-static && tar xzf lucide-static-*.tgz
    python generate.py package
Requires: Python 3.9+, no packages.
"""
import math
import re
import shutil
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

HERE = Path(__file__).resolve().parent
ICONS = HERE.parent
OUT = ICONS.parent.parent / "src" / "icons" / "set.rs"
KAPPA = 0.5522847498307936  # cubic approximation of a quarter circle


def fmt(v):
    s = f"{v:.3f}".rstrip("0")
    if s.endswith("."):
        s += "0"
    return "0.0" if s == "-0.0" else s


class Path_:
    """Collects absolute commands."""

    def __init__(self):
        self.cmds = []
        self.cur = (0.0, 0.0)
        self.start = (0.0, 0.0)

    def move(self, x, y):
        self.cmds.append(("M", x, y))
        self.cur = self.start = (x, y)

    def line(self, x, y):
        self.cmds.append(("L", x, y))
        self.cur = (x, y)

    def cubic(self, x1, y1, x2, y2, x, y):
        self.cmds.append(("C", x1, y1, x2, y2, x, y))
        self.cur = (x, y)

    def close(self):
        self.cmds.append(("Z",))
        self.cur = self.start

    def arc(self, rx, ry, rot, large, sweep, x, y):
        """SVG endpoint arc to cubics (SVG 1.1 appendix F.6)."""
        x0, y0 = self.cur
        if (x0, y0) == (x, y):
            return
        rx, ry = abs(rx), abs(ry)
        if rx == 0 or ry == 0:
            self.line(x, y)
            return
        phi = math.radians(rot)
        cp, sp = math.cos(phi), math.sin(phi)
        dx, dy = (x0 - x) / 2, (y0 - y) / 2
        x1p, y1p = cp * dx + sp * dy, -sp * dx + cp * dy
        lam = x1p**2 / rx**2 + y1p**2 / ry**2
        if lam > 1:
            rx, ry = rx * math.sqrt(lam), ry * math.sqrt(lam)
        num = rx**2 * ry**2 - rx**2 * y1p**2 - ry**2 * x1p**2
        den = rx**2 * y1p**2 + ry**2 * x1p**2
        k = math.sqrt(max(0.0, num / den)) * (-1 if large == sweep else 1)
        cxp, cyp = k * rx * y1p / ry, -k * ry * x1p / rx
        cx = cp * cxp - sp * cyp + (x0 + x) / 2
        cy = sp * cxp + cp * cyp + (y0 + y) / 2
        t1 = math.atan2((y1p - cyp) / ry, (x1p - cxp) / rx)
        t2 = math.atan2((-y1p - cyp) / ry, (-x1p - cxp) / rx)
        dt = t2 - t1
        if sweep and dt < 0:
            dt += 2 * math.pi
        elif not sweep and dt > 0:
            dt -= 2 * math.pi
        n = max(1, math.ceil(abs(dt) / (math.pi / 2) - 1e-9))
        step = dt / n
        alpha = 4 / 3 * math.tan(step / 4)

        def point(t):
            return (
                cx + rx * math.cos(t) * cp - ry * math.sin(t) * sp,
                cy + rx * math.cos(t) * sp + ry * math.sin(t) * cp,
            )

        def deriv(t):
            return (
                -rx * math.sin(t) * cp - ry * math.cos(t) * sp,
                -rx * math.sin(t) * sp + ry * math.cos(t) * cp,
            )

        for i in range(n):
            a, b = t1 + i * step, t1 + (i + 1) * step
            pa, pb, da, db = point(a), point(b), deriv(a), deriv(b)
            end = (x, y) if i == n - 1 else pb
            self.cubic(
                pa[0] + alpha * da[0],
                pa[1] + alpha * da[1],
                pb[0] - alpha * db[0],
                pb[1] - alpha * db[1],
                *end,
            )


TOKEN = re.compile(r"([MmLlHhVvCcSsQqTtAaZz])|(-?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)")


def parse_d(d, p):
    tokens = [(m.group(1), m.group(2)) for m in TOKEN.finditer(d)]
    p.cur = p.start = (0.0, 0.0)  # a leading relative "m" is absolute
    i = 0
    cmd = None
    last_ctrl = None  # reflected control point for S/T
    last_kind = None

    def is_num():
        return i < len(tokens) and tokens[i][1] is not None

    def num():
        nonlocal i
        v = float(tokens[i][1])
        i += 1
        return v

    def flag():
        # Arc flags may be written without separators ("a1 1 0 01.5.5").
        nonlocal i
        text = tokens[i][1]
        if len(text) > 1:
            tokens[i] = (None, text[1:])
            return int(text[0])
        i += 1
        return int(float(text))

    while i < len(tokens):
        if tokens[i][0]:
            cmd = tokens[i][0]
            i += 1
            if cmd in "Zz":
                p.close()
                last_kind = None
                continue
        rel = cmd.islower()
        ox, oy = p.cur if rel else (0.0, 0.0)
        c = cmd.upper()
        kind = None
        if c == "M":
            p.move(ox + num(), oy + num())
            cmd = "l" if rel else "L"
        elif c == "L":
            p.line(ox + num(), oy + num())
        elif c == "H":
            p.line(ox + num(), p.cur[1])
        elif c == "V":
            p.line(p.cur[0], oy + num())
        elif c == "C":
            v = [num() for _ in range(6)]
            p.cubic(ox + v[0], oy + v[1], ox + v[2], oy + v[3], ox + v[4], oy + v[5])
            last_ctrl, kind = (ox + v[2], oy + v[3]), "C"
        elif c == "S":
            v = [num() for _ in range(4)]
            x0, y0 = p.cur
            c1 = (2 * x0 - last_ctrl[0], 2 * y0 - last_ctrl[1]) if last_kind == "C" else (x0, y0)
            p.cubic(*c1, ox + v[0], oy + v[1], ox + v[2], oy + v[3])
            last_ctrl, kind = (ox + v[0], oy + v[1]), "C"
        elif c in "QT":
            x0, y0 = p.cur
            if c == "Q":
                q = (ox + num(), oy + num())
            else:
                q = (2 * x0 - last_ctrl[0], 2 * y0 - last_ctrl[1]) if last_kind == "Q" else (x0, y0)
            x, y = ox + num(), oy + num()
            p.cubic(
                x0 + 2 / 3 * (q[0] - x0),
                y0 + 2 / 3 * (q[1] - y0),
                x + 2 / 3 * (q[0] - x),
                y + 2 / 3 * (q[1] - y),
                x,
                y,
            )
            last_ctrl, kind = q, "Q"
        elif c == "A":
            rx, ry, rot = num(), num(), num()
            large, sweep = flag(), flag()
            p.arc(rx, ry, rot, large, sweep, ox + num(), oy + num())
        else:
            raise ValueError(f"unsupported path command {cmd}")
        last_kind = kind
        if not is_num() and i < len(tokens) and tokens[i][0] is None:
            raise ValueError("bad path data")


def ellipse(p, cx, cy, rx, ry):
    kx, ky = rx * KAPPA, ry * KAPPA
    p.move(cx + rx, cy)
    p.cubic(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry)
    p.cubic(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy)
    p.cubic(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry)
    p.cubic(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy)
    p.close()


def convert(svg_path):
    root = ET.parse(svg_path).getroot()
    p = Path_()
    for el in root.iter():
        tag = el.tag.split("}")[-1]
        a = el.attrib
        if tag == "svg":
            continue
        if tag == "circle" and a.get("fill") == "currentColor" and float(a["r"]) <= 1:
            # A filled dot: drawn like a zero-length stroke (a round dot).
            p.move(float(a["cx"]), float(a["cy"]))
            p.line(float(a["cx"]), float(a["cy"]))
            continue
        for bad in ("fill", "transform", "stroke-width", "opacity"):
            if bad in a:
                raise ValueError(f"{svg_path.name}: unsupported attribute {bad} on <{tag}>")
        f = lambda k, default=0.0: float(a.get(k, default))
        if tag == "path":
            parse_d(a["d"], p)
        elif tag == "circle":
            ellipse(p, f("cx"), f("cy"), f("r"), f("r"))
        elif tag == "ellipse":
            ellipse(p, f("cx"), f("cy"), f("rx"), f("ry"))
        elif tag == "line":
            p.move(f("x1"), f("y1"))
            p.line(f("x2"), f("y2"))
        elif tag in ("polyline", "polygon"):
            pts = [float(v) for v in re.findall(r"-?[\d.]+", a["points"])]
            p.move(pts[0], pts[1])
            for j in range(2, len(pts), 2):
                p.line(pts[j], pts[j + 1])
            if tag == "polygon":
                p.close()
        elif tag == "rect":
            x, y, w, h = f("x"), f("y"), f("width"), f("height")
            rx = f("rx", a.get("ry", 0))
            ry = f("ry", a.get("rx", 0))
            rx, ry = min(rx, w / 2), min(ry, h / 2)
            if rx == 0 or ry == 0:
                p.move(x, y)
                p.line(x + w, y)
                p.line(x + w, y + h)
                p.line(x, y + h)
            else:
                p.move(x + rx, y)
                p.line(x + w - rx, y)
                p.arc(rx, ry, 0, 0, 1, x + w, y + ry)
                p.line(x + w, y + h - ry)
                p.arc(rx, ry, 0, 0, 1, x + w - rx, y + h)
                p.line(x + rx, y + h)
                p.arc(rx, ry, 0, 0, 1, x, y + h - ry)
                p.line(x, y + ry)
                p.arc(rx, ry, 0, 0, 1, x + rx, y)
            p.close()
        else:
            raise ValueError(f"{svg_path.name}: unsupported element <{tag}>")
    return p.cmds


def rust(cmds):
    out = []
    for c in cmds:
        if c[0] == "Z":
            out.append("Z")
        else:
            out.append(f"{c[0]}({', '.join(fmt(v) for v in c[1:])})")
    return ", ".join(out)


def main():
    package = Path(sys.argv[1])
    names = [
        line.strip()
        for line in (ICONS / "icons.txt").read_text().splitlines()
        if line.strip() and not line.startswith("#")
    ]
    if len(set(names)) != len(names):
        raise SystemExit("duplicate names in icons.txt")
    svg_dir = ICONS / "svg"
    shutil.rmtree(svg_dir, ignore_errors=True)
    svg_dir.mkdir()
    shutil.copy(package / "LICENSE", ICONS / "LICENSE")
    version = re.search(r'"version":\s*"([^"]+)"', (package / "package.json").read_text()).group(1)

    lines = [
        f"// Generated by assets/icons/tools/generate.py from Lucide {version}. Do not edit.",
        "",
        "use super::Icon;",
        "use crate::paint::PathCmd::{Close as Z, Cubic as C, Line as L, Move as M};",
        "",
    ]
    consts = []
    for name in names:
        src = package / "icons" / f"{name}.svg"
        shutil.copy(src, svg_dir / src.name)
        const = name.upper().replace("-", "_")
        consts.append(const)
        lines += [
            f"/// Lucide `{name}`.",
            "#[rustfmt::skip]",
            f"pub const {const}: Icon = Icon::from_path(&[{rust(convert(src))}]);",
        ]
    lines += [
        "",
        "/// Every bundled icon with its Lucide name, in catalogue order.",
        "pub const ALL: &[(&str, Icon)] = &[",
        *[f'    ("{n}", {c}),' for n, c in zip(names, consts)],
        "];",
        "",
    ]
    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text("\n".join(lines))
    print(f"wrote {len(names)} icons to {OUT}")


if __name__ == "__main__":
    main()
