//! The black-metal band logo generator (`scr_logo`), opened from the upgrade tree's LOGO button.
//!
//! Typed latin letters are laid out as calligraphic ribbons along a slight arch; thorns grow from
//! stroke tips and corners on the outline, drips hang from the bottom, a vertical axis crosses
//! the middle, and webs fill the outline and the strokes. Growth is animated by `logo_progress`
//! (letters first, then blades by their `birth`). Geometry is in "letter units" (a glyph is about
//! one unit tall); [`logo_paint`] scales it into a GUI rectangle.
//!
//! Pure Rust (no Bevy): the model and the triangle strips `logo_paint` emits are checked against
//! the LÖVE golden (`rust/golden/data/logo.json`). `ui::logo_screen` draws the strips and the
//! screen around them.

use myco_core::balance::vis;
use myco_core::gml::{
    angle_difference, clamp, dcos, dsin, frac, gmod, lengthdir_x, lengthdir_y, lerp,
    point_direction, point_distance, sign,
};

/// One glyph of `global.logo_glyphs`: width (in letter heights) and strokes of unit points.
pub struct LogoGlyph {
    pub w: f64,
    pub s: &'static [&'static [(f64, f64)]],
}

macro_rules! glyph {
    ($w:expr, [$([$(($x:expr, $y:expr)),* $(,)?]),* $(,)?]) => {
        LogoGlyph { w: $w, s: &[$(&[$(($x, $y)),*]),*] }
    };
}

static GLYPHS: [LogoGlyph; 26] = [
    glyph!(0.56, [[(0.05, 0.98), (0.5, 0.03), (0.95, 0.98)], [(0.22, 0.64), (0.78, 0.64)]]),
    glyph!(0.56, [
        [(0.2, 0.02), (0.2, 0.98)],
        [(0.2, 0.02), (0.66, 0.07), (0.82, 0.26), (0.6, 0.47), (0.2, 0.5)],
        [(0.2, 0.5), (0.7, 0.55), (0.88, 0.76), (0.62, 0.95), (0.2, 0.98)],
    ]),
    glyph!(0.56, [[(0.88, 0.13), (0.48, 0.02), (0.14, 0.3), (0.14, 0.7), (0.48, 0.98), (0.88, 0.87)]]),
    glyph!(0.58, [
        [(0.2, 0.02), (0.2, 0.98)],
        [(0.2, 0.02), (0.64, 0.12), (0.86, 0.5), (0.64, 0.88), (0.2, 0.98)],
    ]),
    glyph!(0.54, [
        [(0.86, 0.07), (0.48, 0.02), (0.18, 0.18), (0.18, 0.82), (0.48, 0.98), (0.86, 0.93)],
        [(0.18, 0.5), (0.66, 0.5)],
    ]),
    glyph!(0.52, [
        [(0.2, 0.02), (0.2, 0.98)],
        [(0.2, 0.02), (0.86, 0.08)],
        [(0.2, 0.5), (0.66, 0.5)],
    ]),
    glyph!(0.58, [[
        (0.88, 0.13), (0.48, 0.02), (0.14, 0.3), (0.14, 0.7), (0.48, 0.98), (0.86, 0.85),
        (0.86, 0.56), (0.54, 0.56)
    ]]),
    glyph!(0.58, [
        [(0.18, 0.02), (0.18, 0.98)],
        [(0.82, 0.02), (0.82, 0.98)],
        [(0.18, 0.5), (0.82, 0.5)],
    ]),
    glyph!(0.28, [
        [(0.5, 0.02), (0.5, 0.98)],
        [(0.08, 0.04), (0.92, 0.04)],
        [(0.08, 0.96), (0.92, 0.96)],
    ]),
    glyph!(0.48, [
        [(0.7, 0.02), (0.7, 0.75), (0.42, 0.98), (0.12, 0.83)],
        [(0.3, 0.04), (0.98, 0.04)],
    ]),
    glyph!(0.58, [
        [(0.18, 0.02), (0.18, 0.98)],
        [(0.9, 0.03), (0.18, 0.55)],
        [(0.36, 0.44), (0.94, 0.98)],
    ]),
    glyph!(0.52, [[(0.2, 0.02), (0.2, 0.96)], [(0.2, 0.96), (0.9, 0.96)]]),
    glyph!(0.78, [[(0.06, 0.98), (0.06, 0.03), (0.5, 0.68), (0.94, 0.03), (0.94, 0.98)]]),
    glyph!(0.62, [[(0.14, 0.98), (0.14, 0.03), (0.86, 0.97), (0.86, 0.02)]]),
    glyph!(0.58, [[(0.5, 0.02), (0.12, 0.5), (0.5, 0.98), (0.88, 0.5), (0.5, 0.02)]]),
    glyph!(0.54, [
        [(0.2, 0.02), (0.2, 0.98)],
        [(0.2, 0.02), (0.72, 0.08), (0.88, 0.3), (0.62, 0.5), (0.2, 0.53)],
    ]),
    glyph!(0.58, [
        [(0.5, 0.02), (0.12, 0.5), (0.5, 0.98), (0.88, 0.5), (0.5, 0.02)],
        [(0.62, 0.72), (1.0, 1.08)],
    ]),
    glyph!(0.58, [
        [(0.2, 0.02), (0.2, 0.98)],
        [(0.2, 0.02), (0.72, 0.08), (0.88, 0.3), (0.62, 0.5), (0.2, 0.53)],
        [(0.46, 0.53), (0.94, 0.98)],
    ]),
    glyph!(0.54, [[
        (0.86, 0.12), (0.46, 0.02), (0.14, 0.22), (0.5, 0.46), (0.86, 0.66), (0.56, 0.98),
        (0.12, 0.86)
    ]]),
    glyph!(0.56, [[(0.02, 0.04), (0.98, 0.04)], [(0.5, 0.04), (0.5, 0.98)]]),
    glyph!(0.58, [[(0.14, 0.02), (0.14, 0.72), (0.5, 0.98), (0.86, 0.72), (0.86, 0.02)]]),
    glyph!(0.58, [[(0.06, 0.02), (0.5, 0.98), (0.94, 0.02)]]),
    glyph!(0.84, [[(0.03, 0.02), (0.27, 0.98), (0.5, 0.34), (0.73, 0.98), (0.97, 0.02)]]),
    glyph!(0.58, [[(0.06, 0.02), (0.94, 0.98)], [(0.94, 0.02), (0.06, 0.98)]]),
    glyph!(0.58, [[(0.06, 0.02), (0.5, 0.52), (0.94, 0.02)], [(0.5, 0.52), (0.5, 0.98)]]),
    glyph!(0.54, [[(0.1, 0.04), (0.9, 0.04), (0.1, 0.96), (0.9, 0.96)]]),
];

