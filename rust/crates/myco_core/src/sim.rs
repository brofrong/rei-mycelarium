//! The simulation: one `Sim` per running game, stepped 60 times a second by the app.
//!
//! - `sim.state` (the save model) is the live home of every saved global: `state.state.food`,
//!   `state.mushrooms`, `state.water` (= `global.water_sources`), ... Subsystems keep only
//!   runtime-only data in their `*Rt` structs and write it back in their `before_save` hooks.
//! - `Sim::step` runs the instances of Room1 in the original order: Begin Step of all
//!   instances (only obj_hud has one), then Step (obj_game, obj_forest_controller,
//!   obj_mycelium_controller, obj_life); no instance has an End Step. Subsystem tasks fill the
//!   hooks it calls and add their own `Command` variants, nothing else here.
//! - Construction runs the room's Create events through the same hooks for a new game and for
//!   a loaded one, as GameMaker does (load_game runs on a live Room1): `game::init`
//!   (obj_game Create, `game_init`), then `forest::create`, `mycelium::create`,
//!   `life::create` in instance order. A new game then runs `save_start_new` +
//!   `game_mode_start`; a load replaces `sim.state` with the save, applies load_game's data
//!   fixups and calls the `on_load` hooks (world, progress, forest, mycelium, mushrooms, life,
//!   then `weather::init`). Hook contract: on a load, `create` has always run before `on_load`,
//!   so `on_load` may rely on create-built runtime state (stump parts, growth duration, fresh
//!   tutorial/tasks) and must only overwrite what the save restores.
//! - The app passes UI state in (`StepInput`), turns clicks on in-world buttons into
//!   `Command`s and drains `SimEvent`s; logic never draws or measures text.

use crate::balance;
use crate::events::SimEvent;
use crate::mushrooms::MushroomKind;
use crate::rng::Rng;
use crate::save::{self, SaveError};
use crate::state::{GameMode, GameState, SlotMeta};
use crate::{camera, forest, game, life, mushrooms, mycelium, progress, weather, world};
use std::fmt;
use std::path::{Path, PathBuf};

/// The logical game view (1×, `view_for_window`), world pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

/// Keys the game reads (`keyboard_check*` in the original).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Shift,
    Alt,
    Enter,
    Escape,
    Backspace,
    F1,
    F2,
    /// letter (uppercase) or digit key, as `ord("A")` / `ord("1")`
    Char(char),
    /// numpad digit 0–9 (`vk_numpad0` = 96 + n)
    Numpad(u8),
}

