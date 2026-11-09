# Mycelarium на Rust + Bevy: план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Полная версия Mycelarium на Rust + Bevy в `rust/`, с геймплеем и цифрами оригинала, сейвами,
общими с LÖVE-портом, и тремя новшествами: ПКМ ставит узел, зум камеры, всплывающие `+N/−N`.

**Architecture:** Два крейта.
- `myco_core` — чистая детерминированная симуляция без Bevy:
  - состояние игры сериализуется прямо в формат сейва LÖVE-порта;
  - шаг 60 Гц принимает типизированный ввод и выдаёт события;
  - каждая функция оригинала перенесена в типизированный Rust с маркером `// port: <имя>`.
- `myco_app` — Bevy-приложение:
  - отрисовка: сущности и меши, синхронизированные с симуляцией по стабильным id;
  - UI, звук, ввод, окно, автопилот.

Эталон поведения — LÖVE-порт: golden-данные снимаются из него скриптами и сверяются в тестах Rust.

**Tech Stack:**
- Rust stable (rustup) и Bevy последней стабильной версии;
- serde/serde_json для сейвов;
- LÖVE 11.5 (`build/tools/love.app`) для golden-данных;
- Python 3 (`python3 -I`) для инструментов покрытия.

**Spec:** `docs/superpowers/specs/2026-11-09-mycelarium-rust-bevy-design.md`

## Global Constraints

- **Код:** только в `rust/` (workspace) плюс точечные правки `game/` (golden-хуки, исправления
  расхождений) и `tools/`. Ассеты и exe не коммитить: `build/` уже в `.gitignore`.
- **Цель сборки:** только macOS, Apple Silicon.
- **Баланс точно как в оригинале.** Числа в `myco_core` — `f64`, как double в GML. Округление
  `gml::round` — банковское (half-even), как `gm.round` (`game/gm/mathx.lua`).
- **Время.** Логика идёт строго 60 шагов в секунду. Таймеры в сейве считаются в кадрах.
- **Сейвы:**
  - папка `~/Library/Application Support/LOVE/Mycelarium/`;
  - файлы `save_<slot+1>.json`, `save_<slot+1>_meta.json`, `settings.ini`;
  - JSON с завершающим `\0`: отбросить при чтении, дописать при записи;
  - запись через временный файл;
  - неизвестные поля сохраняются как есть.
- **Источник логики:** `game/myco/gen/*.lua`. При расхождении с `re/lifted/` или `re/decomp/` права
  декомпиляция, и расхождение чинится в обеих версиях тем же коммитом.
- **Без заглушек.** Функции, назначенные задаче в `rust/portmap.tsv`, переносятся полностью. Задача не
  закрыта, пока `python3 -I tools/port_coverage.py --task <T>` не выходит с кодом 0.
- **Экран:** по умолчанию полный экран ниже чёлки; логический вид по правилу `view_for_window`
  (`game/gm/display_math.lua`); UI шириной 1920 и привязан к низу. Зум влияет только на камеру
  отрисовки. Для логики всегда игровой вид 1×, `ViewRect` с тем же центром.
- **Субагенты:** только модель opus, effort medium. Без вложенных агентов, без worktree, без коммитов от
  субагентов, если задача не говорит иначе.
- **Коммиты:** сообщение заканчивается строкой
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **Мышь попадает точно.** Это исходная беда GameHub: клик должен приходиться ровно в точку под
   курсором в мире при любом зуме, на Retina (×2), в полном экране ниже чёлки, после переключения
   окно ↔ полный экран и после ресайза окна. Тест в Task 4: `screen_to_world` при зуме 0.37,
   смещении safe area 64 px и масштабе 2.
2. **ПКМ не по месту:**
   - над HUD — ничего не делает;
   - в режиме постановки гриба — отменяет режим, а не ставит узел;
   - при зажатой ЛКМ (во время прицеливания) — игнорируется;
   - многократный ПКМ в одной точке ставит один узел, потом даёт отказ `TooCloseToNode`.

   Тесты в Task 8 и Task 17.
3. **Старые и битые сейвы.**
   - Сейв без части полей (раннее сохранение, другой режим) загружается со значениями по умолчанию.
   - Обрезанный или битый JSON не роняет меню: слот показан как повреждённый, игра продолжает работать.

   Тест в Task 2.
4. **Маленькая колония и зум.** Когда разведано меньше одного экрана, предел отдаления не меньше
   обычного масштаба 1×. Зум не уводит камеру за верх неба и низ мира. Тест в Task 21.
5. **Шторм всплывающих чисел.**
   - Много грибов в одном кадре: на каждый источник одно число за кадр.
   - Источники за экраном не создают числа.
   - Видимых чисел не больше 64, лишние сливаются в ближайшие.

   Тест в Task 17.

---

## Карта файлов

```
rust/
  Cargo.toml                      workspace: crates/myco_core, crates/myco_app
  portmap.tsv                     функция оригинала -> задача (T5..T21) или skip:<причина>
  golden/scripts/<topic>.lua      скрипты, снимающие эталон из LÖVE-порта
  golden/data/<topic>.json        снятые эталоны (коммитятся: это числа, не ассеты)
  crates/myco_core/
    src/lib.rs
    src/gml.rs                    семантика GML-математики (round, lerp, point_direction, ...)
    src/rng.rs                    xorshift64* из game/gm/random.lua, побитово
    src/balance.rs                bal/vis/wld, game_mode, experience_price, perk_cost_mult
    src/locale.rs                 localization.csv, loc(key), выбор шрифта по языку
    src/state.rs                  GameState = формат save_N.json (serde), SlotMeta, DisplaySettings
    src/save.rs                   чтение/запись слотов, settings.ini, путь папки
    src/sim.rs                    Sim, StepInput, SimEvent, Command, порядок шага
    src/events.rs                 SimEvent, ResourceKind, NodeReject, SoundCue
    src/world/{mod,hash,gen,soil,lakes,water,food,survival,obstacles}.rs   Task 5
    src/world/{caves,relics,perks,bones}.rs                                 Task 6
    src/world/{stones,maze}.rs                                              Task 7
    src/mycelium/{mod,shot,place,knots,growth,soil}.rs                      Task 8
    src/mushrooms/{mod,types,grow,produce,repeller,stinky,consumptor}.rs    Task 9
    src/life/{mod,hedgehog,squirrel,crow,boar,worm,leaves,predator,drops}.rs Task 10
    src/progress/{upgrades,skilltree,tasks,goals,tutorial}.rs               Task 11
    src/weather.rs                                                          Task 12
    tests/common/mod.rs           golden(topic) -> serde_json::Value, approx-сравнение
    tests/<topic>.rs              тесты по задачам
  crates/myco_app/
    src/main.rs                   App, плагины, состояния
    src/assets.rs                 текстурные страницы, спрайты по имени, шрифты, звуки
    src/display.rs                окно, полный экран ниже чёлки, view_for_window, settings.ini
    src/camera.rs                 игровой ViewRect, камера отрисовки, зум, camera_jump/fly
    src/autopilot.rs              MYCO_INPUT/MYCO_PROBE-аналоги, дамп состояния, скриншот
    src/sim_bridge.rs             Resource<Sim>, FixedUpdate 60 Гц, сбор StepInput, раздача SimEvent
    src/render/{world,mycelium,mushrooms,life,forest,weather,fx}.rs
    src/render/shaders/{blackmetal,glow_blur,stinky}.wgsl
    src/ui/{hud,popups,cursor_tip,placement,tasks,tree,menu,slots,name,settings,credits,pause}.rs
    src/logo.rs                   процедурный логотип меню (scr_logo)
    src/cine.rs                   интро и кат-сцены
    src/audio.rs                  scr_audio: sfx, петли, эмбиент, музыка, звук гиф
tools/
  golden.sh                       снять golden: tools/golden.sh <topic>
  portmap_init.py                 первичное заполнение rust/portmap.tsv
  port_coverage.py                проверка маркеров // port:
  package_rust.sh                 dist/Mycelarium Rust.app
game/main.lua                     + MYCO_SEED, MYCO_GOLDEN, MYCO_DUMP
```

