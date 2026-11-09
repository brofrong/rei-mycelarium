//! Game assets extracted by `tools/extract.py` (`build/assets`), read from the same JSON files as
//! `game/gm/assets.lua`: texture pages `pages/page<N>.png`, `sprites.json`, `fonts.json` (GameMaker
//! bitmap fonts on the pages), `sounds.json` + `sounds/`, TTF fonts in `fonts/`.
//!
//! Asset order (the original's numeric ids) is the key order in each JSON file (`serde_json`
//! with `preserve_order`), as `assets.lua ordered_keys` restores it.
//!
//! The folder comes from `MYCO_ASSETS`, else `<app>/Contents/Resources/assets` inside a bundle,
//! else `build/assets` (relative to the working directory or the repository).

use bevy::prelude::*;
use bevy::sprite::Anchor;
use myco_core::locale::FontId;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One frame of a sprite: a rectangle on a texture page plus its trim offset inside the
/// sprite's `size` box (`dst` in sprites.json).
#[derive(Clone, Debug)]
pub struct FrameRef {
    pub page: Handle<Image>,
    pub page_index: usize,
    /// source rectangle on the page, pixels
    pub rect: Rect,
    /// where the trimmed rectangle sits inside the untrimmed `size` box
    pub trim_offset: Vec2,
}

#[derive(Clone, Debug)]
pub struct SpriteDef {
    pub name: String,
    /// original asset index (data.win order)
    pub index: usize,
    pub frames: Vec<FrameRef>,
    /// `sprite_get_xoffset/yoffset`
    pub origin: Vec2,
    /// untrimmed size (`w`, `h`)
    pub size: Vec2,
    /// `[left, top, right, bottom]` collision box
    pub bbox: [f32; 4],
    pub speed: f64,
    pub speed_type: i64,
}

impl SpriteDef {
    /// A Bevy sprite drawing `frame` with the sprite origin at the entity's position
    /// (y up in Bevy; flips must be applied by the caller through `Transform` scale).
    pub fn bevy_sprite(&self, frame: usize) -> (Sprite, Anchor) {
        let fr = &self.frames[frame % self.frames.len().max(1)];
        let size = fr.rect.size();
        let local = self.origin - fr.trim_offset; // origin inside the trimmed rect, y down
        let anchor = Anchor(Vec2::new(local.x / size.x - 0.5, 0.5 - local.y / size.y));
        let sprite = Sprite {
            image: fr.page.clone(),
            rect: Some(fr.rect),
            ..default()
        };
        (sprite, anchor)
    }
}

/// A glyph of a GameMaker bitmap font (`fonts.json`), coordinates relative to the font's frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub shift: f32,
    pub offset: f32,
}

/// GameMaker bitmap font (`fnt_gm_15`, `Font1`, ...). Rendering is the UI tasks' job.
#[derive(Clone, Debug)]
pub struct GmFont {
    pub name: String,
    pub index: usize,
    pub size: f64,
    pub page: Handle<Image>,
    pub page_index: usize,
    /// the font's rectangle on the page; glyph rects are relative to its top-left
    pub frame: Rect,
    pub glyphs: HashMap<u32, Glyph>,
    pub ascender: f64,
    pub ascender_offset: f64,
    pub line_height: f64,
    pub raw: Value,
}

#[derive(Clone, Debug)]
pub struct SoundDef {
    pub name: String,
    pub index: usize,
    pub handle: Handle<AudioSource>,
    pub volume: f64,
    pub pitch: f64,
    pub group: i64,
    pub streamed: bool,
}

#[derive(Resource, Debug)]
pub struct GameAssets {
    pub root: PathBuf,
    pub pages: Vec<Handle<Image>>,
    pub sprites: Vec<SpriteDef>,
    sprite_by_name: HashMap<String, usize>,
    pub gm_fonts: Vec<GmFont>,
    gm_font_by_name: HashMap<String, usize>,
    fonts: HashMap<FontId, Handle<Font>>,
    pub sounds: Vec<SoundDef>,
    sound_by_name: HashMap<String, usize>,
    /// `game.json`
    pub game: Value,
}

impl GameAssets {
    /// Sprite by name; panics like `gm: unknown sprite` for a missing name.
    pub fn sprite(&self, name: &str) -> &SpriteDef {
        self.try_sprite(name)
            .unwrap_or_else(|| panic!("unknown sprite {name}"))
    }

    pub fn try_sprite(&self, name: &str) -> Option<&SpriteDef> {
        self.sprite_by_name.get(name).map(|&i| &self.sprites[i])
    }

    /// TTF font for a language font id. `FontId::Default` is the GameMaker bitmap font `Font1`
    /// (see [`gm_font`](Self::gm_font)); as a Bevy `Font` it falls back to the Latin/Cyrillic TTF.
    pub fn font(&self, id: FontId) -> Handle<Font> {
        self.fonts
            .get(&id)
            .or_else(|| self.fonts.get(&FontId::CyrGerSpaPort))
            .cloned()
            .unwrap_or_default()
    }