/// `variable_struct_get(global.logo_glyphs, c)`: only `A`..`Z` exist (struct names are
/// case-sensitive).
pub fn logo_glyph(c: char) -> Option<&'static LogoGlyph> {
    c.is_ascii_uppercase()
        .then(|| &GLYPHS[(c as u8 - b'A') as usize])
}

/// A ribbon (`logo_make_path`): polyline with cumulative lengths and a pen nib width.
#[derive(Clone, Debug, PartialEq)]
pub struct LogoPath {
    pub pts: Vec<[f64; 2]>,
    pub len: Vec<f64>,
    pub total: f64,
    pub nib: f64,
    pub letter: f64,
    pub tip_start: bool,
    pub tip_end: bool,
}

/// A thorn/drip/web blade (`logo_push_blade`).
#[derive(Clone, Debug, PartialEq)]
pub struct LogoBlade {
    pub pts: Vec<[f64; 2]>,
    pub width: f64,
    pub root: f64,
    pub birth: f64,
    pub span: f64,
}

/// `logo_build`'s model (`___struct___155`).
#[derive(Clone, Debug, PartialEq)]
pub struct LogoModel {
    pub ribbons: Vec<LogoPath>,
    pub blades: Vec<LogoBlade>,
    pub letters: i64,
    pub duration: f64,
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
    pub letter_x1: f64,
    pub letter_x2: f64,
    pub letter_y1: f64,
    pub letter_y2: f64,
}

impl LogoModel {
    fn new() -> LogoModel {
        LogoModel {
            ribbons: Vec::new(),
            blades: Vec::new(),
            letters: 0,
            duration: 1.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            letter_x1: 0.0,
            letter_x2: 0.0,
            letter_y1: 0.0,
            letter_y2: 0.0,
        }
    }
}

/// `logo_path_at` result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathPoint {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
}

/// `logo_path_nearest` result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathNearest {
    pub t: f64,
    pub dist: f64,
}

/// `logo_anchor` result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub x: f64,
    pub y: f64,
    pub half: f64,
}

/// `logo_seed_points` item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeedPoint {
    pub x: f64,
    pub y: f64,
    pub t: f64,
    pub dir: f64,
}

/// `logo_outline` column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlineCol {
    pub x: f64,
    pub top: f64,
    pub bottom: f64,
    pub has: bool,
}

// port: logo_noise
/// Hash noise in 0..1.
pub fn logo_noise(a: f64, b: f64) -> f64 {
    frac(((a * 12.9898 + b * 78.233).sin() * 43758.5453).abs())
}

// port: logo_text_seed
/// Seed contribution of the text (`h = (h * 31 + ord(c)) mod 99991` from 7).
pub fn logo_text_seed(s: &str) -> f64 {
    let mut h = 7.0;
    for c in s.chars() {
        h = gmod(h * 31.0 + c as u32 as f64, 99991.0);
    }
    h
}

// port: logo_make_path
pub fn logo_make_path(pts: Vec<[f64; 2]>, nib: f64) -> LogoPath {
    let mut lens = vec![0.0];
    let mut total = 0.0;
    for i in 1..pts.len() {
        let (a, b) = (pts[i - 1], pts[i]);
        total += point_distance(a[0], a[1], b[0], b[1]);
        lens.push(total);
    }
    LogoPath {
        pts,
        len: lens,
        total: total.max(0.0001),
        nib,
        letter: 0.0,
        tip_start: true,
        tip_end: true,
    }
}

// port: logo_path_at
/// Point and heading at fraction `t` of the path length.
pub fn logo_path_at(path: &LogoPath, t: f64) -> PathPoint {
    let d = clamp(t, 0.0, 1.0) * path.total;
    let pts = &path.pts;
    let n = pts.len();
    for i in 1..n {
        if path.len[i] >= d || i == n - 1 {
            let seg = (path.len[i] - path.len[i - 1]).max(1e-05);
            let u = clamp((d - path.len[i - 1]) / seg, 0.0, 1.0);
            let ([x0, y0], [x1, y1]) = (pts[i - 1], pts[i]);
            return PathPoint {
                x: lerp(x0, x1, u),
                y: lerp(y0, y1, u),
                dir: point_direction(x0, y0, x1, y1),
            };
        }
    }
    PathPoint {
        x: pts[0][0],
        y: pts[0][1],
        dir: 90.0,
    }
}