## Общие интерфейсы (определяются в Task 1–3, все задачи опираются на них)

```rust
// myco_core::gml
pub fn round(x: f64) -> f64;                 // half-even, как gm.round
pub fn lerp(a: f64, b: f64, t: f64) -> f64;
pub fn clamp(x: f64, lo: f64, hi: f64) -> f64;
pub fn point_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64;
pub fn point_direction(x1: f64, y1: f64, x2: f64, y2: f64) -> f64; // градусы, y вниз
pub fn lengthdir_x(len: f64, dir: f64) -> f64;
pub fn lengthdir_y(len: f64, dir: f64) -> f64;
pub fn angle_difference(a: f64, b: f64) -> f64;
pub fn gmod(a: f64, b: f64) -> f64;          // как gm.mod (fmod, знак делимого)
pub fn frac(x: f64) -> f64;  pub fn sign(x: f64) -> f64;

// myco_core::rng — побитово как game/gm/random.lua
pub struct Rng { state: u64, seed: i64 }
impl Rng {
  pub fn new(seed: i64) -> Self;           // = random_set_seed
  pub fn seed(&self) -> i64;
  pub fn random(&mut self, n: f64) -> f64;
  pub fn random_range(&mut self, a: f64, b: f64) -> f64;
  pub fn irandom(&mut self, n: f64) -> i64;
  pub fn irandom_range(&mut self, a: f64, b: f64) -> i64;
  pub fn choose<T: Copy>(&mut self, items: &[T]) -> T;
  pub fn randomise(&mut self) -> i64;
}

// myco_core::events
pub enum ResourceKind { Food, Water, UpgradePoints }
pub enum NodeReject { NotOnMycelium, InWater, TooCloseToNode, NotEnoughFood { need: i64 }, Blocked }
pub enum SimEvent {
  Resource { kind: ResourceKind, delta: f64, x: f64, y: f64, source: u64 }, // source: стабильный id источника
  NodeRejected { reason: NodeReject, x: f64, y: f64 },
  Sound(SoundCue),                       // определяется в Task 3, расширяется задачами логики
  Saved, GameOver, TutorialTip { key: String },
}

// myco_core::sim
pub struct ViewRect { pub x: f64, pub y: f64, pub w: f64, pub h: f64 }   // игровой вид 1×
pub struct ButtonState { pub down: bool, pub pressed: bool, pub released: bool }
pub struct StepInput {
  pub cursor: Option<(f64, f64)>,          // мир, px; None — курсор вне окна
  pub cursor_gui: Option<(f64, f64)>,      // GUI-координаты (1920 по ширине)
  pub lmb: ButtonState, pub rmb: ButtonState,
  pub keys_pressed: Vec<Key>, pub keys_down: Vec<Key>, pub text: String,
  pub ui_captured: bool,                   // курсор над HUD/меню
  pub view: ViewRect,
  pub commands: Vec<Command>,
}
pub enum Command {
  QuickNode { x: f64, y: f64 },            // ПКМ
  PlaceMushroom { kind: MushroomKind, x: f64, y: f64 },
  BuyUpgrade { id: String },
  SetPaused(bool), Save,
  // задачи 9–12 добавляют свои варианты здесь, остальное не трогают
}
pub struct Sim {
  pub state: GameState, pub rng: Rng, pub frame: u64,
  pub world: world::WorldRt, pub mycelium: mycelium::MyceliumRt, pub mushrooms: mushrooms::MushroomsRt,
  pub life: life::LifeRt, pub progress: progress::ProgressRt, pub weather: weather::WeatherRt,
  events: Vec<SimEvent>,
}
impl Sim {
  pub fn new_game(mode: GameMode, name: &str, world_seed: Option<i64>) -> Sim;
  pub fn from_save(state: GameState) -> Result<Sim, LoadError>;
  pub fn to_save(&self) -> (GameState, SlotMeta);
  pub fn step(&mut self, input: &StepInput);
  pub fn emit(&mut self, e: SimEvent);
  pub fn drain_events(&mut self) -> Vec<SimEvent>;
}
```

Порядок внутри `Sim::step` повторяет порядок экземпляров и событий оригинала:
- экземпляры: `obj_game` → `obj_forest_controller` → `obj_mycelium_controller` → `obj_life` → `obj_hud`
  (`build/assets/rooms.json`, Room1);
- в каждом кадре: Begin Step всех экземпляров → Step всех → End Step всех.

Task 3 вписывает в `step` вызовы `world::begin_step/step`, `mycelium::...` и т.д. в этом порядке.
Модули задач 5–12 реализуют эти функции и не трогают `sim.rs`, кроме своих вариантов `Command`.

---

### Task 1: Тулчейн, workspace, карта переноса, golden-хуки в LÖVE

**Files:**
- Create:
  - `rust/Cargo.toml`, `rust/crates/myco_core/{Cargo.toml,src/lib.rs,src/gml.rs,src/rng.rs}`;
  - `rust/crates/myco_core/tests/{common/mod.rs,gml.rs,rng.rs}`;
  - `rust/golden/scripts/{gml,rng}.lua`, `rust/golden/data/{gml,rng}.json`;
  - `tools/golden.sh`, `tools/portmap_init.py`, `tools/port_coverage.py`, `rust/portmap.tsv`;
  - `tools/tests/test_port_coverage.py`.
- Modify: `game/main.lua` (MYCO_GOLDEN, MYCO_SEED, MYCO_DUMP), `game/myco/gen/*` — только
  место `world_init`, где берётся сид, чтобы читать MYCO_SEED; `.gitignore` (`rust/target/`).