impl Key {
    /// GameMaker virtual key code.
    pub fn vk(self) -> i64 {
        match self {
            Key::Left => 37,
            Key::Right => 39,
            Key::Up => 38,
            Key::Down => 40,
            Key::Shift => 16,
            Key::Alt => 18,
            Key::Enter => 13,
            Key::Escape => 27,
            Key::Backspace => 8,
            Key::F1 => 112,
            Key::F2 => 113,
            Key::Char(c) => c.to_ascii_uppercase() as i64,
            Key::Numpad(n) => 96 + n as i64,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StepInput {
    /// world px; `None` when the cursor is outside the window
    pub cursor: Option<(f64, f64)>,
    /// GUI coordinates: the GUI layer is view-sized (`display_set_gui_size(view_w, view_h)`,
    /// so `0..view.w` x `0..view.h`)
    pub cursor_gui: Option<(f64, f64)>,
    pub lmb: ButtonState,
    pub rmb: ButtonState,
    pub keys_pressed: Vec<Key>,
    pub keys_down: Vec<Key>,
    pub text: String,
    /// the cursor is over the HUD or a menu
    pub ui_captured: bool,
    pub view: ViewRect,
    pub commands: Vec<Command>,
    /// a tasks popup is open (`tasks_popup_open`, app state; ruling R12)
    pub tasks_popup_open: bool,
    /// the upgrade tree screen is open (`global.menu_open`, HUD state; read by `tutorial_step`)
    pub menu_open: bool,
    /// the debug overlay is open (`global.debug_open`, HUD state)
    pub debug_open: bool,
    /// the logo screen is shown (`global.logo_open`, app state)
    pub logo_open: bool,
    /// the HUD is hidden (`global.hud_hidden`, HUD state; read by `tasks_visible`)
    pub hud_hidden: bool,
    /// a slot name is being typed (`obj_hud.hud_slot_naming != -1`): the F glow key is text
    pub slot_naming: bool,
    /// the HUD's placement mode (`obj_hud.hud_place_type >= 0`): the type being placed. While
    /// set, `myc_place_check` runs for the cursor every step (`MushroomsRt::place_preview`,
    /// ruling R22) and `Command::QuickNode` is ignored (a right click cancels placement).
    pub placement: Option<MushroomKind>,
}

/// What the app asks the simulation to do this step (clicks the app hit-tests, ruling R9).
///
/// Contract with Task 8: the in-world button commands (`MycReleaseSpores`, `MycBuildOption`,
/// `MycRegenRepeller`) are dropped while `StepInput.ui_captured` is set, so the in-world
/// prompts must never set `UiCapture` themselves; only HUD and menu screens do.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// right click
    QuickNode {
        x: f64,
        y: f64,
    },
    PlaceMushroom {
        kind: MushroomKind,
        x: f64,
        y: f64,
    },
    BuyUpgrade {
        id: String,
    },
    SetPaused(bool),
    Save,
    /// `tasks_block_toggle(block)`: a click on a tasks panel header (the app hit-tests the
    /// text-measured header, ruling R15); flips the saved open flag in obj_hud's Begin Step
    ToggleTaskBlock {
        block: crate::progress::tasks::Block,
    },
    /// HUD camera buttons (obj_hud Begin Step): `camera_jump_edge(dir)` (left -1 / right 1)
    CameraJumpEdge {
        dir: f64,
    },
    /// HUD camera button: `camera_jump_origin()`
    CameraJumpOrigin,
    /// a click on an animal marker: `camera_fly_to_marker` toward the animal at (x, y)
    CameraFlyTo {
        x: f64,
        y: f64,
    },
    /// the flown marker's position this step (the original reads `camera_fly.marker` live; the
    /// markers are the app's): `camera_fly_track`
    CameraFlyTrack {
        x: f64,
        y: f64,
    },
    /// the app's render zoom moved the logic camera so the point under the cursor stays put:
    /// its new top-left, applied at the start of the step ([`crate::camera::camera_pan_to`]),
    /// so the logic camera only changes inside steps and replays the same
    CameraPan {
        x: f64,
        y: f64,
    },
    /// Task 8: a click on the in-world build menu's option `index` (the app hit-tests
    /// `myc_build_button_rect`; ruling R9b)
    MycBuildOption {
        index: usize,
    },
    /// Task 8: a click on the "release spores" button (`myc_spore_button_rect`)
    MycReleaseSpores,
    /// Task 8: a click on the "regenerate" button (`myc_regen_button_rect`)
    MycRegenRepeller,
    /// Task 8: open the in-world build menu at a world point (the original's right click,
    /// `myc_open_build_menu_at`; the port's right click is `QuickNode`)
    OpenBuildMenu {
        x: f64,
        y: f64,
    },
    // Tasks 9–12 add their variants here and touch nothing else
}

/// `global.save_slot` / `save_name` / `save_timer`.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveRt {
    /// -1: no slot (title screen, tests)
    pub slot: i64,
    pub name: String,
    /// frames until the next autosave
    pub timer: f64,
}

impl Default for SaveRt {
    // port: save_init
    fn default() -> Self {
        // save_init
        SaveRt {
            slot: -1,
            name: String::new(),
            timer: 0.0,
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    /// `version` other than 1
    Version(i64),
    Save(SaveError),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Version(v) => write!(f, "unsupported save version {v}"),
            LoadError::Save(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<SaveError> for LoadError {
    fn from(e: SaveError) -> Self {
        match e {
            SaveError::Version(v) => LoadError::Version(v),
            e => LoadError::Save(e),
        }
    }
}

pub struct Sim {
    pub state: GameState,
    pub rng: Rng,
    /// frames stepped since this Sim was created
    pub frame: u64,
    pub world: world::WorldRt,
    pub forest: forest::ForestRt,
    pub mycelium: mycelium::MyceliumRt,
    pub mushrooms: mushrooms::MushroomsRt,
    pub life: life::LifeRt,
    pub progress: progress::ProgressRt,
    pub weather: weather::WeatherRt,
    pub camera: camera::CameraRt,
    pub save: SaveRt,
    /// where `save_game` writes; `None` keeps the session in memory (tests, conformance runs)
    pub save_dir: Option<PathBuf>,
    /// `global.title_open`, `global.intro_active`, `global.pause_open`
    pub title_open: bool,
    pub intro_active: bool,
    pub pause_open: bool,
    /// `global.fps` (`game_get_speed`): logic runs at 60 steps per second
    pub fps: f64,
    /// fixed world seed (like `MYCO_SEED`), used by `world_init` (Task 5)
    pub world_seed: Option<i64>,
    /// Unique per `Sim` created in this process: the app's render caches see a replaced game
    /// (a load, a restart to the title) by a new session.
    pub session: u64,
    /// When `Some`, `step` appends the name of each subsystem call (order tests).
    pub trace: Option<Vec<&'static str>>,
    /// The hooks `new_game` / `from_save` called, in order (a dozen names; order tests).
    pub build_log: Vec<&'static str>,
    events: Vec<SimEvent>,
}

impl Sim {
    /// A Sim before any game code ran (`save_init` state, clock-seeded RNG).
    fn blank() -> Sim {
        static SESSIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let mut rng = Rng::new(0);
        rng.randomise();
        Sim {
            session: SESSIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            state: GameState::default(),
            rng,
            frame: 0,
            world: Default::default(),
            forest: Default::default(),
            mycelium: Default::default(),
            mushrooms: Default::default(),
            life: Default::default(),
            progress: Default::default(),
            weather: Default::default(),
            camera: Default::default(),
            save: SaveRt::default(),
            save_dir: None,
            title_open: false,
            intro_active: false,
            pause_open: false,
            fps: 60.0,
            world_seed: None,
            trace: None,
            build_log: Vec::new(),
            events: Vec::new(),
        }
    }

    fn built(&mut self, name: &'static str) {
        self.build_log.push(name);
    }

    /// The room's Create events in instance order (obj_hud's is the app's). Shared by
    /// `new_game` and `from_save`.
    fn create_room(world_seed: Option<i64>) -> Sim {
        let mut sim = Sim::blank();
        sim.world_seed = world_seed;
        sim.built("game.init");
        sim.obj_game_create();
        sim.built("forest.create");
        forest::create(&mut sim);
        sim.built("mycelium.create");
        mycelium::create(&mut sim);
        sim.built("life.create");
        life::create(&mut sim);
        sim
    }

    pub(crate) fn mark(&mut self, name: &'static str) {
        if let Some(t) = &mut self.trace {
            t.push(name);
        }
    }

    /// `bal(key)` under this game's mode.
    pub fn bal(&self, key: &str) -> f64 {
        balance::bal(self.state.state.mode, key)
    }

    /// A new game: the room's Create events (obj_game, obj_forest_controller,
    /// obj_mycelium_controller, obj_life; obj_hud's is the app's), then what the title menu
    /// does for "new game": `save_start_new` (no slot yet: bind one with
    /// [`Sim::save_start_new`]) and `game_mode_start`.
    pub fn new_game(mode: GameMode, name: &str, world_seed: Option<i64>) -> Sim {
        let mut sim = Sim::create_room(world_seed);
        sim.built("save_start_new");
        sim.save_start_new(-1, name);
        sim.built("game_mode_start");
        sim.game_mode_start(mode);
        sim.title_open = false;
        sim
    }

    /// The game as it boots (and after `game_restart`): the room's Create events with the title
    /// menu open over a fresh world; no slot is bound. The menu continues with
    /// [`Sim::start_from_title`] or replaces it with [`Sim::load_game`].
    pub fn title_screen(world_seed: Option<i64>) -> Sim {
        let mut sim = Sim::create_room(world_seed);
        sim.title_open = true;
        sim
    }

    /// What the title menu does once a new game is named (obj_hud Step_1): bind the slot and
    /// name, start the chosen mode on the world behind the title, close the title. Together
    /// with [`Sim::title_screen`] this is the same sequence as [`Sim::new_game`].
    pub fn start_from_title(&mut self, slot: i64, name: &str, mode: GameMode) {
        self.save_start_new(slot, name);
        self.game_mode_start(mode);
        self.title_open = false;
    }

    // port: gml_Object_obj_game_Create_0
    /// obj_game Create_0: `game_init` (Task 10 fills `game::init`); `audio_init` is the app's.
    fn obj_game_create(&mut self) {
        game::init(self);
    }

    // port: gml_Object_obj_game_Other_3
    /// obj_game Other_3 (Game End): save on quit.
    pub fn game_end(&mut self) -> bool {
        self.save_game()
    }

    // port: game_mode_start
    /// `game_mode_start(mode)`: apply the mode, reset the animal timers and starting food and
    /// water, thin the resources of an existing world in survival.
    pub fn game_mode_start(&mut self, mode: GameMode) {
        self.game_mode_apply(mode);
        life::animal_timers_init(self);
        self.state.state.food = self.bal("start_food");
        self.state.state.water_free = self.bal("start_water");
        if balance::game_mode(&self.state) == GameMode::Survival {
            world::survival_thin_all(self);
        }
    }

    /// `game_mode_apply(mode)`: `global.game_mode`. `bal` follows `state.state.mode`; the
    /// world's copy (`WorldRt::mode`, read where no `Sim` is at hand) is kept in step.
    pub fn game_mode_apply(&mut self, mode: GameMode) {
        self.state.state.mode = mode;
        self.world.mode = mode;
    }

    // port: food_add
    pub fn food_add(&mut self, n: f64) {
        self.state.state.food += n;
    }

    // port: food_spend
    /// Spend `n` food if there is that much.
    pub fn food_spend(&mut self, n: f64) -> bool {
        if self.state.state.food < n {
            return false;
        }
        self.state.state.food -= n;
        true
    }

    // port: water_total
    /// Free water plus what is left in every connected source; 1e6 with infinite water.
    pub fn water_total(&self) -> f64 {
        if self.state.state.water_infinite {
            return 1_000_000.0;
        }
        let mut total = self.state.state.water_free;
        for src in &self.state.water {
            total += world::water_source_amount(self, src);
        }
        total
    }

    /// One logic frame (1/60 s).
    /// `current_time` on the sim clock: ms since the Sim was created, `floor(frame * 1000 / 60)`.
    /// Every port of `current_time` uses this mapping.
    pub fn now_ms(&self) -> f64 {
        (self.frame as f64 * 1000.0 / 60.0).floor()
    }

    pub fn step(&mut self, input: &StepInput) {
        for cmd in &input.commands {
            match cmd {
                Command::SetPaused(p) => self.pause_open = *p,
                Command::CameraPan { x, y } => crate::camera::camera_pan_to(self, *x, *y),
                Command::Save => {
                    self.save_game();
                }
                // read by the subsystem steps where the original reads the input
                _ => {}
            }
        }
        // Begin Step
        self.mark("hud.begin_step");
        progress::hud_begin_step(self, input);
        // Step: obj_game (game_step, save_step, cheats_step; display/focus/audio steps are the
        // app's, steam_step is skipped)
        game::obj_game_step(self, input);
        self.mark("forest.step");
        forest::step(self, input);
        self.mark("mycelium.step");
        mycelium::step(self, input);
        self.mark("life.step");
        life::step(self, input);
        self.frame += 1;
    }

    pub fn emit(&mut self, e: SimEvent) {
        self.events.push(e);
    }

    pub fn drain_events(&mut self) -> Vec<SimEvent> {
        std::mem::take(&mut self.events)
    }

    // -----------------------------------------------------------------------------------------
    // saving (scr_save; file I/O in save.rs)

    // port: save_start_new
    /// Bind the game to a slot and name; the first autosave comes after 2 seconds.
    pub fn save_start_new(&mut self, slot: i64, name: &str) {
        self.save.slot = slot;
        self.save.name = name.to_string();
        self.save.timer = self.fps * 2.0;
    }

    // port: save_step
    /// Autosave countdown (paused on the title, during the intro and in the pause menu).
    pub fn save_step(&mut self) {
        if self.save.slot < 0 || self.title_open || self.intro_active || self.pause_open {
            return;
        }
        self.save.timer -= 1.0;
        if self.save.timer <= 0.0 {
            self.save_game();
        }
    }

    // port: save_game
    /// Save into the bound slot: `save_<slot+1>.json` and its meta. Returns false when there is
    /// no slot, on the title or in the intro, or when writing fails; emits `SimEvent::Saved`
    /// after a successful write. Without `save_dir` nothing is written.
    pub fn save_game(&mut self) -> bool {
        if self.save.slot < 0 || self.title_open || self.intro_active {
            return false;
        }
        self.save.timer = self.bal("save_autosave_seconds") * self.fps;
        let Some(dir) = self.save_dir.clone() else {
            return true;
        };
        let (state, meta) = self.to_save();
        match save::write_slot(&dir, self.save.slot as usize, &state, &meta) {
            Ok(()) => {
                self.emit(SimEvent::Saved);
                true
            }
            Err(_) => false,
        }
    }

    /// What `save_game` writes: the live state with each subsystem's runtime data written back,
    /// then save_game's own transforms; and the slot meta.
    pub fn to_save(&self) -> (GameState, SlotMeta) {
        // port: save_copy_struct
        // (save_copy_struct copies each mushroom before its fields are changed: the clone)
        let mut out = self.state.clone();
        out.version = 1;
        out.name = self.save.name.clone();
        world::before_save(self, &mut out);
        forest::before_save(self, &mut out);
        mycelium::before_save(self, &mut out);
        mushrooms::before_save(self, &mut out);
        life::before_save(self, &mut out);
        weather::before_save(self, &mut out);
        progress::before_save(self, &mut out);
        // stolen mushrooms are not saved; carried ones drop to the surface
        out.mushrooms.retain(|m| !m.stolen);
        for m in &mut out.mushrooms {
            m.stench.clear();
            m.crow_target = false;
            if m.carried_by != -1 {
                m.y = balance::wld("surface_height_px");
                m.carried_by = -1;
            }
        }
        for chunk in out.world.chunks.values_mut() {
            for lake in &mut chunk.lakes {
                lake.flares = None;
            }
        }
        let length_cm = crate::mycelium::soil::pixels_to_cm(self, out.state.length_px);
        let meta = SlotMeta {
            length_cm,
            mode: balance::game_mode(&self.state),
            name: self.save.name.clone(),
            saved: save::date_current_datetime(),
        };
        (out, meta)
    }

    /// `load_game(slot)`: read the slot and continue it (autosaves go back into the slot).
    pub fn load_game(dir: &Path, slot: usize) -> Result<Sim, LoadError> {
        let state = save::read_slot(dir, slot)?;
        let mut sim = Sim::from_save(state)?;
        sim.save.slot = slot as i64;
        sim.save_dir = Some(dir.to_path_buf());
        Ok(sim)
    }

    /// `load_game(slot)` from the title menu: the original loads into the live room, so the
    /// loaded game keeps this sim's clock (`current_time` counts from boot) and game RNG (load
    /// never reseeds it); everything load_game resets is rebuilt from the save.
    pub fn load_game_live(&self, dir: &Path, slot: usize) -> Result<Sim, LoadError> {
        let state = save::read_slot(dir, slot)?;
        let mut sim = Sim::from_save_on(state, Some((self.rng.clone(), self.frame)))?;
        sim.save.slot = slot as i64;
        sim.save_dir = Some(dir.to_path_buf());
        Ok(sim)
    }

    /// `game_restart` back to the title: a fresh room (game_init reseeds the RNG), the clock
    /// keeps counting from boot (`current_time`). GameMaker keeps global variables across a
    /// restart; those no Create event or game_init sets again survive: `global.backdrop_leaves`
    /// (backdrop_leaves_init only fills it when undefined).
    pub fn restart_title(&self, world_seed: Option<i64>) -> Sim {
        let mut sim = Sim::title_screen(world_seed);
        sim.frame = self.frame;
        sim.forest.backdrop_leaves = self.forest.backdrop_leaves.clone();
        sim
    }

    // port: load_game
    /// A Sim continuing a save (no slot bound: see [`Sim::load_game`]). Applies load_game's
    /// fixups to the data, then lets each subsystem rebuild its runtime state. Like the
    /// original, it runs on a live room: the Create sequence first (see the module docs).
    pub fn from_save(state: GameState) -> Result<Sim, LoadError> {
        Sim::from_save_on(state, None)
    }

    /// [`Sim::from_save`]; `live`: the running game's RNG and clock, which the loaded game
    /// continues (the original's load_game runs in the live room).
    fn from_save_on(mut state: GameState, live: Option<(Rng, u64)>) -> Result<Sim, LoadError> {
        if state.version != 1 {
            return Err(LoadError::Version(state.version));
        }
        // tips/stats merges and the mode-aware boar timer fallback (idempotent after read_slot)
        state.apply_load_defaults();
        let mut sim = Sim::create_room(None);
        if let Some((rng, frame)) = live {
            sim.rng = rng;
            sim.frame = frame;
        }
        sim.state = state;
        let mode = sim.state.state.mode;
        sim.game_mode_apply(mode);
        let st = &mut sim.state;
        // game_mode_apply(st.mode): `bal` follows state.mode (missing mode reads as exploration)
        if !st.state.cave_yellow.as_ref().is_some_and(|v| v.is_object()) {
            st.state.cave_yellow = None;
        }
        for chunk in st.world.chunks.values_mut() {
            for s in &mut chunk.stones {
                s.moving = false;
                s.vx = 0.0;
                s.vy = 0.0;
                s.spin = 0.0;
            }
            for lake in &mut chunk.lakes {
                lake.flares = None;
            }
        }
        let tasks = &mut st.state.tasks;
        if tasks.popup == "migration" {
            tasks.popup = "migration_ready".into();
        }
        if tasks.popup == "migration_end" {
            tasks.popup.clear();
        }
        for m in &mut st.mushrooms {
            // the host stays a save id here; Task 9 resolves it against the forest
            m.stench.clear();
            m.stench_due = 0.0;
            m.carried_by = -1;
            m.stolen = false;
            m.crow_target = false;
            m.born = None;
        }
        sim.built("world.on_load");
        world::on_load(&mut sim);
        sim.built("progress.on_load");
        progress::on_load(&mut sim);
        sim.built("forest.on_load");
        forest::on_load(&mut sim);
        sim.built("mycelium.on_load");
        mycelium::on_load(&mut sim);
        sim.built("mushrooms.on_load");
        mushrooms::on_load(&mut sim);
        sim.built("life.on_load");
        life::on_load(&mut sim);
        sim.built("weather.init");
        weather::init(&mut sim);
        sim.intro_active = false;
        sim.title_open = false;
        sim.save.name = sim.state.name.clone();
        sim.save.timer = sim.bal("save_autosave_seconds") * sim.fps;
        Ok(sim)
    }
}