    pub fn gm_font(&self, name: &str) -> Option<&GmFont> {
        self.gm_font_by_name.get(name).map(|&i| &self.gm_fonts[i])
    }

    /// Sound by name; panics for a missing name.
    pub fn sound(&self, name: &str) -> Handle<AudioSource> {
        self.sound_def(name)
            .unwrap_or_else(|| panic!("unknown sound {name}"))
            .handle
            .clone()
    }

    pub fn sound_def(&self, name: &str) -> Option<&SoundDef> {
        self.sound_by_name.get(name).map(|&i| &self.sounds[i])
    }

    /// Text file under the assets folder (`assets.read_text`), e.g. `localization.csv`.
    pub fn read_text(&self, name: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.root.join(name))
    }

    /// Read the JSON catalogues and start loading pages, fonts and sounds through `load`
    /// (`AssetServer::load` with a path relative to the assets folder).
    pub fn load(
        root: &Path,
        mut load_image: impl FnMut(&str) -> Handle<Image>,
        mut load_font: impl FnMut(&str) -> Handle<Font>,
        mut load_sound: impl FnMut(&str) -> Handle<AudioSource>,
    ) -> Result<GameAssets, String> {
        let read = |name: &str| -> Result<Value, String> {
            let text = std::fs::read_to_string(root.join(name))
                .map_err(|e| format!("{}: {e}", root.join(name).display()))?;
            serde_json::from_str(&text).map_err(|e| format!("{name}: {e}"))
        };
        let game = read("game.json")?;
        let mut pages = Vec::new();
        while root.join(format!("pages/page{}.png", pages.len())).exists() {
            pages.push(load_image(&format!("pages/page{}.png", pages.len())));
        }
        let page = |i: usize| -> Result<Handle<Image>, String> {
            pages
                .get(i)
                .cloned()
                .ok_or_else(|| format!("missing texture page {i}"))
        };
        let num = |v: &Value, k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0);
        let arr = |v: &Value, k: &str, i: usize| {
            v.get(k)
                .and_then(|a| a.get(i))
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32
        };
        let rect = |v: &Value| {
            Rect::new(
                arr(v, "src", 0),
                arr(v, "src", 1),
                arr(v, "src", 0) + arr(v, "src", 2),
                arr(v, "src", 1) + arr(v, "src", 3),
            )
        };

        let mut sprites = Vec::new();
        let mut sprite_by_name = HashMap::new();
        let sj = read("sprites.json")?;
        for (index, (name, s)) in sj
            .as_object()
            .ok_or("sprites.json: not an object")?
            .iter()
            .enumerate()
        {
            let mut frames = Vec::new();
            for fr in s
                .get("frames")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let pi = num(fr, "page") as usize;
                frames.push(FrameRef {
                    page: page(pi)?,
                    page_index: pi,
                    rect: rect(fr),
                    trim_offset: Vec2::new(arr(fr, "dst", 0), arr(fr, "dst", 1)),
                });
            }
            sprite_by_name.insert(name.clone(), index);
            sprites.push(SpriteDef {
                name: name.clone(),
                index,
                frames,
                origin: Vec2::new(arr(s, "origin", 0), arr(s, "origin", 1)),
                size: Vec2::new(num(s, "w") as f32, num(s, "h") as f32),
                bbox: [
                    arr(s, "bbox", 0),
                    arr(s, "bbox", 1),
                    arr(s, "bbox", 2),
                    arr(s, "bbox", 3),
                ],
                speed: num(s, "speed"),
                speed_type: num(s, "speed_type") as i64,
            });
        }

        let mut gm_fonts = Vec::new();
        let mut gm_font_by_name = HashMap::new();
        let fj = read("fonts.json")?;
        for (index, (name, f)) in fj
            .as_object()
            .ok_or("fonts.json: not an object")?
            .iter()
            .enumerate()
        {
            let frame = f.get("frame").cloned().unwrap_or(Value::Null);
            let pi = num(&frame, "page") as usize;
            let mut glyphs = HashMap::new();
            for (code, g) in f
                .get("glyphs")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                let Ok(code) = code.parse::<u32>() else {
                    continue;
                };
                glyphs.insert(
                    code,
                    Glyph {
                        x: num(g, "x") as f32,
                        y: num(g, "y") as f32,
                        w: num(g, "w") as f32,
                        h: num(g, "h") as f32,
                        shift: num(g, "shift") as f32,
                        offset: num(g, "offset") as f32,
                    },
                );
            }
            gm_font_by_name.insert(name.clone(), index);
            gm_fonts.push(GmFont {
                name: name.clone(),
                index,
                size: num(f, "size"),
                page: page(pi)?,
                page_index: pi,
                frame: rect(&frame),
                glyphs,
                ascender: num(f, "ascender"),
                ascender_offset: num(f, "ascender_offset"),
                line_height: num(f, "line_height"),
                raw: f.clone(),
            });
        }

        let mut fonts = HashMap::new();
        for id in [
            FontId::Default,
            FontId::ChSim,
            FontId::Korean,
            FontId::Japan,
            FontId::CyrGerSpaPort,
        ] {
            if let Some(file) = id.file() {
                if root.join(file).exists() {
                    fonts.insert(id, load_font(file));
                }
            }
        }

        let mut sounds = Vec::new();
        let mut sound_by_name = HashMap::new();
        let so = read("sounds.json")?;
        for (index, (name, s)) in so
            .as_object()
            .ok_or("sounds.json: not an object")?
            .iter()
            .enumerate()
        {
            let file = s.get("file").and_then(Value::as_str).unwrap_or_default();
            sound_by_name.insert(name.clone(), index);
            sounds.push(SoundDef {
                name: name.clone(),
                index,
                handle: load_sound(&format!("sounds/{file}")),
                volume: num(s, "volume"),
                pitch: num(s, "pitch"),
                group: num(s, "group") as i64,
                streamed: s.get("streamed").and_then(Value::as_bool).unwrap_or(false),
            });
        }

        Ok(GameAssets {
            root: root.to_path_buf(),
            pages,
            sprites,
            sprite_by_name,
            gm_fonts,
            gm_font_by_name,
            fonts,
            sounds,
            sound_by_name,
            game,
        })
    }
}