**Interfaces:**
- Produces:
  - `gml::*` и `rng::Rng` (смотри «Общие интерфейсы»);
  - `tests/common::golden(topic: &str) -> serde_json::Value`;
  - `tests/common::assert_close(a: f64, b: f64, rel: f64)`;
  - CLI `tools/golden.sh <topic>` пишет `rust/golden/data/<topic>.json`;
  - CLI `python3 -I tools/port_coverage.py --task T<N> | --all`, код выхода 1, если есть
    непокрытые функции.
- Env LÖVE-порта:
  - `MYCO_GOLDEN=<script.lua>` и `MYCO_GOLDEN_OUT=<file.json>`: после загрузки игры (первый кадр
    Room1 или меню) выполнить скрипт, который возвращает таблицу; записать её в JSON с числами в
    `%.17g` и выйти;
  - `MYCO_SEED=<int>` — сид мира вместо `irandom(99999)`;
  - `MYCO_DUMP=<кадр>:<file.json>` — записать `save_full`-эквивалент текущего состояния, без записи
    слота.

- [ ] **Step 1:** Установить тулчейн: `brew install rustup && rustup-init -y --default-toolchain stable`.
  Проверка: `~/.cargo/bin/cargo --version` печатает версию.
- [ ] **Step 2:** Тест на `port_coverage.py` в `tools/tests/test_port_coverage.py`.
  - `test_missing_function_reported`: временный portmap с `foo T5` и Rust-файл без маркера →
    код выхода 1, в выводе `foo`.
  - `test_marker_covers`: файл с `// port: foo` → код выхода 0.
  - `test_alias_marker_covers`: строка portmap `anon__1__x  myc_foo  T8`, маркер `// port: myc_foo` →
    покрыто.
  - `test_skip_not_required`: `bar skip:steam` не требует маркера.
- [ ] **Step 3:** Запустить `python3 -I -m pytest tools/tests/test_port_coverage.py -q`. Ожидание:
  FAIL (модуля нет).
- [ ] **Step 4:** Написать `tools/port_coverage.py` и `tools/portmap_init.py`.
  - **Формат `rust/portmap.tsv`:** колонки `name<TAB>alias<TAB>task`.
  - **name:** каждое `^function NAME` в `game/myco/gen/*.lua`, кроме `___struct___*`
    (структуры — это форма данных, их маркировать не нужно).
  - **alias:** имя из `self.ALIAS = gm.method(self, NAME)`; если алиаса нет, `-`.
  - **Начальное назначение задач:**

    | Функции | Задача |
    |---|---|
    | `world_*`, `lake*`, `water_*`, `world_food*` из `scr_world_b_*`; `worldgen_*` | T5 |
    | `cave*`, `relic*`, `perk*`, `bone*`, `caves_*` и прочее из `scr_world_a_*` | T6 |
    | `scr_stones_*` | T7 |
    | алиасы `myc_*`, функции `myc_*`, `mycelium_*`, `length*`, `reach*`, `pixels_to*` | T8 |
    | `mushroom*`, `shroom*`, `repeller*`, `stinky*`, `consum*`, `tree_shroom`, `experience*` | T9 |
    | прочее из `scr_life_*`, кроме `intro*`, `backdrop*`, `camera_fly`, `animal_marker*` | T10 |
    | `upg*`, `upgrades*`, `tasks*`, `goal*`, `tutorial*`, `cheat*`, `food*`, `balance*` | T11 |
    | `weather*`, `rain*` | T12 |
    | `bal`, `vis`, `wld`, `game_mode`, `perk_cost_mult`, `loc*`, `locale*`, `save*`, `load*` | T3 |
    | `world_draw*`, `world_paint*`, `food_glow*` | T13 |
    | `tree_*` из `ui_1` | T19 |
    | `logo*` | T18 |
    | `cine*`, `intro*`, `camera_*` | T21 |
    | `sound*`, `sfx*`, `audio*`, `music*`, `ambience*`, `hyphae_sound*`, `rain_sound`, `forest_sound`, `water_touch` | T20 |
    | алиасы `hud_*`, функции `tasks_popup*`/`tasks_block*` (отрисовка), `hover_bar*` | T17 |
    | алиасы `forest_*`, `backdrop*`, `leaf_fg*` | T16 |
    | `Steam*`, `ach_*`, `ExtensionCore*`, `steam_*`, `gml_*` (обёртки GameMaker) | `skip:<причина>` |
    | остальное | `T?` |

  - **`T?`:** инструмент печатает эти строки; Step 6 разбирает их руками.
- [ ] **Step 5:** `python3 -I -m pytest tools/tests -q` → PASS (вместе со старыми 29 тестами).
- [ ] **Step 6:** `python3 -I tools/portmap_init.py > rust/portmap.tsv`, затем вручную разнести все
  `T?`.
  - Ориентиры: файл чанка, что функция рисует или считает, кто её вызывает (`re/callgraph.json`).
  - Проверка: `grep -c 'T?' rust/portmap.tsv` → 0.
- [ ] **Step 7:** Golden-хуки в `game/main.lua` и сид в `world_init`.
  - Скрипт `rust/golden/scripts/rng.lua`: `random_set_seed(12345)`, затем 50 значений `random(1)`,
    20 `irandom(10)`, 20 `irandom_range(-5,5)`, 20 `choose(1,2,3)`.
  - Скрипт `gml.lua`: таблица входов и выходов для `round`, `point_direction`, `lengthdir_x/y`,
    `angle_difference`, `gm.mod`, включая отрицательные числа и `x.5`.
  - Запуск: `tools/golden.sh rng && tools/golden.sh gml`.
  - Проверка: оба JSON непустые; `MYCO_SEED=31607` даёт `world.seed == 31607` в `MYCO_DUMP`.
- [ ] **Step 8:** Тесты Rust `tests/rng.rs::matches_love_sequence` и
  `tests/gml.rs::matches_love_table` читают golden и сравнивают побитово для rng и с
  `assert_close(.., 1e-12)` для тригонометрии.
  Запуск: `cd rust && cargo test -p myco_core` → FAIL (нет реализации).
- [ ] **Step 9:** Реализовать `gml.rs`, `rng.rs`. Сдвиги и умножения xorshift64* — `u64` wrapping, как
  `uint64_t` в LuaJIT. `next01 = (x*2685821657736338717 >> 11) / 2^53`.
  `cargo test -p myco_core` → PASS.
- [ ] **Step 10:** Коммит
  `feat(rust): workspace, GML math and RNG matching LÖVE, golden hooks, port map`.

### Task 2: Модель сейва и чтение/запись слотов

**Files:**
- Create: `rust/crates/myco_core/src/{state.rs,save.rs}`, `tests/save.rs`,
  `rust/golden/scripts/save_fresh.lua`.
- Fixtures: `rust/crates/myco_core/tests/fixtures/{exploration,survival,old_minimal,truncated}.json`.
  Exploration и survival снять через `MYCO_DUMP` из LÖVE с `MYCO_SEED=31607` после короткой сессии
  автопилота: два выстрела, узел. Это данные сида, не ассеты, их можно коммитить.

