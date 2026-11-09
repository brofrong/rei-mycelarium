//! obj_game orchestration (ruling R8): `game_init` (obj_game Create_0), `game_step` and the
//! obj_game Step event. `Sim` calls [`init`] from its Create sequence and [`obj_game_step`]
//! from `step`.

use crate::balance::vis;
use crate::mycelium::soil::mycelium_cm;
use crate::progress::{self, tasks, tutorial, upgrades};
use crate::sim::{Sim, StepInput};
use crate::state::GameMode;

// port: game_init
/// `game_init`: a clock-seeded RNG, exploration mode, the progress systems, the starting
/// globals, `world_init`, the wind, `weather_init` and the camera box. (The balance/visual/
/// worldgen tables are static; locale, display, logo and the shader handles are the app's;
/// `menu_open`/`debug_open`/`hud_hidden`/`title_alpha`/`bw_mode` are HUD state.)
pub fn init(sim: &mut Sim) {
    sim.rng.randomise();
    // balance_init; game_mode_apply("exploration") (WorldRt.mode follows the state's mode)
    sim.game_mode_apply(GameMode::Exploration);
    upgrades::init(sim);
    tutorial::init(sim);
    tasks::init(sim);
    sim.save = crate::sim::SaveRt::default();
    let st = &mut sim.state.state;
    st.start_x = 0.0;
    st.food = crate::balance::bal(st.mode, "start_food");
    st.water_free = crate::balance::bal(st.mode, "start_water");
    st.water_infinite = false;
    st.cave_perks.clear();
    st.cave_taken.clear();
    st.play_frames = 0;
    st.cave_yellow = None;
    st.relic_taken.clear();
    st.length_px = 0.0;
    st.points_awarded = 0;
    sim.state.water.clear();
    sim.pause_open = false;
    sim.intro_active = true;
    sim.title_open = true;
    let start_x = sim.state.state.start_x;
    // world_init rebuilds WorldRt: water_dip/water_dip_hold and the cave globals (cave_flash,
    // cave_wrap, cave_popups, cave_burst_pending, cave_yellow) start fresh with it
    crate::world::world_init(sim, start_x);
    let dir = sim.rng.choose(&[-1.0, 1.0]);
    let speed = sim
        .rng
        .random_range(vis("wind_min_strength"), vis("wind_max_strength"));
    sim.state.state.wind = [dir, speed, vis("wind_change_interval")];
    crate::weather::init(sim);
    let margin = vis("camera_reach_margin_px");
    let half = vis("screen_width_px") * 0.5;
    sim.state.state.cam_box = [
        start_x - half - margin,
        0.0,
        start_x + half + margin,
        vis("screen_height_px"),
    ];
}

// port: game_step
/// One frame of the game rules: the yellow cave, the wind (a strong wind blows a leaf gust in
/// exploration), the upgrade tree growth, the tutorial and tasks, and upgrade points for every
/// `xp_length_step_cm` of mycelium.
pub fn step(sim: &mut Sim, input: &StepInput) {
    crate::world::caves::cave_yellow_step(sim);
    sim.state.state.wind[2] -= 1.0;
    if sim.state.state.wind[2] <= 0.0 {
        let dir = sim.rng.choose(&[-1.0, 1.0]);
        let speed = sim
            .rng
            .random_range(vis("wind_min_strength"), vis("wind_max_strength"));
        sim.state.state.wind = [dir, speed, vis("wind_change_interval")];
        if crate::balance::game_mode(&sim.state) == GameMode::Exploration
            && speed >= vis("wind_max_strength")
            && !sim.title_open
            && !sim.intro_active
        {
            crate::life::leaves::leaf_gust_spawn(sim);
        }
    }
    upgrades::grow_step(sim);
    tutorial::step(sim, input);
    tasks::step(sim, input);
    let pts = (mycelium_cm(sim) / sim.bal("xp_length_step_cm")).floor() as i64;
    let awarded = sim.state.state.points_awarded;
    if pts > awarded {
        sim.state.state.upgrade_points += pts - awarded;
        if crate::weather::is_raining(sim) {
            tasks::stat_add(sim, "rain_points", (pts - awarded) as f64);
        }
        sim.state.state.points_awarded = pts;
    }
}

// port: gml_Object_obj_game_Step_0
/// obj_game Step_0: `game_step`, `save_step`, `cheats_step` (display/focus/audio steps are the
/// app's; steam_step is skipped).
pub fn obj_game_step(sim: &mut Sim, input: &StepInput) {
    sim.mark("game.step");
    step(sim, input);
    sim.mark("save.step");
    sim.save_step();
    progress::cheats_step(sim);
}