// port: logo_path_nearest
pub fn logo_path_nearest(path: &LogoPath, px: f64, py: f64) -> PathNearest {
    let pts = &path.pts;
    let mut best = 999999.0;
    let mut best_t = 0.0;
    for i in 1..pts.len() {
        let ([x0, y0], [x1, y1]) = (pts[i - 1], pts[i]);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let l2 = (dx * dx + dy * dy).max(1e-05);
        let u = clamp(((px - x0) * dx + (py - y0) * dy) / l2, 0.0, 1.0);
        let d = point_distance(px, py, x0 + dx * u, y0 + dy * u);
        if d < best {
            best = d;
            best_t = (path.len[i - 1] + l2.sqrt() * u) / path.total;
        }
    }
    PathNearest {
        t: best_t,
        dist: best,
    }
}

// port: logo_half_width
/// Half the pen width at `t`: a broad nib held at `logo_pen_angle`.
pub fn logo_half_width(path: &LogoPath, t: f64) -> f64 {
    let p = logo_path_at(path, t);
    path.nib
        * 0.5
        * vis("logo_pen_contrast").max(dsin(p.dir - vis("logo_pen_angle")).abs())
}

// port: logo_reach
/// How far a growth heading `dir` may reach at horizontal position `side` (-1..1): wings out to
/// the sides, drips down, spikes up.
pub fn logo_reach(dir: f64, side: f64) -> f64 {
    let ctr = if side < 0.0 { 145.0 } else { 35.0 };
    let wing_c = dcos(angle_difference(dir, ctr)).max(0.0);
    let wing = wing_c.powf(2.5) * side.abs().powf(0.9);
    let drip_s = (-dsin(dir)).max(0.0);
    let drip = drip_s.powf(2.0) * (1.0 - side.abs() * 0.45);
    let top_s = dsin(dir).max(0.0);
    let top = top_s.powf(2.0) * side.abs();
    vis("logo_reach_base")
        + vis("logo_reach_wing") * wing
        + vis("logo_reach_drip") * drip
        + vis("logo_reach_top") * top
}

// port: logo_stretch
fn logo_stretch(m: &mut LogoModel, x: f64, y: f64) {
    m.x1 = m.x1.min(x);
    m.x2 = m.x2.max(x);
    m.y1 = m.y1.min(y);
    m.y2 = m.y2.max(y);
}

// port: logo_push_blade
fn logo_push_blade(m: &mut LogoModel, pts: Vec<[f64; 2]>, width: f64, root: f64, birth: f64, span: f64) {
    for p in &pts {
        logo_stretch(m, p[0], p[1]);
    }
    m.blades.push(LogoBlade {
        pts,
        width,
        root,
        birth,
        span,
    });
}

// port: logo_anchor
/// The nearest point on any ribbon to (x, y), with the pen half-width there.
pub fn logo_anchor(m: &LogoModel, x: f64, y: f64) -> Anchor {
    let mut a = Anchor {
        x,
        y,
        half: vis("logo_nib") * 0.5,
    };
    let mut best = 99999.0;
    for r in &m.ribbons {
        let near = logo_path_nearest(r, x, y);
        if near.dist < best {
            let p = logo_path_at(r, near.t);
            best = near.dist;
            a = Anchor {
                x: p.x,
                y: p.y,
                half: logo_half_width(r, near.t),
            };
        }
    }
    a
}

/// Shape parameters of one strand (`logo_grow_strand`'s arguments after `m`).
#[derive(Clone, Copy, Debug)]
struct Strand {
    x: f64,
    y: f64,
    dir: f64,
    len: f64,
    width: f64,
    root: f64,
    curl: f64,
    birth: f64,
    depth: f64,
    forks: f64,
    len_scale: f64,
    width_scale: f64,
    steps: f64,
    seed: f64,
}

// port: logo_grow_strand
/// A curling blade of `steps` segments, forking recursively `depth` times.
fn logo_grow_strand(m: &mut LogoModel, s: Strand) {
    if s.len < 0.015 || s.width < 0.0015 {
        return;
    }
    if (m.blades.len() as f64) >= vis("logo_max_blades") {
        return;
    }
    let jitter = vis("logo_strand_jitter");
    let mut pts = vec![[s.x, s.y]];
    let (mut cx, mut cy, mut d) = (s.x, s.y, s.dir);
    let mut i = 0.0;
    while i < s.steps {
        d += s.curl + (logo_noise(s.seed + i, 4.1) - 0.5) * jitter;
        cx += lengthdir_x(s.len / s.steps, d);
        cy += lengthdir_y(s.len / s.steps, d);
        pts.push([cx, cy]);
        i += 1.0;
    }
    logo_push_blade(m, pts.clone(), s.width, s.root, s.birth, 1.0);
    if s.depth > 0.0 {
        let mut k = 0.0;
        while k < s.forks {
            let f = (k + 1.0) / (s.forks + 1.0) * 0.85 + logo_noise(s.seed + k, 2.2) * 0.12;
            let idx = clamp((f * s.steps).floor(), 1.0, s.steps - 1.0);
            let ft = idx / s.steps;
            let ii = idx as usize;
            let [px, py] = pts[ii];
            let base_dir = point_direction(pts[ii - 1][0], pts[ii - 1][1], px, py);
            let side = if gmod(k, 2.0) == 0.0 { 1.0 } else { -1.0 };
            let w = s.width * (1.0 - ft).powf(1.35);
            logo_grow_strand(
                m,
                Strand {
                    x: px,
                    y: py,
                    dir: base_dir + side * (26.0 + logo_noise(s.seed + k, 3.3) * 20.0),
                    len: s.len * s.len_scale * (1.25 - ft),
                    width: w * s.width_scale,
                    root: w * 0.5,
                    curl: s.curl * side * 0.8,
                    birth: s.birth + 0.18,
                    depth: s.depth - 1.0,
                    forks: s.forks,
                    len_scale: s.len_scale,
                    width_scale: s.width_scale,
                    steps: (s.steps - 3.0).max(4.0),
                    seed: s.seed + 137.0 + k * 19.0,
                },
            );
            k += 1.0;
        }
    }
}

