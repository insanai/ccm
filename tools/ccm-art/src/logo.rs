//! The CCM mark: an isometric cube with its near corner cut away, holding a smaller cube.
//!
//! Seen along the cube diagonal, the mark is three concentric hexagons. The outer ring is
//! the convex body, the middle ring is the concave notch, and the center is the changed
//! cube. A change starts at the center and stays inside the layers that surround it.
//!
//! Every polygon is a face of an axis-aligned square in voxel space, projected with the
//! standard isometric map. No coordinate is tuned by hand.
use std::fmt::Write;

/// Edge length of the outer cube, in voxel units.
const OUTER: f64 = 3.0;
/// Edge length of the notch cut from the near corner of the outer cube.
const NOTCH: f64 = 2.0;
/// Edge length of the changed cube that sits in the inner corner of the notch.
const CHANGE: f64 = 1.0;
/// Width and height of the square viewBox.
const VIEW: f64 = 120.0;
/// Clear space between the hexagon and the viewBox edge, top and bottom.
const MARGIN: f64 = 4.0;
/// Width of the thin edge drawn around every face, in viewBox units.
const EDGE: f64 = 0.9;

/// Colors of the three visible faces of one solid.
struct Palette {
    top: String,
    left: String,
    right: String,
}

/// The visible part of one solid: in each axis plane `axis = level`, the square
/// `span` x `span` in the two other coordinates, minus the corner square `cut` x `cut`.
struct Shell {
    level: f64,
    span: (f64, f64),
    cut: Option<(f64, f64)>,
}

/// The three axes of voxel space. A face is named by the axis of its outward normal.
#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

/// Renders the mark as a standalone SVG document.
///
/// `mono` selects tints of the single ink color instead of the purple and pink palette.
pub(super) fn render(mono: bool) -> String {
    let mut out = String::with_capacity(4096);
    let _ = write!(
        out,
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {view} {view}\" ",
            "role=\"img\" aria-labelledby=\"ccm-logo-title ccm-logo-desc\">\n",
            "<title id=\"ccm-logo-title\">CCM mark</title>\n",
            "<desc id=\"ccm-logo-desc\">An isometric cube whose near corner is cut away. ",
            "A small bright cube sits in the inner corner of the cut. Seen along the ",
            "diagonal, the solid forms three nested hexagons: a change at the center, ",
            "contained by the layers around it.</desc>\n"
        ),
        view = VIEW
    );
    let (body, change, edge) = palettes(mono);
    let _ = writeln!(
        out,
        concat!(
            "<g stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\" ",
            "stroke-opacity=\"{}\">"
        ),
        edge,
        EDGE,
        if mono { 0.9 } else { 0.55 }
    );
    let rim = OUTER - NOTCH;
    let shells = [
        // The convex body: three L-shaped faces at the outer surface.
        (
            Shell {
                level: OUTER,
                span: (0.0, OUTER),
                cut: Some((rim, OUTER)),
            },
            &body,
        ),
        // The concave notch: three L-shaped walls that face back towards the viewer.
        (
            Shell {
                level: rim,
                span: (rim, OUTER),
                cut: Some((rim, rim + CHANGE)),
            },
            &body,
        ),
        // The changed cube in the inner corner of the notch.
        (
            Shell {
                level: rim + CHANGE,
                span: (rim, rim + CHANGE),
                cut: None,
            },
            &change,
        ),
    ];
    for (shell, palette) in &shells {
        for axis in [Axis::Z, Axis::Y, Axis::X] {
            face(&mut out, axis, shell, palette);
        }
    }
    out.push_str("</g>\n</svg>\n");
    out
}

/// Returns the body palette, the change palette, and the edge color.
fn palettes(mono: bool) -> (Palette, Palette, String) {
    const INK: (u8, u8, u8) = (0x26, 0x20, 0x2e);
    const PURPLE: (u8, u8, u8) = (0x73, 0x41, 0xaa);
    const PINK: (u8, u8, u8) = (0xee, 0x77, 0xad);
    const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
    if mono {
        // One ink, three tints per solid. The change carries the strongest tones.
        let tint = |t: f64| hex(mix(WHITE, INK, t));
        let body = Palette {
            top: tint(0.10),
            left: tint(0.32),
            right: tint(0.54),
        };
        let change = Palette {
            top: tint(0.62),
            left: tint(0.84),
            right: tint(1.0),
        };
        return (body, change, hex(INK));
    }
    // Light comes from the upper left: tops are lightest, right faces darkest.
    let body = Palette {
        top: hex(mix(mix(PURPLE, PINK, 0.3), WHITE, 0.42)),
        left: hex(PURPLE),
        right: hex(mix(PURPLE, INK, 0.6)),
    };
    let change = Palette {
        top: hex(mix(PINK, WHITE, 0.6)),
        left: hex(PINK),
        right: hex(mix(PINK, PURPLE, 0.55)),
    };
    (body, change, hex(WHITE))
}

/// Writes the face of `shell` that lies in the plane of `axis` as an SVG polygon.
fn face(out: &mut String, axis: Axis, shell: &Shell, palette: &Palette) {
    let (lo, hi) = shell.span;
    let level = shell.level;
    let corners: Vec<(f64, f64)> = match shell.cut {
        None => vec![(lo, lo), (hi, lo), (hi, hi), (lo, hi)],
        // The cut square shares one corner with the face square.
        Some((a, _)) if a > lo => vec![(lo, lo), (hi, lo), (hi, a), (a, a), (a, hi), (lo, hi)],
        Some((_, b)) => vec![(b, lo), (hi, lo), (hi, hi), (lo, hi), (lo, b), (b, b)],
    };
    let fill = match axis {
        Axis::Z => &palette.top,
        Axis::Y => &palette.left,
        Axis::X => &palette.right,
    };
    out.push_str("<polygon points=\"");
    for (index, &(u, v)) in corners.iter().enumerate() {
        let point = match axis {
            Axis::X => (level, u, v),
            Axis::Y => (u, level, v),
            Axis::Z => (u, v, level),
        };
        let (x, y) = project(point);
        let separator = if index == 0 { "" } else { " " };
        let _ = write!(out, "{separator}{x:.3},{y:.3}");
    }
    let _ = writeln!(out, "\" fill=\"{fill}\"/>");
}

/// Maps a voxel-space point to viewBox coordinates.
///
/// The x axis runs down to the right, the y axis down to the left, and z straight up.
/// The center of the outer cube lands on the center of the viewBox.
fn project((x, y, z): (f64, f64, f64)) -> (f64, f64) {
    let unit = (VIEW - 2.0 * MARGIN) / (2.0 * OUTER);
    let half = OUTER / 2.0;
    let (x, y, z) = (x - half, y - half, z - half);
    let cos30 = 3.0_f64.sqrt() / 2.0;
    let across = (x - y) * cos30 * unit;
    let down = ((x + y) / 2.0 - z) * unit;
    (VIEW / 2.0 + across, VIEW / 2.0 + down)
}

/// Mixes color `a` towards color `b` by the fraction `t`.
fn mix(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let channel = |p: u8, q: u8| (f64::from(p) + (f64::from(q) - f64::from(p)) * t).round() as u8;
    (channel(a.0, b.0), channel(a.1, b.1), channel(a.2, b.2))
}

/// Formats a color as `#rrggbb`.
fn hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}