/// Resolve the assets folder: `MYCO_ASSETS`, the app bundle's `Contents/Resources/assets`, or
/// `build/assets`. Relative paths are tried against the working directory, then the repository
/// root (so `MYCO_ASSETS=build/assets cargo run` works from `rust/`). Returns an absolute path.
pub fn assets_root() -> PathBuf {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let resolve = |p: PathBuf| -> PathBuf {
        if p.is_absolute() {
            return p;
        }
        let cwd = std::env::current_dir().unwrap_or_default().join(&p);
        if cwd.exists() {
            return cwd;
        }
        repo.join(&p)
    };
    let p = if let Some(env) = std::env::var_os("MYCO_ASSETS").filter(|v| !v.is_empty()) {
        resolve(PathBuf::from(env))
    } else {
        let bundle = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|d| d.join("../Resources/assets")))
            .filter(|p| p.join("sprites.json").exists());
        bundle.unwrap_or_else(|| resolve(PathBuf::from("build/assets")))
    };
    p.canonicalize().unwrap_or(p)
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_game_assets);
    }
}

fn load_game_assets(mut commands: Commands, server: Res<AssetServer>) {
    let root = assets_root();
    match GameAssets::load(
        &root,
        |p| server.load(p.to_owned()),
        |p| server.load(p.to_owned()),
        |p| server.load(p.to_owned()),
    ) {
        Ok(a) => {
            info!(
                "assets: {} ({} sprites, {} pages)",
                root.display(),
                a.sprites.len(),
                a.pages.len()
            );
            commands.insert_resource(a);
        }
        Err(e) => panic!(
            "cannot load game assets from {}: {e} (set MYCO_ASSETS)",
            root.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Against the real extracted assets when they are present (skipped otherwise).
    #[test]
    fn reads_catalogues_in_file_order() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build/assets");
        if !root.join("sprites.json").exists() {
            eprintln!("skipped: no build/assets");
            return;
        }
        let a = GameAssets::load(
            &root,
            |_| Handle::default(),
            |_| Handle::default(),
            |_| Handle::default(),
        )
        .unwrap();
        assert_eq!(a.sprites[0].name, "apple_bite");
        assert_eq!(a.sprite("apple").index, 1);
        let apple = a.sprite("apple_bite");
        assert_eq!(apple.index, 0);
        assert_eq!(apple.size, Vec2::new(73.0, 72.0));
        assert_eq!(apple.frames[0].page_index, 4);
        assert_eq!(
            apple.frames[0].rect,
            Rect::new(1772.0, 243.0, 1837.0, 310.0)
        );
        assert_eq!(apple.frames[0].trim_offset, Vec2::new(4.0, 1.0));
        assert_eq!(
            a.gm_fonts
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            ["fnt_gm_15", "fnt_gm_20", "fnt_gm_30", "Font1"]
        );
        assert_eq!(a.gm_font("fnt_gm_15").unwrap().glyphs[&33].x, 2.0);
        assert_eq!(a.sounds.len(), 24);
        assert_eq!(a.sound_def("bird").unwrap().index, 0);
        assert!(a.fonts.contains_key(&FontId::CyrGerSpaPort));
    }

    #[test]
    fn anchor_puts_origin_at_entity() {
        let def = SpriteDef {
            name: "t".into(),
            index: 0,
            frames: vec![FrameRef {
                page: Handle::default(),
                page_index: 0,
                rect: Rect::new(0.0, 0.0, 20.0, 10.0),
                trim_offset: Vec2::new(5.0, 0.0),
            }],
            origin: Vec2::new(15.0, 10.0),
            size: Vec2::new(30.0, 10.0),
            bbox: [0.0; 4],
            speed: 0.0,
            speed_type: 0,
        };
        let (_, anchor) = def.bevy_sprite(0);
        // origin (15,10) is (10,10) inside the trimmed 20x10 rect: bottom centre
        assert_eq!(anchor.0, Vec2::new(0.0, -0.5));
    }
}