/// A plain strand without forks (drips, the axis and its feet).
#[allow(clippy::too_many_arguments)]
fn plain_strand(
    m: &mut LogoModel,
    x: f64,
    y: f64,
    dir: f64,
    len: f64,
    width: f64,
    root: f64,
    curl: f64,
    birth: f64,
    steps: f64,
    seed: f64,
) {
    logo_grow_strand(
        m,
        Strand {
            x,
            y,
            dir,
            len,
            width,
            root,
            curl,
            birth,
            depth: 0.0,
            forks: 0.0,
            len_scale: 0.5,
            width_scale: 0.5,
            steps,
            seed,
        },
    );
}

// port: logo_grow_thorn
#[allow(clippy::too_many_arguments)]
fn logo_grow_thorn(
    m: &mut LogoModel,
    x: f64,
    y: f64,
    dir: f64,
    len: f64,
    width: f64,
    root: f64,
    curl: f64,
    birth: f64,
    seed: f64,
) {
    logo_grow_strand(
        m,
        Strand {
            x,
            y,
            dir,
            len,
            width,
            root,
            curl,
            birth,
            depth: vis("logo_thorn_depth"),
            forks: vis("logo_thorn_forks"),
            len_scale: 0.55,
            width_scale: 0.6,
            steps: 12.0,
            seed,
        },
    );
}

// port: logo_grow_web
#[allow(clippy::too_many_arguments)]
fn logo_grow_web(
    m: &mut LogoModel,
    x: f64,
    y: f64,
    dir: f64,
    len: f64,
    width: f64,
    root: f64,
    curl: f64,
    birth: f64,
    seed: f64,
) {
    logo_grow_strand(
        m,
        Strand {
            x,
            y,
            dir,
            len,
            width,
            root,
            curl,
            birth,
            depth: vis("logo_web_depth"),
            forks: vis("logo_web_forks"),
            len_scale: 0.62,
            width_scale: 0.62,
            steps: 6.0,
            seed,
        },
    );
}

// port: logo_seed_points
/// Where thorns may start on a ribbon: free tips (pointing outwards) and sharp corners with
/// long enough legs (pointing away from the turn).
pub fn logo_seed_points(path: &LogoPath) -> Vec<SeedPoint> {
    let mut out = Vec::new();
    let pts = &path.pts;
    let n = pts.len();
    if path.tip_start {
        let (p0, p1) = (pts[0], pts[1]);
        out.push(SeedPoint {
            x: p0[0],
            y: p0[1],
            t: 0.0,
            dir: point_direction(p1[0], p1[1], p0[0], p0[1]),
        });
    }
    if path.tip_end {
        let (pa, pb) = (pts[n - 2], pts[n - 1]);
        out.push(SeedPoint {
            x: pb[0],
            y: pb[1],
            t: 1.0,
            dir: point_direction(pa[0], pa[1], pb[0], pb[1]),
        });
    }
    let gap = path.total * vis("logo_corner_leg");
    for i in 1..n.saturating_sub(1) {
        let (prev, cur, nxt) = (pts[i - 1], pts[i], pts[i + 1]);
        let din = point_direction(prev[0], prev[1], cur[0], cur[1]);
        let dout = point_direction(cur[0], cur[1], nxt[0], nxt[1]);
        let turn = angle_difference(dout, din);
        let skip = turn.abs() < vis("logo_corner_turn")
            || path.len[i] - path.len[i - 1] < gap
            || path.len[i + 1] - path.len[i] < gap;
        if !skip {
            let bisect = din + turn * 0.5;
            out.push(SeedPoint {
                x: cur[0],
                y: cur[1],
                t: path.len[i] / path.total,
                dir: bisect - sign(turn) * 90.0,
            });
        }
    }
    out
}

// port: logo_outline
/// Top and bottom of the lettering in `n` columns across the letters' width.
pub fn logo_outline(m: &LogoModel, n: f64) -> Vec<OutlineCol> {
    let (x1, x2) = (m.letter_x1, m.letter_x2);
    let span = (x2 - x1).max(0.0001);
    // `while i < n`: a fractional n gets ceil(n) columns
    let cols_n = n.max(0.0).ceil() as usize;
    let mut cols: Vec<OutlineCol> = (0..cols_n)
        .map(|i| OutlineCol {
            x: x1 + span * (i as f64 + 0.5) / n,
            top: 9999.0,
            bottom: -9999.0,
            has: false,
        })
        .collect();
    for rb in &m.ribbons {
        let steps = 14.0;
        let mut s = 0.0;
        while s <= steps {
            let p = logo_path_at(rb, s / steps);
            let hw = logo_half_width(rb, s / steps);
            let ci = clamp(((p.x - x1) / span * n).floor(), 0.0, n - 1.0) as usize;
            let c = &mut cols[ci];
            c.top = c.top.min(p.y - hw);
            c.bottom = c.bottom.max(p.y + hw);
            c.has = true;
            s += 1.0;
        }
    }
    cols
}

