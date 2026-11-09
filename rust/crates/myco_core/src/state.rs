//! Typed save model: the exact shape of `save_N.json` written by `save_game` in the LÖVE port
//! (`game/myco/gen/meta_3.lua`, the `___struct___16x/17x__save_game` builders) and read back by
//! `load_game`. Every struct keeps unknown keys in `extra` so saves pass through unchanged.
//!
//! Numbers: integer-only quantities (ids, indices, kinds, counts, grid sizes) are `i64`; gameplay
//! quantities that may become fractional in GML (positions, food, water, timers) are `f64`.
//! Missing fields take the values a fresh Exploration game has (golden `save_fresh`), or the
//! explicit defaults of `load_game` where it has one.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub type Extra = Map<String, Value>;

/// The whole `save_N.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub version: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub forest: ForestSave,
    #[serde(default)]
    pub mycelium: MyceliumSave,
    #[serde(default)]
    pub mushrooms: Vec<MushroomSave>,
    #[serde(default)]
    pub water: Vec<WaterSave>,
    #[serde(default)]
    pub state: RunState,
    pub world: WorldSave,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Default for GameState {
    /// An empty version-1 save (what a fresh `Sim` starts from before `game_init`).
    fn default() -> Self {
        GameState {
            version: 1,
            name: String::new(),
            forest: ForestSave::default(),
            mycelium: MyceliumSave::default(),
            mushrooms: Vec::new(),
            water: Vec::new(),
            state: RunState::default(),
            world: WorldSave::default(),
            extra: Extra::new(),
        }
    }
}