**Interfaces:**
- Consumes: ничего, кроме serde.
- Produces:
  ```rust
  pub struct GameState { pub version: i64, pub name: String, pub forest: ForestSave, pub mycelium: MyceliumSave,
    pub mushrooms: Vec<MushroomSave>, pub water: Vec<WaterSave>, pub state: RunState, pub world: WorldSave,
    #[serde(flatten)] pub extra: serde_json::Map<String, Value> }
  pub struct SlotMeta { pub length_cm: f64, pub mode: GameMode, pub name: String, pub saved: f64 }
  pub enum GameMode { Exploration, Survival }   // serde: "exploration" | "survival"
  pub struct DisplaySettings { pub fullscreen: bool, pub window_w: u32, pub window_h: u32 }
  pub enum SlotStatus { Empty, Ok(SlotMeta), Corrupt(String) }
  pub fn save_dir() -> PathBuf;   // ~/Library/Application Support/LOVE/Mycelarium
  pub fn read_slot(dir: &Path, slot: usize) -> Result<GameState, SaveError>;
  pub fn write_slot(dir: &Path, slot: usize, state: &GameState, meta: &SlotMeta) -> io::Result<()>;
  pub fn list_slots(dir: &Path, count: usize) -> Vec<SlotStatus>;
  pub fn read_settings(dir: &Path) -> DisplaySettings;  pub fn write_settings(dir: &Path, s: &DisplaySettings) -> io::Result<()>;
  ```
- Поля структур берутся из функций сохранения и загрузки в `game/myco/gen/meta_3.lua` и
  вызываемых из них сериализаторов. В каждой вложенной структуре стоит
  `#[serde(flatten)] extra`.
- Числа хранятся в типе, который пишет LÖVE: целые как `i64`, если в JSON целые. `water_free` и
  подобные — `f64`.
- Поля-словари (`cave_perks`, `upgrade_levels`, `tasks.done`) — `BTreeMap<String, Value|i64|bool>`.

- [ ] **Step 1:** Тесты в `tests/save.rs`:
  - `roundtrip_exploration_preserves_values`: read → write во временную папку → read → значения
    равны. Сравнение через `serde_json::Value` с допуском 1e-9 для чисел.
  - `roundtrip_survival_preserves_values`: то же для survival.
  - `file_ends_with_nul_and_tmp_removed`: последний байт файла `0`, временного файла после записи нет.
  - `unknown_fields_survive`: в JSON добавлено `"future_field": {"a": 1}` на верхнем уровне и в
    `state`; после round-trip оба на месте.
  - `old_minimal_loads_with_defaults`: Review Focus 3. Сейв без `state.tasks`, `state.timers`,
    `cave_perks` грузится, значения по умолчанию соответствуют `save_fresh.lua`.
  - `truncated_is_corrupt_not_panic`: `list_slots` даёт `SlotStatus::Corrupt` для обрезанного файла.
  - `settings_ini_matches_love`: строка `[display]\r\nfullscreen="1.000000"\r\n...` разбирается, и
    `write_settings` даёт байт-в-байт тот же текст (`game/gm/display_math.lua: settings_to_ini`).
- [ ] **Step 2:** `cargo test -p myco_core --test save` → FAIL.
- [ ] **Step 3:** Реализовать `state.rs`, `save.rs`. Значения по умолчанию для отсутствующих полей
  взять из `save_fresh.lua` (golden нового сейва Exploration при `MYCO_SEED=31607`).
- [ ] **Step 4:** `cargo test -p myco_core --test save` → PASS.
- [ ] **Step 5:** Коммит `feat(rust): typed save model compatible with LÖVE saves`.

### Task 3: Баланс, локализация, каркас симуляции

**Files:**
- Create: `rust/crates/myco_core/src/{balance.rs,locale.rs,events.rs,sim.rs}`.
- Create: пустые модули с `Default`-структурами `WorldRt`, `MyceliumRt`, `MushroomsRt`, `LifeRt`,
  `ProgressRt`, `WeatherRt` и функциями-заглушками шага в `src/{world,mycelium,mushrooms,life,progress}/mod.rs`
  и `src/weather.rs`. Это единственные разрешённые заглушки: их заполняют задачи 5–12.
- Tests: `tests/{balance,locale,sim_frame}.rs`, golden `rust/golden/scripts/{balance,locale}.lua`.

**Interfaces:**
- Consumes: Task 1 (`gml`, `Rng`), Task 2 (`GameState`).
- Produces:
  - `balance::{bal(key: &str) -> f64, vis(key: &str) -> f64, wld(key: &str) -> f64}`. Таблицы — константы
    Rust, сгенерированные из Lua-исходников `bal`/`vis`/`wld` (`core_1.lua:258`, `core_3.lua:1234`,
    `core_3.lua:1419`). Неизвестный ключ — паника в debug, а не тихий 0.
  - `balance::game_mode(&GameState) -> GameMode`.
  - `balance::perk_cost_mult(&GameState) -> f64`.
  - `balance::experience_price(..)`: сигнатура как в Lua, типы явные.
  - `locale::Locale { fn load(csv: &str) -> Locale; fn set_language(&mut self, lang: Lang);
    fn loc(&self, key: &str) -> &str; fn font_for(&self) -> FontId }`.
  - `Lang` — 12 языков из заголовка `localization.csv` (с BOM).
  - Трейт `world::WorldQuery` в `world/mod.rs`, его реализует `WorldRt` в Task 5; задачи 8–10
    пишутся против трейта:
    ```rust
    pub trait WorldQuery {
      fn tile_at(&self, x: f64, y: f64) -> Tile;
      fn is_solid(&self, x: f64, y: f64) -> bool;
      fn water_shown_at(&self, x: f64, y: f64) -> bool;
      fn soil_at(&self, x: f64, y: f64) -> Soil;
      fn surface_y(&self) -> f64;
    }
    ```
    `Tile` и `Soil` — enum по кодам тайлов из `scr_world`.
  - `events::SoundCue { name: String, x: Option<f64>, y: Option<f64>, gain: f64, pitch: f64 }`:
    `name` — имя звука из `build/assets/sounds`.
  - Все типы из «Общих интерфейсов» в `events.rs` и `sim.rs`. `Sim::step` вызывает шаги
    подсистем в порядке оригинала. `Sim::new_game` повторяет `obj_game` Create + `world_init` +
    `world_start_guarantee` через функции подсистем: пока они пустые, этот вызов остаётся
    TODO-ссылкой на Task 5, а тест `sim_frame` проверяет только счётчик кадров и порядок.