struct Span {
    left: [f64; 2],
    right: [f64; 2],
}

struct ThornSeed {
    x: f64,
    y: f64,
    dir: f64,
    width: f64,
    half: f64,
    key: f64,
}

// port: logo_build
/// Build the logo model for `text` (letters without a glyph are dropped; spaces advance).
pub fn logo_build(text: &str, seed: f64) -> LogoModel {
    let mut m = LogoModel::new();
    let chars: Vec<char> = text
        .chars()
        .filter(|&c| c == ' ' || logo_glyph(c).is_some())
        .collect();
    let count = chars.iter().filter(|&&c| c != ' ').count() as f64;
    if count == 0.0 {
        return m;
    }
    let arch = vis("logo_arch");
    let tilt = vis("logo_tilt");
    let spread = vis("logo_size_spread");
    let advance = vis("logo_letter_advance");
    let nib = vis("logo_nib");
    let mut pen = 0.0;
    let mut spans: Vec<Span> = Vec::new();
    let mut first = true;
    for &c in &chars {
        let li = m.letters as f64;
        let t = if count > 1.0 {
            li / (count - 1.0) * 2.0 - 1.0
        } else {
            0.0
        };
        let size = 1.0 + spread * t.abs().powf(1.5);
        let yoff = -arch * t * t;
        let rot = -tilt * t;
        if c == ' ' {
            pen += 0.3;
            continue;
        }
        let g = logo_glyph(c).expect("filtered");
        let gw = g.w * vis("logo_glyph_width");
        m.letters += 1;
        let (cr, sr) = (dcos(rot), dsin(rot));
        let r0 = m.ribbons.len();
        let (mut minx, mut maxx) = (9999.0, -9999.0);
        let (mut lp, mut rp) = ([0.0, 0.0], [0.0, 0.0]);
        for stroke in g.s {
            let mut pts = Vec::with_capacity(stroke.len());
            for &(sx, sy) in stroke.iter() {
                let lx = sx * gw * size;
                let ly = (sy - 0.5) * size;
                let px = pen + lx * cr + ly * sr;
                let py = yoff - lx * sr + ly * cr;
                pts.push([px, py]);
                if first {
                    m.x1 = px;
                    m.x2 = px;
                    m.y1 = py;
                    m.y2 = py;
                    first = false;
                }
                logo_stretch(&mut m, px, py);
                if px < minx {
                    minx = px;
                    lp = [px, py];
                }
                if px > maxx {
                    maxx = px;
                    rp = [px, py];
                }
            }
            let mut path = logo_make_path(pts, nib * size);
            path.letter = li;
            m.ribbons.push(path);
        }
        // stroke ends touching another stroke of the same letter are not free tips
        for a in r0..m.ribbons.len() {
            for e in 0..2 {
                let ep = if e == 0 {
                    m.ribbons[a].pts[0]
                } else {
                    *m.ribbons[a].pts.last().expect("stroke")
                };
                for b in r0..m.ribbons.len() {
                    if b != a && logo_path_nearest(&m.ribbons[b], ep[0], ep[1]).dist < 0.09 * size {
                        if e == 0 {
                            m.ribbons[a].tip_start = false;
                        } else {
                            m.ribbons[a].tip_end = false;
                        }
                        break;
                    }
                }
            }
        }
        spans.push(Span {
            left: lp,
            right: rp,
        });
        pen += gw * size * advance;
    }
    m.letter_x1 = m.x1;
    m.letter_x2 = m.x2;
    m.letter_y1 = m.y1;
    m.letter_y2 = m.y2;
    if vis("logo_bridges") > 0.0 {
        for i in 1..spans.len() {
            let (a, b) = (spans[i - 1].right, spans[i].left);
            let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5 - 0.06];
            let mut br = logo_make_path(vec![a, mid, b], nib * 0.8);
            br.letter = i as f64;
            br.tip_start = false;
            br.tip_end = false;
            m.ribbons.push(br);
        }
    }
    let tail = m.letters as f64 * 0.5;
    let cols_n = vis("logo_outline_columns");
    let cols = logo_outline(&m, cols_n);
    let lx1 = m.letter_x1;
    let lw = (m.letter_x2 - lx1).max(0.0001);
    let cx = (m.letter_x1 + m.letter_x2) * 0.5;
    let cy = (m.letter_y1 + m.letter_y2) * 0.5;
    let hw = (lw * 0.5).max(0.0001);
    let tol = vis("logo_nib") * vis("logo_edge_tolerance");
    let flare = vis("logo_blade_flare");
    let thorn_w = vis("logo_thorn_width");
    let curl = vis("logo_thorn_curl");

    // thorns from tips and corners on the outline (or in the end columns)
    let mut seeds: Vec<ThornSeed> = Vec::new();
    for (ri, rb) in m.ribbons.iter().enumerate() {
        for (k, s) in logo_seed_points(rb).iter().enumerate() {
            let col = clamp(((s.x - lx1) / lw * cols_n).floor(), 0.0, cols_n - 1.0);
            let cc = &cols[col as usize];
            let skip = !cc.has
                || (s.y > cc.top + tol
                    && s.y < cc.bottom - tol
                    && col != 0.0
                    && col != cols_n - 1.0);
            if !skip {
                let half = logo_half_width(rb, s.t);
                seeds.push(ThornSeed {
                    x: s.x,
                    y: s.y,
                    dir: s.dir,
                    width: half * 2.0 * thorn_w,
                    half,
                    key: seed + ri as f64 * 29.0 + k as f64 * 7.0,
                });
            }
        }
    }
    let (mut left_i, mut right_i): (Option<usize>, Option<usize>) = (None, None);
    for (i, s) in seeds.iter().enumerate() {
        let side = clamp((s.x - cx) / hw, -1.0, 1.0);
        let out_dir = point_direction(cx, cy, s.x, s.y);
        let bd = s.dir + angle_difference(out_dir, s.dir) * flare;
        let len = logo_reach(bd, side) * (0.75 + logo_noise(s.key, 2.3) * 0.5);
        let tcurl = if s.x < cx { curl } else { -curl };
        logo_grow_thorn(
            &mut m,
            s.x,
            s.y,
            bd,
            len,
            s.width,
            s.half,
            tcurl,
            tail + 0.4 + side.abs() * 0.4,
            s.key,
        );
        if left_i.is_none_or(|l| s.x < seeds[l].x) {
            left_i = Some(i);
        }
        if right_i.is_none_or(|r| s.x > seeds[r].x) {
            right_i = Some(i);
        }
    }
    // the two outermost seeds get a long horn
    for e in 0..2 {
        let idx = if e == 0 { left_i } else { right_i };
        if let Some(idx) = idx {
            let s = &seeds[idx];
            let side = if e == 0 { -1.0 } else { 1.0 };
            let dir = 90.0 - side * 38.0;
            logo_grow_thorn(
                &mut m,
                s.x,
                s.y,
                dir,
                logo_reach(dir, side) * 1.4,
                s.width,
                s.half,
                -side * curl,
                tail + 0.8,
                seed + 501.0,
            );
        }
    }
    // drips under the columns
    let drip_w = vis("logo_nib") * 0.45;
    for (i, cc) in cols.iter().enumerate() {
        if !cc.has {
            continue;
        }
        let fi = i as f64;
        let t = if cols_n > 1.0 {
            fi / (cols_n - 1.0) * 2.0 - 1.0
        } else {
            0.0
        };
        let dk = seed + fi.min(cols_n - 1.0 - fi) * 37.0;
        if logo_noise(dk, 6.1) > vis("logo_drip_chance") {
            continue;
        }
        let dd = 270.0 + t * 10.0 + (logo_noise(dk, 4.7) - 0.5) * 8.0;
        let an = logo_anchor(&m, cc.x, cc.bottom);
        let dlen = logo_reach(dd, t) * (0.6 + logo_noise(dk, 5.9) * 0.9);
        let dcurl = (logo_noise(dk, 8.2) - 0.5) * 3.0;
        plain_strand(&mut m, an.x, an.y, dd, dlen, drip_w, an.half, dcurl, tail + 0.9, 12.0, dk + 71.0);
    }
    // the vertical axis through the middle column, with a cross foot
    let mc = cols[(cols_n / 2.0).trunc() as usize];
    let axis_t = tail + 1.2;
    let axis_reach = vis("logo_axis_reach");
    if mc.has {
        let nib_v = vis("logo_nib");
        let top_a = logo_anchor(&m, cx, mc.top);
        let bot_a = logo_anchor(&m, cx, mc.bottom);
        logo_grow_thorn(
            &mut m,
            top_a.x,
            top_a.y,
            90.0,
            axis_reach * 0.8,
            nib_v * 0.5,
            top_a.half,
            0.0,
            axis_t,
            seed + 11.0,
        );
        plain_strand(
            &mut m,
            bot_a.x,
            bot_a.y,
            270.0,
            axis_reach,
            nib_v * 0.55,
            bot_a.half,
            0.0,
            axis_t,
            12.0,
            seed + 12.0,
        );
        let foot_y = bot_a.y + axis_reach * 0.62;
        let foot_w = nib_v * 0.55 * 0.27083691323011094 * 0.5;
        for dir in [180.0, 0.0] {
            plain_strand(
                &mut m,
                bot_a.x,
                foot_y,
                dir,
                axis_reach * 0.34,
                nib_v * 0.34,
                foot_w,
                0.0,
                axis_t + 0.3,
                8.0,
                seed + 13.0,
            );
        }
    }
    // webs off the outline, above and below
    let web_n = clamp(m.letters as f64 * vis("logo_web_per_letter"), 16.0, 40.0);
    let wcols = logo_outline(&m, web_n);
    let web_w = vis("logo_nib") * vis("logo_web_width");
    let web_len = vis("logo_web_length");
    let web_curl = vis("logo_web_curl");
    for (i, cc) in wcols.iter().enumerate() {
        if !cc.has {
            continue;
        }
        let fi = i as f64;
        let t = if web_n > 1.0 {
            fi / (web_n - 1.0) * 2.0 - 1.0
        } else {
            0.0
        };
        let wk = seed + fi.min(web_n - 1.0 - fi) * 53.0 + 300.0;
        let wb = tail + 0.5 + t.abs() * 0.6;
        for e2 in 0..2 {
            let e2f = e2 as f64;
            let top = e2 == 0;
            let ey = if top { cc.top } else { cc.bottom };
            let base = if top { 90.0 } else { 270.0 };
            let sw = (logo_noise(wk + e2f * 11.0, 1.4) - 0.5) * 44.0 - t * 26.0;
            let dd = if top { base + sw } else { base + -sw };
            let cs = if logo_noise(wk + e2f * 7.0, 3.8) < 0.5 {
                -web_curl
            } else {
                web_curl
            };
            let an = logo_anchor(&m, cc.x, ey);
            let n1 = logo_noise(wk + e2f, 5.2);
            let wlen = logo_reach(dd, t) * web_len * (0.6 + n1 * 0.8);
            let n2 = logo_noise(wk + e2f, 6.4);
            logo_grow_web(
                &mut m,
                an.x,
                an.y,
                dd,
                wlen,
                web_w * (0.7 + n2 * 0.6),
                an.half,
                cs,
                wb + e2f * 0.15,
                wk + e2f * 3.0,
            );
        }
    }
    // short webs along every stroke, alternating sides
    let inner_len = vis("logo_web_inner_length");
    let inner_density = vis("logo_web_inner_density");
    for ri in 0..m.ribbons.len() {
        let cnt = (m.ribbons[ri].total * inner_density).ceil();
        let mut j = 0.0;
        while j < cnt {
            let ik = seed + ri as f64 * 71.0 + j * 11.0 + 700.0;
            let u = (j + 0.5) / cnt + (logo_noise(ik, 1.6) - 0.5) * 0.1;
            let rb = &m.ribbons[ri];
            let p = logo_path_at(rb, clamp(u, 0.03, 0.97));
            let phw = logo_half_width(rb, clamp(u, 0.03, 0.97));
            let sd = if gmod(j, 2.0) == 0.0 { 1.0 } else { -1.0 };
            let dd = p.dir + sd * (34.0 + logo_noise(ik, 2.8) * 46.0);
            let ilen = inner_len * (0.6 + logo_noise(ik, 4.2) * 0.9);
            let iw = web_w * (0.6 + logo_noise(ik, 5.4) * 0.5);
            let ib = tail + 0.45 + logo_noise(ik, 6.8) * 0.7;
            logo_grow_web(&mut m, p.x, p.y, dd, ilen, iw, phw, sd * web_curl, ib, ik);
            j += 1.0;
        }
    }
    m.duration = tail + 3.0;
    m
}