impl GameState {
    /// `load_game`'s merges for partially present data: tutorial tips missing from the save are
    /// taken from the fresh tutorial (`tut.order` loop), task stats are laid over fresh stats.
    pub fn apply_load_defaults(&mut self) {
        let fresh = TutorialSave::default();
        for (k, v) in fresh.tips {
            self.state.tutorial.tips.entry(k).or_insert(v);
        }
        for (k, v) in TasksSave::default().stats {
            self.state.tasks.stats.entry(k).or_insert(v);
        }
        // `boar_timer = bal("boar_spawn_interval") * fps` after game_mode_apply(st.mode)
        if self.state.timers.boar.is_none() {
            let fps = 60.0;
            let mode = self.state.mode;
            self.state.timers.boar = Some(crate::balance::bal(mode, "boar_spawn_interval") * fps);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// forest (obj_forest_controller.surface_chunks + camera)

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ForestSave {
    pub camera_x: f64,
    pub camera_y: f64,
    pub chunks: BTreeMap<String, ForestChunkSave>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Default for ForestSave {
    fn default() -> Self {
        // fresh game: camera centred on start_x (0) with the 1920-wide view
        ForestSave {
            camera_x: -960.0,
            camera_y: 0.0,
            chunks: BTreeMap::new(),
            extra: Extra::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ForestChunkSave {
    pub trees: Vec<TreeSave>,
    pub small_trees: Vec<SmallTreeSave>,
    pub bushes: Vec<PartSave>,
    pub plants: Vec<PartSave>,
    pub decor_trees: Vec<PartSave>,
    pub stumps: Vec<StumpSave>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Sprite root rectangle (`roots` of trees and small trees).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RootsRect {
    pub ax: f64,
    pub h: f64,
    pub left: f64,
    pub top: f64,
    pub w: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `___struct___160`: id, sprite name, x, y, flip, roots, consume.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TreeSave {
    pub id: i64,
    pub sprite: String,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub roots: Option<RootsRect>,
    pub consume: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `___struct___161`: sprite, x, y, flip, roots.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SmallTreeSave {
    pub sprite: String,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub roots: Option<RootsRect>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Stump roots: a positioned root sprite.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StumpRoots {
    pub flip: f64,
    pub roots: Option<RootsRect>,
    pub x: f64,
    pub y: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `___struct___162`: id, stump part index, x, y, flip, roots, consume, start.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StumpSave {
    pub id: i64,
    pub part: i64,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub roots: Option<StumpRoots>,
    pub consume: f64,
    pub start: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `___struct___163..165` (bushes, plants, decor trees): part, x, y, flip.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PartSave {
    pub part: i64,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

// ---------------------------------------------------------------------------------------------
// mycelium (obj_mycelium_controller.myc_growths / myc_knots)

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MyceliumSave {
    pub growths: Vec<GrowthSave>,
    pub knots: Vec<PointSave>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `___struct___168`: branches.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GrowthSave {
    pub branches: Vec<BranchSave>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// `___struct___167`: points, start, width, shoved, swept.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BranchSave {
    pub points: Vec<PointSave>,
    pub start: f64,
    pub width: f64,
    pub shoved: bool,
    pub swept: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PointSave {
    pub x: f64,
    pub y: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

// ---------------------------------------------------------------------------------------------
// mushrooms (save_copy_struct of the mushroom_add struct; host -> save id, transient fields reset)

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MushroomSave {
    #[serde(rename = "type")]
    pub kind: i64,
    pub variant: i64,
    pub x: f64,
    pub y: f64,
    pub active: bool,
    pub timer: f64,
    pub charges: f64,
    pub flies: Vec<FlySave>,
    pub carried_by: i64,
    pub stolen: bool,
    pub crow_target: bool,
    /// save id of the host tree/stump, -1 for none
    pub host: i64,
    pub side: f64,
    pub stench: Vec<Value>,
    pub stench_due: f64,
    pub hold_angle: f64,
    pub hold_xscale: f64,
    pub hold_yscale: f64,
    pub flash: f64,
    /// current_time at creation; written by save_game, removed by load_game
    #[serde(skip_serializing_if = "Option::is_none")]
    pub born: Option<f64>,
    /// a repeller's autoregen counter (set by `mushrooms_step` on repellers only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regen_timer: Option<f64>,
    /// a stinky mushroom's column was in view on the last step (set by `stinky_step`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stench_live: Option<bool>,
    /// runtime only (Task 9): stable id, host reference, stench particle pool
    #[serde(skip)]
    pub live: MushroomLive,
    #[serde(flatten)]
    pub extra: Extra,
}

/// The runtime part of a mushroom (not saved; `save_game` writes the host as its save id).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MushroomLive {
    /// stable id (`SimEvent::Resource::source`), 0 until assigned
    pub id: u64,
    /// `m.host`: the tree or stump a consumptor eats
    pub host: Option<crate::forest::HostId>,
    /// `m.stench` as a pool struct (`stinky_pool`)
    pub pool: Option<Box<crate::mushrooms::stinky::StinkyPool>>,
}

impl Default for MushroomSave {
    fn default() -> Self {
        // ___struct___59__mushroom_add defaults
        MushroomSave {
            kind: 0,
            variant: 0,
            x: 0.0,
            y: 0.0,
            active: true,
            timer: 0.0,
            charges: 0.0,
            flies: Vec::new(),
            carried_by: -1,
            stolen: false,
            crow_target: false,
            host: -1,
            side: 1.0,
            stench: Vec::new(),
            stench_due: 0.0,
            hold_angle: 0.0,
            hold_xscale: 1.0,
            hold_yscale: 1.0,
            flash: 0.0,
            born: None,
            regen_timer: None,
            stench_live: None,
            live: MushroomLive::default(),
            extra: Extra::new(),
        }
    }
}

/// `___struct___60__mushroom_add`: predator fly orbit.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FlySave {
    pub angle: f64,
    pub radius: f64,
    pub speed: f64,
    pub bob: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

// ---------------------------------------------------------------------------------------------
// water sources

/// `___struct___169` (connected lake: chunk + index in its lakes) or `___struct___170` (water
/// cell: chunk + tile index).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WaterSave {
    Lake {
        lake_cx: i64,
        lake: i64,
        #[serde(flatten)]
        extra: Extra,
    },
    Cell {
        cx: i64,
        idx: i64,
        #[serde(flatten)]
        extra: Extra,
    },
}

// ---------------------------------------------------------------------------------------------
// state (globals; ___struct___174 and its timers/tutorial/tasks parts)

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    #[default]
    Exploration,
    Survival,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RunState {
    pub start_x: f64,
    pub food: f64,
    pub water_free: f64,
    pub water_infinite: bool,
    pub cave_perks: BTreeMap<String, Value>,
    pub cave_taken: BTreeMap<String, Value>,
    pub length_px: f64,
    pub play_frames: i64,
    /// struct or undefined (written as null)
    pub cave_yellow: Option<Value>,
    pub relic_taken: BTreeMap<String, Value>,
    pub mode: GameMode,
    pub points_awarded: i64,
    pub upgrade_points: i64,
    pub upgrade_levels: BTreeMap<String, i64>,
    /// [cam_box_x1, cam_box_y1, cam_box_x2, cam_box_y2]
    pub cam_box: [f64; 4],
    /// [wind_dir, wind_speed, wind_timer]
    pub wind: [f64; 3],
    pub timers: TimersSave,
    pub tutorial: TutorialSave,
    pub tasks: TasksSave,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Default for RunState {
    fn default() -> Self {
        // game_init + game_mode_start("exploration"), balance values as in core_3 / balance_init
        RunState {
            start_x: 0.0,
            food: 50.0,       // start_food
            water_free: 80.0, // start_water
            water_infinite: false,
            cave_perks: BTreeMap::new(),
            cave_taken: BTreeMap::new(),
            length_px: 0.0,
            play_frames: 0,
            cave_yellow: None,
            relic_taken: BTreeMap::new(),
            mode: GameMode::Exploration,
            points_awarded: 0,
            upgrade_points: 0,
            upgrade_levels: BTreeMap::from([("origin".to_string(), 1)]),
            // start_x -/+ screen_width_px/2 - camera_reach_margin_px; 0..screen_height_px
            cam_box: [-1600.0, 0.0, 1600.0, 1120.0],
            // jitter-free: dir 1, (wind_min_strength + wind_max_strength) / 2, wind_change_interval
            wind: [1.0, 2.0, 420.0],
            timers: TimersSave::default(),
            tutorial: TutorialSave::default(),
            tasks: TasksSave::default(),
            extra: Extra::new(),
        }
    }
}

/// `___struct___175`: animal spawn timers in frames.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimersSave {
    pub hedgehog: f64,
    /// `None` when the save lacks it: load_game falls back to `boar_spawn_interval * fps`
    /// under the save's mode ([`GameState::apply_load_defaults`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boar: Option<f64>,
    pub squirrel: f64,
    pub crow: f64,
    pub experience_crows: Vec<f64>,
    pub leaf: f64,
    pub worm: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Default for TimersSave {
    /// life_animal_timers_init without the RNG jitter (60 fps): hedgehog = (spawn interval +
    /// mid first delay), squirrel/crow = spawn interval; boar unset (load_game's mode-aware
    /// fallback fills it); leaf = leaf_spawn_interval, worm = 60.
    fn default() -> Self {
        TimersSave {
            hedgehog: 32130.0,
            boar: None,
            squirrel: 18360.0,
            crow: 24480.0,
            experience_crows: vec![15300.0, 19380.0],
            leaf: 2700.0,
            worm: 60.0,
            extra: Extra::new(),
        }
    }
}

/// `___struct___176`: tutorial progress.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TutorialSave {
    pub tips: BTreeMap<String, TipSave>,
    pub events: BTreeMap<String, Value>,
    pub ate: bool,
    pub drank: bool,
    pub hand_done: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Tutorial tip keys in a fresh game (`tutorial_init`).
pub const TUTORIAL_TIPS: [&str; 10] = [
    "clay",
    "final",
    "mushrooms",
    "node",
    "rain",
    "salt",
    "shoot",
    "upgrade",
    "water",
    "xp",
];

impl Default for TutorialSave {
    fn default() -> Self {
        TutorialSave {
            tips: TUTORIAL_TIPS
                .iter()
                .map(|k| (k.to_string(), TipSave::default()))
                .collect(),
            events: BTreeMap::new(),
            ate: false,
            drank: false,
            hand_done: false,
            extra: Extra::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TipSave {
    pub alpha: f64,
    pub anchor: String,
    pub state: i64,
    pub timer: f64,
    pub x: f64,
    pub y: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Default for TipSave {
    fn default() -> Self {
        TipSave {
            alpha: 0.0,
            anchor: "world".into(),
            state: 0,
            timer: -1.0,
            x: 0.0,
            y: 0.0,
            extra: Extra::new(),
        }
    }
}

/// `___struct___177`: tasks panel and goals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TasksSave {
    pub stats: BTreeMap<String, f64>,
    pub done: BTreeMap<String, Value>,
    pub open: bool,
    pub shown: bool,
    pub stage: i64,
    pub goals: BTreeMap<String, Value>,
    pub popup: String,
    pub goal_open: BTreeMap<String, Value>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Task stat counters in a fresh game (`tasks_init`).
pub const TASK_STATS: [&str; 8] = [
    "built",
    "food",
    "hedgehogs",
    "rain_points",
    "spores",
    "stinky_built",
    "tree_built",
    "water",
];

impl Default for TasksSave {
    fn default() -> Self {
        let goals = |v: bool| -> BTreeMap<String, Value> {
            ["magic", "migration"]
                .iter()
                .map(|k| (k.to_string(), Value::Bool(v)))
                .collect()
        };
        TasksSave {
            stats: TASK_STATS.iter().map(|k| (k.to_string(), 0.0)).collect(),
            done: BTreeMap::new(),
            open: true,
            shown: false,
            stage: 0,
            goals: goals(false),
            popup: String::new(),
            goal_open: goals(true),
            extra: Extra::new(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// world (global.world, saved whole; layout from world_init / world chunk generation)

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldSave {
    pub bottom_y: f64,
    pub chunk_index: Vec<i64>,
    pub chunk_w: i64,
    pub chunks: BTreeMap<String, WorldChunkSave>,
    pub cols: i64,
    pub hard_row: i64,
    pub layer_rows: i64,
    /// `world.obstacles`: absent until `world_obstacles_reach` first builds it (lazily, from
    /// `start_x`), so a save without the key starts the bands afresh
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obstacles: Option<ObstaclesSave>,
    pub rows: i64,
    pub seed: i64,
    pub start_x: f64,
    pub surface_y: f64,
    pub tile: i64,
    /// asset index of the turf sprite; load_game replaces it
    pub turf: i64,
    pub water_row: i64,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldChunkSave {
    pub cx: i64,
    pub rows: i64,
    pub tiles: Vec<i64>,
    pub amount: Vec<f64>,
    pub connected: Vec<bool>,
    pub foods: Vec<FoodSave>,
    pub lakes: Vec<LakeSave>,
    pub stones: Vec<StoneSave>,
    pub stone_cells: Vec<StoneCellSave>,
    pub stone_map: BTreeMap<String, StoneCellSave>,
    pub bones: Vec<BoneSave>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clusters_done: Option<BTreeMap<String, Value>>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FoodSave {
    pub cell: i64,
    pub collected: bool,
    pub kind: i64,
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maze: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sv: Option<bool>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LakeSave {
    pub amount: f64,
    pub body_h: f64,
    pub body_w: f64,
    pub body_x: f64,
    pub body_y: f64,
    pub connected: bool,
    pub cx: i64,
    pub flare_seq: i64,
    /// runtime sparkle list; save_game clears it (null)
    pub flares: Option<Value>,
    pub full: f64,
    pub kind: i64,
    pub seed: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sv: Option<bool>,
    pub x: f64,
    pub y: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A loose stone (`___struct___215__world_paint_stones`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StoneSave {
    pub angle: f64,
    pub cx: i64,
    pub kind: i64,
    pub moving: bool,
    pub side: f64,
    pub spin: f64,
    pub vx: f64,
    pub vy: f64,
    pub x: f64,
    pub y: f64,
    #[serde(flatten)]
    pub extra: Extra,
    /// runtime identity (Lua compares the struct references): `global.stones_moving` holds
    /// stones by uid; not saved, ignored by `==`
    #[serde(skip)]
    pub uid: u64,
}

impl PartialEq for StoneSave {
    fn eq(&self, o: &Self) -> bool {
        self.angle == o.angle
            && self.cx == o.cx
            && self.kind == o.kind
            && self.moving == o.moving
            && self.side == o.side
            && self.spin == o.spin
            && self.vx == o.vx
            && self.vy == o.vy
            && self.x == o.x
            && self.y == o.y
            && self.extra == o.extra
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StoneCellSave {
    /// tile depth row (0/1); old saves may lack it (`stone_redress_after_cut` then hashes one)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band: Option<i64>,
    pub col: i64,
    /// whole deposit below the deep line; old saves may lack it (`world_draw_stone_deposits`
    /// then uses the cell's own row)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep: Option<bool>,
    pub row: i64,
    pub tc: i64,
    pub tr: i64,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BoneSave {
    pub flip: f64,
    pub h: f64,
    pub kind: i64,
    pub w: f64,
    pub x: f64,
    pub y: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObstaclesSave {
    pub left: ObstacleSideSave,
    pub right: ObstacleSideSave,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObstacleSideSave {
    pub count: i64,
    pub cursor: f64,
    pub list: Vec<ObstacleSave>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// One obstacle band (`___struct___228/229__world_obstacles_reach`): a `"wall"` has `cx`, a
/// `"maze"` has `c1`/`c2` and `data` (null until `world_maze_build` caches the maze in it).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObstacleSave {
    pub key: i64,
    pub kind: String,
    pub x1: f64,
    pub x2: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cx: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c2: Option<f64>,
    /// absent (walls), `Some(None)` = null, `Some(Some(_))` = the built maze
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_or_null"
    )]
    pub data: Option<Option<MazeData>>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A key that is present deserializes to `Some` (null included); `#[serde(default)]` gives
/// `None` when it is absent.
fn present_or_null<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}

/// A built maze or rock cluster (`___struct___236__maze_finish`): its stone cells, foods and
/// lakes, painted into each chunk they overlap (`world_paint_maze_part`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MazeData {
    pub entries: Vec<StoneCellSave>,
    pub foods: Vec<MazeFoodSave>,
    pub lakes: Vec<LakeSave>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// A food of a maze room (`___struct___235__maze_finish`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MazeFoodSave {
    pub x: f64,
    pub y: f64,
    pub kind: i64,
    pub cell: i64,
    #[serde(flatten)]
    pub extra: Extra,
}

// ---------------------------------------------------------------------------------------------
// slot meta and display settings

/// `save_N_meta.json` (`___struct___178`): shown in the slot list without loading the save.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotMeta {
    pub length_cm: f64,
    #[serde(default)]
    pub mode: GameMode,
    #[serde(default)]
    pub name: String,
    /// date_current_datetime(): days since 1899-12-30
    pub saved: f64,
}

/// `settings.ini` [display] (`game/gm/display_math.lua`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplaySettings {
    pub fullscreen: bool,
    pub window_w: u32,
    pub window_h: u32,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        DisplaySettings {
            fullscreen: true,
            window_w: 1280,
            window_h: 720,
        }
    }
}