- [ ] **Step 1:** Тесты.
  - `balance_matches_love`: все ключи `bal`/`vis`/`wld`, которые встречаются в
    `game/myco/gen/*.lua` (собрать grep'ом в golden-скрипте), равны значениям LÖVE.
  - `locale_matches_love`: 30 ключей на каждом из 12 языков совпадают с LÖVE. Сюда входят
    `button_add_node` и `hint_no_nodes_in_water`.
  - `sim_frame::step_advances_frame_and_calls_in_order`: тестовый журнал вызовов шагов подсистем
    (через `#[cfg(test)]`-хук) равен ожидаемому порядку begin/step/end.
- [ ] **Step 2:** `cargo test -p myco_core` → FAIL.
- [ ] **Step 3:** Реализовать. Поставить маркеры `// port:` на перенесённые функции.
- [ ] **Step 4:** `cargo test -p myco_core` → PASS; `python3 -I tools/port_coverage.py --task T3` → 0.
- [ ] **Step 5:** Коммит `feat(rust): balance tables, localization, simulation skeleton`.

### Task 4: Оболочка Bevy: ассеты, экран, мост симуляции, автопилот

**Files:**
- Create: `rust/crates/myco_app/{Cargo.toml,src/main.rs,src/assets.rs,src/display.rs,src/camera.rs,src/sim_bridge.rs,src/autopilot.rs}`.
- Tests: `rust/crates/myco_app/tests/display.rs`; юнит-тесты в `camera.rs`.

**Interfaces:**
- Consumes: `Sim`, `StepInput`, `ViewRect`, `DisplaySettings`, `read_settings`.
- Produces:
  - `assets::GameAssets` (Resource):
    - `sprite(name: &str) -> &SpriteDef`;
    - `SpriteDef { frames: Vec<FrameRef>, origin: Vec2, size: Vec2 }`;
    - `FrameRef { page: Handle<Image>, rect: Rect, trim_offset: Vec2 }`;
    - `font(FontId) -> Handle<Font>`;
    - `sound(name: &str) -> Handle<AudioSource>`.

    Читается из `build/assets` (путь из env `MYCO_ASSETS` или `<app>/Contents/Resources/assets`)
    по тем же JSON, что `game/gm/assets.lua`.
  - `display`:
    - `view_for_window(ww: f64, wh: f64) -> ViewLayout`: порт `display_math.lua`;
    - полный экран ниже чёлки: borderless на safe area. NSScreen safeAreaInsets через
      `objc2-app-kit` или winit API; при отсутствии — весь экран;
    - переключение окно ↔ полный экран пишет `settings.ini`.
  - `camera::Cameras` (Resource):
    - `game_view: ViewRect` — 1×, его центр задаёт логика камеры;
    - `zoom: f64`;
    - `screen_to_world(p: Vec2) -> DVec2`;
    - `world_to_screen(..)`;
    - `screen_to_gui(..)`.
  - `sim_bridge`:
    - `Resource<SimRes(Sim)>`;
    - система в `FixedUpdate` с `Time<Fixed>` = 60 Гц;
    - ввод собирается в `Update` в `PendingInput`: `pressed`/`released` защёлкиваются до ближайшего
      фиксированного шага и сбрасываются после него;
    - события симуляции раздаются как Bevy `Event<SimEventMsg>`.
  - `autopilot`:
    - `MYCO_INPUT` в том же формате и с теми же действиями, что `game/main.lua`;
    - `MYCO_DUMP=<кадр>:<file>` пишет `Sim::to_save()`;
    - `MYCO_SHOT=<file>:<кадр>`;
    - `MYCO_SEED`;
    - `MYCO_FOCUS` (игнор потери фокуса).

- [ ] **Step 1:** Тесты.
  - `view_for_window_matches_lua`: те же случаи, что в `tests/test_display_math.lua`:
    3024×1898, 1280×720, 1000×1000, 4000×1000.
  - `screen_to_world_roundtrip_with_zoom` (Review Focus 1): масштаб 2, safe area сверху 64 px,
    зум 0.37. Для 5 точек `world_to_screen(screen_to_world(p)) == p` с допуском 1e-6. Точка в центре
    экрана мапится в центр `game_view`.
  - `latched_click_survives_frame_without_fixed_step`: два `Update` без `FixedUpdate`, клик в
    первом → ровно один `pressed` в следующем шаге симуляции.
- [ ] **Step 2:** `cargo test -p myco_app` → FAIL.
- [ ] **Step 3:** Реализовать. Временная сцена: спрайт `spr_mushroom...` (любой из `sprites.json`)
  рисуется в центре вида, текст «Mycelarium» шрифтом из `GameAssets`.
- [ ] **Step 4:** Тесты → PASS.
  Проверка:
  ```
  MYCO_ASSETS=build/assets MYCO_SHOT=/tmp/r.png:30 cargo run -p myco_app --release
  ```
  На скриншоте полный экран ниже чёлки, спрайт и текст.
- [ ] **Step 5:** Коммит `feat(rust): Bevy shell with assets, display, fixed-step bridge, autopilot`.

---

**Задачи 5–12 (логика) независимы друг от друга** после Task 3 и могут идти параллельно. Общий шаблон
каждой из них:

1. Golden-скрипт `rust/golden/scripts/<topic>.lua` вызывает перенесённые функции LÖVE-порта на
   наборе входов. Для методов контроллера через `gm.instance_find(obj_mycelium_controller, 0)`, на
   мире `MYCO_SEED=31607` и `MYCO_SEED=7` и, где нужно, на загруженной фикстуре из Task 2. Скрипт
   сохраняет выходы.
2. Тесты в `tests/<topic>.rs` сравнивают выходы Rust с golden. Допуск 1e-9 для `f64`, точное
   равенство для целых, bool, enum и структур.
3. Перенос функций задачи из `rust/portmap.tsv` с маркерами `// port: <имя>`. Типы явные,
   строковые теги оригинала становятся enum.
4. `cargo test -p myco_core` → PASS; `python3 -I tools/port_coverage.py --task T<N>` → 0.
5. Найденные ошибки перевода в Lua исправить в `game/myco/gen/` с тестом в `tests/test_*.lua`.
   После этого golden снимается заново.
6. Коммит `feat(rust-core): <тема>`.

Ниже для каждой задачи только то, что шаблон не определяет.

### Task 5: Мир: хеши, чанки, слои, почва, озёра, вода, еда, survival

**Files:** `src/world/{mod,hash,gen,soil,lakes,water,food,survival,obstacles}.rs`, `tests/world.rs`.

**Interfaces:**
- `world::hash::{world_hash01(a: f64, b: f64, c: f64) -> f64, cave_hash(a: f64, b: f64, c: f64) -> f64}`:
  побитово как Lua, это основа совместимости миров.
- `WorldRt` хранит чанки. Методы:
  - `ensure_chunk(&mut self, st: &mut GameState, index: i64)`;
  - `tile_at(x: f64, y: f64) -> Tile`;
  - `is_solid(x, y) -> bool`;
  - `soil_at(x, y) -> Soil`;
  - `water_shown_at(x, y) -> bool`: нужен Task 8 для отказа `InWater`;
  - `food_at(..)`.
- `world::new_world(st: &mut GameState, rng: &mut Rng, start_x: f64, seed: i64)` = `world_init` +
  `world_start_guarantee`. Этим заполняется TODO в `Sim::new_game`.

**Тесты:**
- `hash_matches_love`: 10 000 троек.
- `chunk_tiles_match_love`: для сидов 31607 и 7 все тайлы чанков −2..2 и слоёв 0..40 совпадают
  (golden — плоский массив кодов тайлов).
- `survival_layout_matches_love`.
- `fixture_world_loads_identically`: Exploration-фикстура → `WorldRt` → тайлы совпадают с LÖVE после
  загрузки того же сейва.

### Task 6: Пещеры, реликвии, перки, кости

**Files:** `src/world/{caves,relics,perks,bones}.rs`, `tests/caves.rs`.

**Interfaces:**
- Методы `WorldRt`:
  - `caves(&self) -> &[Cave]`;
  - `touch_caves(st: &mut GameState, x, y, r) -> Vec<CaveTouch>`;
  - `relics_touch(..)`.
- `perks::perk_cost(..)`, `perks::perk_time(..)`.
- `SimEvent::Resource` при наградах из пещер, источник — id пещеры.

**Тесты:**
- `caves_match_love`: позиции, форма, ветви для двух сидов.
- `cave_touch_rewards_match_love`: награды при касании по фикстуре.
- `perk_costs_match_love`.

### Task 7: Камни и лабиринты

**Files:** `src/world/{stones,maze}.rs`, `tests/stones.rs`.

**Interfaces:**
- `WorldRt::stones_in(rect) -> impl Iterator<Item=&Stone>`;
- `WorldRt::pick_stone(st, x, y) -> Option<StonePick>`: камень подобран, сколько ресурса;
- `maze::*` для генерации.
- Подбор камня эмитит `SimEvent::Resource`, источник — id камня. Это требование спеки про
  `+N` за камни.

**Тесты:**
- `stone_layout_matches_love`.
- `maze_matches_love`: `maze_rng` даёт последовательности как в LÖVE.
- `stone_pick_reward_matches_love`.

### Task 8: Мицелий: гифы, выстрел, узлы, ПКМ-узел, длина, вода

**Files:** `src/mycelium/{mod,shot,place,knots,growth,soil}.rs`, `tests/mycelium.rs`.

**Interfaces:**
- `MyceliumRt`; типы `Growth`, `Knot` с полями из `MyceliumSave` (Task 2).
- `mycelium::step(sim: &mut Sim, input: &StepInput)`: прицеливание и выстрел ЛКМ, как в
  `obj_mycelium_controller` Step.
- `mycelium::quick_node(sim: &mut Sim, x: f64, y: f64) -> Result<(), NodeReject>` — порт
  `myc_node_quick` (`anon__19580`) + `myc_apply_build_option` для узла (`anon__20040`). Причины
  отказа по порядку проверок оригинала:
  1. нет гифы в радиусе `myc_mycelium_pick_radius` → `NotOnMycelium`;
  2. `world_water_shown_at` → `InWater`;
  3. `myc_has_knot_near(.., knot_radius*2)` → `TooCloseToNode`;
  4. `food < myc_node_food_cost()` → `NotEnoughFood { need }`;
  5. `myc_add_knot` вернул false → `Blocked`.

  При успехе эмитить `Resource { Food, -cost }` в точке узла и событие туториала `"node"`.
- `Command::QuickNode` обрабатывается в `mycelium::step`. Отказ эмитит
  `SimEvent::NodeRejected`.
- Правила ПКМ (Review Focus 2): команда игнорируется, если в этом шаге `input.lmb.down`
  (идёт прицеливание) или `input.ui_captured`.
- Расход воды на выстрел эмитит `Resource { Water, -n }`, впитывание воды — `+n`.

**Тесты:**
- `shot_length_matches_love`: 5 сценариев выстрела по фикстуре, длина и путь гифы как в LÖVE.
- `node_cost_matches_love`.
- `quick_node_rejections`: по одному на каждую причину. Повторный ПКМ в той же точке даёт
  `TooCloseToNode`, и узел ровно один.
- `rmb_ignored_while_aiming`.
- `playability_matches_love`: `myc_segment_is_playable` с `r = 1` (случай из падения `baff3b6`).
- `length_cm_matches_love`.

### Task 9: Грибы

**Files:** `src/mushrooms/{mod,types,grow,produce,repeller,stinky,consumptor}.rs`, `tests/mushrooms.rs`.

**Interfaces:**
- `MushroomKind` — enum из `mushroom_type_*`.
- `mushrooms::{cost, water_cost, affordable, unlocked}(..)`.
- `mushrooms::can_build_at(sim, kind, x, y) -> Result<(), BuildReject>`.
- `Command::PlaceMushroom` → `build_at`.
- Производство еды и воды каждым грибом эмитит `Resource { kind, +n, x, y, source: mushroom_id }`.
  Это главное требование спеки про `+N` у грибов. Начисление в `state` и событие — в одном месте
  кода.

**Тесты:**
- `costs_match_love` (с перками).
- `production_over_600_frames_matches_love`: по фикстуре с грибами, сумма еды и воды за 600 кадров.
- `production_emits_one_event_per_source_per_frame`.
- `stinky_and_repeller_match_love`.

### Task 10: Живность

**Files:** `src/life/{mod,hedgehog,squirrel,crow,boar,worm,leaves,predator,drops}.rs`, `tests/life.rs`.

**Interfaces:**
- `LifeRt { animals: Vec<Animal> }`. `Animal` хранит стабильный `id: u64` и
  `kind: AnimalKind`-enum с данными состояния.
- Логика, читающая вид камеры, использует только `input.view` (игровой вид 1×), никогда зум.
- Таймеры из `state.timers`.
- Еда от листьев и прочих подборов эмитит `Resource`.

**Тесты:**
- `timers_tick_like_love`: по фикстуре, где до появления животных далеко, значения
  `state.timers` через 3600 кадров равны LÖVE. Сами случайные броски между версиями не
  сравниваются. Для остального сравниваются детерминированные функции поведения: кадры анимации,
  пороги, зоны.
- `worm_keep_zone_uses_game_view`: зона червя одинакова при зуме 0.3 и 1.
- `squirrel_view_uses_game_view`.
- Для каждой функции `*_frame`, `*_zone`, `*_can`, `*_check`: golden на сетке входов.

### Task 11: Прогресс: улучшения, дерево, задачи, цели, туториал

**Files:** `src/progress/{mod,upgrades,skilltree,tasks,goals,tutorial}.rs`, `tests/progress.rs`.

**Interfaces:**
- `upgrades::{upg_bonus(st, key: &str) -> f64, can_buy(st, id) -> Result<(), BuyReject>, buy(sim, id)}`.
- `Command::BuyUpgrade` обрабатывается в `progress::step`.
- `skilltree::layout() -> &'static [TreeNode]`: позиции и связи узлов дерева для UI (Task 19).
- `tasks::*`, `tutorial::event(sim, name: &str, x: Option<f64>, y: Option<f64>)`; подсказки идут
  через `SimEvent::TutorialTip`.
- Покупки в дереве улучшений всплывающих чисел в мире не дают: экран дерева показывает цену сам.
  Награды задач и целей в еде или воде эмитят `Resource` в центре игрового вида.

**Тесты:**
- `upg_bonus_matches_love` по всем ключам и уровням.
- `buy_costs_and_requirements_match_love`.
- `tasks_progress_matches_love`: последовательность событий → состояние задач.
- `tutorial_stages_match_love`.

### Task 12: Погода и ветер

**Files:** `src/weather.rs`, `tests/weather.rs`.

**Interfaces:**
- `WeatherRt` с фазой дождя и полосами.
- `weather::is_raining(..)`, `weather::wind(..)`.
- `rain_*`-функции, влияющие на воду, эмитят `Resource { Water, +n }` у источника.

**Тесты:**
- `rain_phase_matches_love` (детерминированная часть).
- `wind_matches_love`.

---

**Задачи 13–21 (представление) независимы друг от друга** после Task 4 и своих задач логики. Общий
шаблон каждой:

1. Порт функций задачи из `rust/portmap.tsv` с маркерами `// port:`.
   - Постоянные объекты мира — сущности, синхронизированные с `Sim` по стабильному id
     (spawn/update/despawn).
   - Тайлы и гифы — чанковые меши, перестраиваемые при изменении.
   - Временные эффекты — сущности с компонентом времени жизни.
2. Скриншот-сверка: одна сцена в LÖVE и в Rust. Обе из `MYCO_SEED=31607` и одного сценария
   `MYCO_INPUT`, кадр 600. Скриншоты кладутся в `build/compare/<task>/{love,rust}.png`. Сверка
   визуальная, в отчёте задачи — оба пути и перечень различий.
3. `cargo test -p myco_app` → PASS; `python3 -I tools/port_coverage.py --task T<N>` → 0.
4. Коммит `feat(rust-app): <тема>`.

### Task 13: Отрисовка мира

**Files:** `src/render/world.rs`.

**Содержание:** функции `world_draw*`, `world_paint*`, `food_glow*`: почва, слои, пещеры, камни, озёра,
вода, свечение еды, туман неразведанного. Чанковые меши с атласом страниц.

**Тест:** `chunk_mesh_vertex_count_matches_tiles`: число квадов = число непустых тайлов чанка.

### Task 14: Отрисовка мицелия

**Files:** `src/render/mycelium.rs`, `src/render/shaders/glow_blur.wgsl`.

**Содержание:**
- гифы как меши-ленты по точкам `Growth`;
- узлы;
- превью прицела;
- свечение (`sh_glow_blur` → WGSL `Material2d`).

Перестройка меша только для изменённых гиф.

**Тест:** `ribbon_mesh_follows_points`: для гифы из 3 точек вершины лежат на расстоянии
`width/2` от оси.

### Task 15: Грибы и живность на экране, шейдеры

**Files:** `src/render/{mushrooms,life}.rs`, `src/render/shaders/{stinky,blackmetal}.wgsl`.

**Содержание:**
- спрайты и кадры грибов и всех животных;
- облака отпугивателей;
- вонючка (`sh_stinky`);
- чёрно-белый пост-эффект (`sh_blackmetal`, `bw_shader`).

**Тест:** `animal_entities_follow_sim_ids`: животное с id 5 появилось → одна сущность; исчезло →
сущность удалена.

### Task 16: Поверхность и лес

**Files:** `src/render/forest.rs`.

**Содержание:** `obj_forest_controller`: генерация и отрисовка чанков леса, листья заднего плана,
`leaf_fg*`. Чанки леса из сейва (`forest.chunks`) грузятся точно. Новые генерируются через `sim.rng`,
как в LÖVE.

Генерация леса — логика, но живёт здесь, потому что в оригинале это отдельный контроллер
поверхности. Её состояние пишется в `GameState.forest` через функции этого модуля, вызываемые из
`sim_bridge`.

**Тест:** `saved_forest_chunk_restored_exactly`: фикстура → те же деревья, кусты, пни с теми же
`x`, `flip`, `part`, `sprite`.

### Task 17: HUD, всплывающие числа, подсказки ПКМ, режим постановки

**Files:** `src/ui/{hud,popups,cursor_tip,placement,tasks}.rs`.

**Содержание:**
- **HUD оригинала** (`obj_hud`, алиасы `hud_*`): ресурсы, длина, кнопки грибов, панель задач,
  всплывающие окна задач, полосы наведения. Стиль, спрайты и шрифты оригинала; масштаб под
  реальный экран; текст не мельче 14 логических px.
- **Всплывающие числа** (`popups.rs`). Из `SimEvent::Resource`:
  - иконка ресурса (спрайт оригинала) и число;
  - зелёное `+N`, красное `−N`;
  - подъём на 60 логических px за 1.0 с с затуханием альфы к 0;
  - события одного `source` в одном шаге суммируются;
  - источник вне камеры отрисовки — не показывать;
  - не больше 64 одновременно: новое при переполнении прибавляется к ближайшему живому числу того
    же ресурса.

  Числа живут в мировых координатах и масштабируются с зумом, но не мельче 12 px на экране.
- **Подсказка ПКМ** (`cursor_tip.rs`). Из `SimEvent::NodeRejected`: красный текст у курсора на
  1.2 с. Тексты через `loc`:

  | Причина | Ключ | Текст / условие |
  |---|---|---|
  | `NotEnoughFood` | `tip_node_need_food` | «Нужно N еды» |
  | `NotOnMycelium` | `tip_node_not_on_mycelium` | |
  | `InWater` | `hint_no_nodes_in_water` | уже есть в CSV |
  | `TooCloseToNode` | `tip_node_too_close` | |
  | `Blocked` | `tip_node_blocked` | |

  Новые ключи добавляются в `rust/crates/myco_app/locale_extra.csv` для 12 языков (английский
  запасной) и подмешиваются к `Locale`.
- **ПКМ** (`placement.rs`):
  - в режиме постановки гриба отменяет режим;
  - иначе, если курсор не над UI, шлёт `Command::QuickNode` в точке `screen_to_world`.
- Контекстное меню «ADD NODE» не делается.

**Тесты:**
- `popups_aggregate_per_source_per_step`;
- `popups_cap_64_merges_into_nearest`;
- `popups_skip_offscreen`;
- `popup_rises_and_fades`: на t=0.5 с смещение 30 px, альфа 0.5±0.05;
- `rmb_in_placement_cancels_not_places`;
- `rmb_over_hud_does_nothing`.

### Task 18: Меню, логотип, слоты, имя, настройки, титры, пауза

**Files:** `src/ui/{menu,slots,name,settings,credits,pause}.rs`, `src/logo.rs`.

**Содержание:**
- Главное меню оригинала: Exploration, Survival, Load, Settings, Credits, Exit, версия v1.02.
- Процедурный логотип (`scr_logo`, 70 функций).
- Выбор слота (`list_slots`): повреждённый слот показан с пометкой, клик по нему ничего не ломает.
- Ввод имени.
- Настройки: язык, звук, полный экран/окно.
- Титры.
- Пауза: продолжить, экран, выход в меню с сохранением.

Сохранение при выходе в меню и при закрытии окна (`obj_game` Other_3).

**Тесты:**
- `corrupt_slot_listed_not_loadable`;
- `logo_geometry_matches_love`: точки логотипа для текста «MYCELARIUM» и сида 1 совпадают с golden;
- `quit_writes_slot`.

### Task 19: Экран дерева улучшений

**Files:** `src/ui/tree.rs`.

**Содержание:** функции `tree_*`: отрисовка дерева, гифы-связи, иконки, состояния (куплено, доступно,
закрыто), покупка через `Command::BuyUpgrade`, отказ с подсказкой.

**Тест:** `tree_node_positions_match_love`: позиции узлов дерева в GUI-координатах равны golden.

### Task 20: Звук

**Files:** `src/audio.rs`.

**Содержание:** функции `scr_audio`: sfx с позиционированием относительно камеры, петли, эмбиент,
дождь, лес, звук роста гиф, музыка с очередью, громкости из настроек. `SimEvent::Sound(SoundCue)`
→ проигрывание. Усиление ассета применяется один раз: известный баг LÖVE-порта не повторять.

**Тест:** `sound_volume_falls_off_like_love`: функция громкости по расстоянию и `sound_camera`
равна golden на сетке.

### Task 21: Камера, зум, интро и кат-сцены

**Files:** `src/camera.rs` (расширить), `src/cine.rs`.

**Содержание:**
- `camera_jump*`, `camera_fly`, `cam_box`, интро (ёж роняет гриб), `cine_*` (планета, споры),
  `intro*`, маркеры животных.
- **Зум:**
  - колесо и `PinchGesture`, к точке под курсором;
  - плавность: экспоненциальное сглаживание с постоянной 0.12 с;
  - пределы: максимум приближения 2×;
  - минимум — масштаб, при котором в экран влезает bbox разведанного: гифы, узлы, грибы плюс
    отступ 256 px; но не меньше `min(1, ...)`, то есть 1× доступен всегда (Review Focus 4);
  - камера отрисовки не уходит выше неба (`world.surface_y − view_h`) и ниже `world.bottom_y`.
- Во время интро и кат-сцен зум заблокирован на 1×.

**Тесты:**
- `zoom_limits_small_colony_allows_1x`;
- `zoom_keeps_point_under_cursor`: после зума мировая точка под курсором та же, ±0.5 px;
- `zoom_clamped_to_world_bounds`;
- `zoom_locked_during_cine`.

---

### Task 22: Сборка игры целиком и сверка с LÖVE

**Files:**
- Modify: `src/main.rs`, `src/sim_bridge.rs` (состояния, связь меню ↔ игра).
- Create: `rust/conformance/run.sh`, `rust/conformance/scenarios/*.txt`, `tools/conformance.py`.

**Interfaces:**
- Consumes: всё выше.
- Produces: `tools/conformance.py <scenario>`:
  1. запускает LÖVE и Rust с `MYCO_SEED`, тем же `MYCO_INPUT` и `MYCO_DUMP` на кадрах из сценария;
  2. сравнивает дампы по детерминированным полям: `state.food`, `water_free`, `length_px`,
     `mycelium.growths/knots` (позиции ±0.5 px), `mushrooms`, `world.chunks`, `upgrade_levels`,
     `tasks`;
  3. печатает расхождения; выходит с кодом 1, если они есть.

**Сценарии:**
1. `explore_basic`: новая игра Exploration, интро, 3 выстрела, ПКМ-узел, сохранение.
2. `survival_basic`: то же в Survival.
3. `mushroom_economy`: фикстура с открытыми грибами, постановка двух грибов, 1800 кадров.
4. `upgrade_buy`: покупка двух улучшений.
5. `cross_save`:
   - сейв Rust → LÖVE загружает → `MYCO_DUMP` равен;
   - сейв LÖVE → Rust загружает → равен.

- [ ] **Step 1:** Написать сценарии и `tools/conformance.py` с тестом
  `tools/tests/test_conformance.py::test_diff_reports_field_path`.
- [ ] **Step 2:** Связать состояния: меню → слот → имя → интро → игра → пауза → меню.
- [ ] **Step 3:** Прогнать все 5 сценариев. Каждое расхождение — баг одной из задач 5–21.
  Чинить с тестом в соответствующем модуле. Если виноват Lua, чинить и Lua.
- [ ] **Step 4:** `python3 -I tools/port_coverage.py --all` → 0.
  `cargo test --workspace` → PASS. `luajit tests/run.lua` → PASS.
- [ ] **Step 5:** Ручной прогон автопилотом 20 000 кадров в каждом режиме: без паник, FPS ≥ 60 на
  3024×1898.
- [ ] **Step 6:** Коммит `feat(rust): full game wired, conformance with LÖVE`.

### Task 23: Упаковка

**Files:** `tools/package_rust.sh`.

- [ ] **Step 1:** `tools/package_rust.sh`:
  1. `cargo build --release -p myco_app`;
  2. собрать `dist/Mycelarium Rust.app`: бинарь, `Contents/Resources/assets` (копия `build/assets`),
     Info.plist через `plutil`, иконка как в `tools/package.sh`;
  3. ad-hoc `codesign`.
- [ ] **Step 2:** Проверка: `open "dist/Mycelarium Rust.app"`. Игра стартует в полный экран ниже
  чёлки и видит сейвы LÖVE-порта в меню загрузки (`MYCO_SHOT` на кадре 120 показывает слоты).
- [ ] **Step 3:** Коммит `chore(rust): package Mycelarium Rust.app`.

## Порядок и параллельность

1. **Последовательно:** Task 1 → 2 → 3 → 4.
2. **Волна логики,** параллельно: задачи 5, 6, 7, 8, 9, 10, 11, 12.
   - Задача 8 зависит от `WorldRt::water_shown_at` и `is_solid` из Task 5. До готовности Task 5 она
     пишет тесты и логику против трейта `WorldQuery`, который Task 5 реализует для `WorldRt`.
   - Так же 9 и 10 используют `WorldQuery`. Трейт определяет Task 3 в `world/mod.rs`:
     `tile_at`, `is_solid`, `water_shown_at`, `surface_y`, `soil_at`.
3. **Волна представления,** параллельно: задачи 13–21, каждая после своих задач логики.
4. **Последовательно:** Task 22 → 23.