/// One `draw_primitive_begin(pr_trianglestrip)` … `draw_primitive_end` run, GUI coordinates.
pub type Strip = Vec<[f64; 2]>;

// port: logo_draw_ribbon
/// A ribbon drawn with a broad nib, grown to fraction `p` (tapering at the growing end).
pub fn logo_draw_ribbon(path: &LogoPath, ox: f64, oy: f64, scale: f64, p: f64, out: &mut Vec<Strip>) {
    if p <= 0.0 {
        return;
    }
    let segs = clamp((path.total * scale / 7.0).ceil(), 8.0, 48.0);
    let ang = vis("logo_pen_angle");
    let hn = path.nib * 0.5 * scale;
    let minpx = vis("logo_min_pixels");
    let mut strip = Vec::new();
    let mut i = 0.0;
    while i <= segs {
        let t = i / segs * p;
        let pt = logo_path_at(path, t);
        let taper = if p >= 1.0 {
            1.0
        } else {
            clamp((p - t) / 0.06, 0.0, 1.0)
        };
        let h = hn * taper;
        let mut dx = lengthdir_x(h, ang);
        let mut dy = lengthdir_y(h, ang);
        let thick = h * dsin(pt.dir - ang).abs();
        if thick < minpx {
            dx += lengthdir_x(minpx - thick, pt.dir + 90.0);
            dy += lengthdir_y(minpx - thick, pt.dir + 90.0);
        }
        let px = ox + pt.x * scale;
        let py = oy + pt.y * scale;
        strip.push([px + dx, py + dy]);
        strip.push([px - dx, py - dy]);
        i += 1.0;
    }
    out.push(strip);
}

