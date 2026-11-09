//! GameMaker sprite drawing in Bevy terms, shared by the render tasks.
//!
//! [`SpriteCmd`] is one `draw_sprite_ext` / `draw_sprite_part_ext` call in world coordinates (y
//! down, as the original). [`SpriteCmd::to_bevy`] turns it into a Bevy `Sprite` + `Anchor` +
//! `Transform`, with the same trimming rules as the LÖVE port (`game/gm/draw.lua`): a part is cut
//! from the untrimmed image and the origin is ignored; `_ext` places the origin at (x, y),
//! scales, then rotates counter-clockwise by `rot` degrees.

use crate::assets::{GameAssets, SpriteDef};
use crate::camera::world_to_bevy;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// GameMaker colour (BGR integer, `c_white` = 16777215) and alpha as a Bevy colour.
pub fn gm_colour(col: f64, alpha: f64) -> Color {
    let c = col.max(0.0) as u32;
    Color::srgba_u8(
        (c & 255) as u8,
        ((c >> 8) & 255) as u8,
        ((c >> 16) & 255) as u8,
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// Same as [`gm_colour`] with float alpha (no 8-bit rounding).
pub fn gm_colour_f(col: f64, alpha: f64) -> Color {
    let c = col.max(0.0) as u32;
    Color::srgba(
        (c & 255) as f32 / 255.0,
        ((c >> 8) & 255) as f32 / 255.0,
        ((c >> 16) & 255) as f32 / 255.0,
        alpha.clamp(0.0, 1.0) as f32,
    )
}

pub const C_WHITE: f64 = 16777215.0;

/// `part_quad` of the LÖVE port: the part `[left, top, w, h]` of the untrimmed image clipped to
/// the trimmed frame at `trim` (`[x, y, w, h]` inside the untrimmed box). Returns the clipped
/// rectangle `[x1, y1, x2, y2]` in untrimmed coordinates, or `None` when nothing is left.
pub fn part_quad(trim: [f64; 4], left: f64, top: f64, w: f64, h: f64) -> Option<[f64; 4]> {
    let [fx, fy, fw, fh] = trim;
    let (x1, y1) = (left.max(fx), top.max(fy));
    let (x2, y2) = ((left + w).min(fx + fw), (top + h).min(fy + fh));
    if x2 <= x1 || y2 <= y1 {
        return None;
    }
    Some([x1, y1, x2, y2])
}

/// What a [`SpriteCmd`] draws.
#[derive(Clone, Debug, PartialEq)]
pub enum SpriteSrc {
    /// `draw_sprite_ext(sprite, frame, ...)`: whole frame, origin at (x, y)
    Ext { sprite: String, frame: usize },
    /// `draw_sprite_part_ext(sprite, frame, left, top, w, h, ...)`: (x, y) is the part's
    /// top-left corner
    Part {
        sprite: String,
        frame: usize,
        left: f64,
        top: f64,
        w: f64,
        h: f64,
    },
}

/// One sprite draw call in world coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteCmd {
    pub src: SpriteSrc,
    pub x: f64,
    pub y: f64,
    pub xs: f64,
    pub ys: f64,
    /// degrees, counter-clockwise on screen (GameMaker)
    pub rot: f64,
    /// BGR colour
    pub col: f64,
    pub alpha: f64,
    pub z: f32,
}

impl SpriteCmd {
    #[allow(clippy::too_many_arguments)]
    pub fn ext(sprite: &str, frame: usize, x: f64, y: f64, xs: f64, ys: f64, rot: f64) -> Self {
        SpriteCmd {
            src: SpriteSrc::Ext {
                sprite: sprite.to_owned(),
                frame,
            },
            x,
            y,
            xs,
            ys,
            rot,
            col: C_WHITE,
            alpha: 1.0,
            z: 0.0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn part(
        sprite: &str,
        left: f64,
        top: f64,
        w: f64,
        h: f64,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
    ) -> Self {
        SpriteCmd {
            src: SpriteSrc::Part {
                sprite: sprite.to_owned(),
                frame: 0,
                left,
                top,
                w,
                h,
            },
            x,
            y,
            xs,
            ys,
            rot: 0.0,
            col: C_WHITE,
            alpha: 1.0,
            z: 0.0,
        }
    }

    pub fn at_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn sprite_name(&self) -> &str {
        match &self.src {
            SpriteSrc::Ext { sprite, .. } | SpriteSrc::Part { sprite, .. } => sprite,
        }
    }

    /// Bevy components for this call; `None` for an unknown sprite or an empty part.
    pub fn to_bevy(&self, assets: &GameAssets) -> Option<(Sprite, Anchor, Transform)> {
        let def = assets.try_sprite(self.sprite_name())?;
        let (mut sprite, anchor, x, y) = match &self.src {
            SpriteSrc::Ext { frame, .. } => {
                let (s, a) = def.bevy_sprite(*frame);
                (s, a, self.x, self.y)
            }
            SpriteSrc::Part {
                frame,
                left,
                top,
                w,
                h,
                ..
            } => {
                let (s, dx, dy) = part_sprite(def, *frame, *left, *top, *w, *h)?;
                (
                    s,
                    Anchor::TOP_LEFT,
                    self.x + dx * self.xs,
                    self.y + dy * self.ys,
                )
            }
        };
        sprite.color = gm_colour_f(self.col, self.alpha);
        let tf = Transform {
            translation: world_to_bevy(x, y).extend(self.z),
            rotation: Quat::from_rotation_z((self.rot as f32).to_radians()),
            scale: Vec3::new(self.xs as f32, self.ys as f32, 1.0),
        };
        Some((sprite, anchor, tf))
    }
}

/// The part `[left, top, w, h]` of a sprite frame as a Bevy sprite anchored at its top-left,
/// plus the offset `(dx, dy)` of that corner from the requested part's corner (the part may be
/// clipped by the frame's trim).
pub fn part_sprite(
    def: &SpriteDef,
    frame: usize,
    left: f64,
    top: f64,
    w: f64,
    h: f64,
) -> Option<(Sprite, f64, f64)> {
    let fr = def.frames.get(frame % def.frames.len().max(1))?;
    let size = fr.rect.size();
    let trim = [
        fr.trim_offset.x as f64,
        fr.trim_offset.y as f64,
        size.x as f64,
        size.y as f64,
    ];
    let [x1, y1, x2, y2] = part_quad(trim, left, top, w, h)?;
    let min = fr.rect.min + Vec2::new((x1 - trim[0]) as f32, (y1 - trim[1]) as f32);
    let rect = Rect::from_corners(min, min + Vec2::new((x2 - x1) as f32, (y2 - y1) as f32));
    let sprite = Sprite {
        image: fr.page.clone(),
        rect: Some(rect),
        ..default()
    };
    Some((sprite, x1 - left, y1 - top))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_quad_clips_to_the_trimmed_frame_like_love() {
        // bush_tree: 789x222, trimmed frame at (0, 25) size 788x197
        let trim = [0.0, 25.0, 788.0, 197.0];
        assert_eq!(
            part_quad(trim, 10.0, 0.0, 100.0, 100.0),
            Some([10.0, 25.0, 110.0, 100.0])
        );
        assert_eq!(part_quad(trim, 10.0, 0.0, 100.0, 25.0), None);
        assert_eq!(
            part_quad(trim, 700.0, 200.0, 200.0, 50.0),
            Some([700.0, 200.0, 788.0, 222.0])
        );
    }

    #[test]
    fn gm_colour_is_bgr() {
        let c = gm_colour(0x0000FF as f64, 1.0).to_srgba();
        assert_eq!((c.red, c.green, c.blue), (1.0, 0.0, 0.0));
        let c = gm_colour(0xFF0000 as f64, 0.5).to_srgba();
        assert_eq!((c.red, c.blue), (0.0, 1.0));
        assert!((c.alpha - 128.0 / 255.0).abs() < 1e-6);
    }
}