// port: logo_draw_blade
/// A blade grown to fraction `prog`: tapering width with a collar at the root.
pub fn logo_draw_blade(blade: &LogoBlade, ox: f64, oy: f64, scale: f64, prog: f64, out: &mut Vec<Strip>) {
    if prog <= 0.0 || prog.is_nan() {
        return;
    }
    let pts = &blade.pts;
    let n = pts.len();
    let nf = n as f64;
    let lim = 1.0 + prog * (nf - 1.0);
    let run = vis("logo_collar_run");
    let collar = vis("logo_collar");
    let minpx = vis("logo_min_pixels");
    let mut strip = Vec::new();
    for i in 0..n {
        let fi = i as f64;
        if fi > lim {
            break;
        }
        let t = fi / (nf - 1.0);
        let x = ox + pts[i][0] * scale;
        let y = oy + pts[i][1] * scale;
        let lo = i.saturating_sub(1);
        let hi = (n - 1).min(i + 1);
        let dir = point_direction(pts[lo][0], pts[lo][1], pts[hi][0], pts[hi][1]);
        let w1 = blade.width * 0.5 * (1.0 - t).powf(1.35);
        let w2 = blade.root * collar * (1.0 - (t / run).min(1.0)).powf(2.0);
        let mut w = w1.max(w2) * scale;
        if prog < 1.0 {
            w *= clamp(lim - fi, 0.0, 1.0);
        }
        w = w.max(minpx);
        strip.push([x + lengthdir_x(w, dir + 90.0), y + lengthdir_y(w, dir + 90.0)]);
        strip.push([x + lengthdir_x(w, dir - 90.0), y + lengthdir_y(w, dir - 90.0)]);
    }
    out.push(strip);
}

// port: logo_paint
/// Every blade then every ribbon of `m` at growth `progress`, fitted into the `w×h` box centred
/// on (cx, cy). Each strip is one triangle strip.
pub fn logo_paint(m: &LogoModel, progress: f64, cx: f64, cy: f64, w: f64, h: f64) -> Vec<Strip> {
    let mw = (m.x2 - m.x1).max(0.6) + 0.2;
    let mh = (m.y2 - m.y1).max(0.6) + 0.2;
    let s = (w / mw).min(h / mh);
    let ox = cx - (m.x1 + m.x2) * 0.5 * s;
    let oy = cy - (m.y1 + m.y2) * 0.5 * s;
    let mut out = Vec::new();
    for b in &m.blades {
        logo_draw_blade(b, ox, oy, s, clamp((progress - b.birth) / b.span, 0.0, 1.0), &mut out);
    }
    for r in &m.ribbons {
        logo_draw_ribbon(r, ox, oy, s, clamp((progress - r.letter * 0.5) / 0.9, 0.0, 1.0), &mut out);
    }
    out
}

/// `global.logo_phase`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogoPhase {
    Type,
    Grow,
}

/// The `global.logo_*` state, plus the cached finished drawing standing in for `logo_surface`.
#[derive(Clone, Debug, PartialEq)]
pub struct LogoState {
    pub open: bool,
    pub phase: LogoPhase,
    pub text: String,
    pub seed: f64,
    pub progress: f64,
    pub model: Option<LogoModel>,
    /// `global.logo_surface`: the finished logo drawn once (`None` = no surface), keyed by the
    /// box it was painted into
    pub surface: Option<([f64; 4], Vec<Strip>)>,
}

impl Default for LogoState {
    fn default() -> Self {
        LogoState::logo_init()
    }
}

impl LogoState {
    // port: logo_init
    /// `logo_init` (the glyph table is the static [`logo_glyph`]).
    pub fn logo_init() -> LogoState {
        LogoState {
            open: false,
            phase: LogoPhase::Type,
            text: String::new(),
            seed: 1.0,
            progress: 0.0,
            model: None,
            surface: None,
        }
    }

    // port: logo_drop_surface
    pub fn logo_drop_surface(&mut self) {
        self.surface = None;
    }

    // port: logo_read_typing
    /// Keep only upper-cased latin letters and spaces of `keyboard_string`, at most 18; both the
    /// keyboard string and the logo text become the result.
    pub fn logo_read_typing(&mut self, keyboard_string: &mut String) {
        let mut s: String = keyboard_string
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .filter(|&c| c.is_ascii_uppercase() || c == ' ')
            .collect();
        s.truncate(18);
        *keyboard_string = s.clone();
        self.text = s;
    }

    // port: logo_has_letters
    pub fn logo_has_letters(&self) -> bool {
        self.text.chars().any(|c| c != ' ')
    }

    // port: logo_grow_start
    pub fn logo_grow_start(&mut self) {
        self.logo_drop_surface();
        let m = logo_build(&self.text, self.seed + logo_text_seed(&self.text));
        let letters = m.letters;
        self.model = Some(m);
        if letters == 0 {
            return;
        }
        self.phase = LogoPhase::Grow;
        self.progress = 0.0;
    }

    // port: logo_edit
    pub fn logo_edit(&mut self, keyboard_string: &mut String) {
        self.logo_drop_surface();
        self.phase = LogoPhase::Type;
        *keyboard_string = self.text.clone();
    }

    // port: logo_reroll
    pub fn logo_reroll(&mut self) {
        self.seed += 1.0;
        self.logo_grow_start();
    }

    // port: logo_open
    pub fn logo_open(&mut self, keyboard_string: &mut String) {
        self.logo_drop_surface();
        self.open = true;
        self.phase = LogoPhase::Type;
        *keyboard_string = self.text.clone();
    }

    // port: logo_step
    /// Typing: read the keyboard, Enter grows. Growing: advance over `logo_grow_frames`.
    pub fn logo_step(&mut self, keyboard_string: &mut String, enter_pressed: bool) {
        if self.phase == LogoPhase::Type {
            self.logo_read_typing(keyboard_string);
            if enter_pressed && self.logo_has_letters() {
                self.logo_grow_start();
            }
        } else {
            let dur = self.model.as_ref().map_or(1.0, |m| m.duration);
            self.progress = dur.min(self.progress + dur / vis("logo_grow_frames").max(1.0));
        }
    }

    // port: logo_draw
    /// The strips to draw this frame: painted live while growing, then painted once and kept
    /// (the original's GUI-sized surface).
    pub fn logo_draw(&mut self, cx: f64, cy: f64, w: f64, h: f64) -> Vec<Strip> {
        let Some(m) = &self.model else {
            return Vec::new();
        };
        if m.letters == 0 {
            return Vec::new();
        }
        if self.progress < m.duration {
            let strips = logo_paint(m, self.progress, cx, cy, w, h);
            self.logo_drop_surface();
            return strips;
        }
        let key = [cx, cy, w, h];
        match &self.surface {
            Some((k, strips)) if *k == key => strips.clone(),
            _ => {
                let strips = logo_paint(m, self.progress, cx, cy, w, h);
                self.surface = Some((key, strips.clone()));
                strips
            }
        }
    }
}
