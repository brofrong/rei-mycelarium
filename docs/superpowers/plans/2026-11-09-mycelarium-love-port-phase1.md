# Mycelarium LÖVE Port — Phase 1 Implementation Plan (фундамент + главное меню)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Собрать фундамент порта (извлечение ресурсов, символизированная декомпиляция YYC, рантайм-совместимость GameMaker на LÖVE, корректный экран и мышь, сборка `.app`). Завершить фазу работающим на Mac главным меню Mycelarium.

**Architecture:** Python-инструменты разбирают `data.win` в `build/assets/` (PNG-страницы + JSON-метаданные). Скрипты Ghidra headless подписывают функции YYC и выгружают декомпиляцию в `re/decomp/`. Игра на LÖVE: `game/gm/` — слой совместимости с GML, `game/myco/` — переведённый код игры с оригинальными именами. Код игры вызывает GML-подобный API, а не LÖVE напрямую.

**Tech Stack:** Python 3.15 (venv: pytest, Pillow), Ghidra 12.2 (brew, openjdk@25, Java GhidraScript), LÖVE 11.5 (brew cask `love`), LuaJIT (brew, для unit-тестов Lua), bash.

**Spec:** `docs/superpowers/specs/2026-11-09-mycelarium-love-port-design.md`

**Фазы проекта.** Это план фазы 1. Фаза 2 (мир + мицелий) и фаза 3 (остальные системы) пишутся
отдельными планами в задаче 11, когда появится инвентарь функций из декомпиляции. Без него
планировать перевод конкретных функций нельзя.

## Global Constraints

- Платформа: macOS, Apple Silicon (arm64). Windows, Linux и Intel не поддерживаются.
- Движок: LÖVE 11.5.
- Источник: только купленная копия в `~/GameHub/steamapps/common/Mycelarium/` (переопределяется переменной `MYCO_GAME_DIR`).
- Не коммитить: `Mycelarium.exe`, `data.win`, любые извлечённые ресурсы (`build/`), проект Ghidra (`re/ghidra/`), собранный `.app` (`dist/`). Декомпиляцию (`re/decomp/`) и заметки коммитить можно, репозиторий приватный.
- Python запускать с `-I` при чтении файлов игры. Скрипты лежат в репозитории, а не в папке игры.
- Базовое разрешение игры 1920×1080. Внутреннее имя `MYCO`.
- Переведённый код сохраняет оригинальные имена: функция `gml_Script_mycelium_add_length` → Lua `mycelium_add_length`; событие `gml_Object_obj_game_Step_0` → `obj_game.Step_0`.
- Массивы GML индексируются с 0. Это обеспечивает тип `gm.array`, в коде игры `+1` не пишется.
- Steamworks: только заглушки, без сети.
- Порядок событий кадра как в GameMaker: Begin Step → Step → End Step → Pre Draw → Draw Begin → Draw → Draw End → Post Draw → Draw GUI Begin → Draw GUI → Draw GUI End.
- Коммиты на ветке `love-port`, сообщения в стиле `feat:`/`fix:`/`docs:`/`chore:`, окончание `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. Фуллскрин на MacBook с вырезом (Retina, DPI 2): картинка в безопасной области, клик по краю кнопки попадает в кнопку. Тест в задаче 6: `screen_to_game` с ненулевым смещением safe area и dpi 2.
2. Окно, изменённое до произвольного соотношения сторон или свёрнутое (0×0): нет деления на ноль и NaN, мышь вне картинки даёт координаты за пределами 0..1920/0..1080, а не мусор. Тест в задаче 6.
3. Многократное переключение окно ↔ фуллскрин: настройки сохраняются, canvas пересоздаётся без утечки, мышь продолжает попадать. Тест в задаче 6 (сохранение и чтение настроек), ручная проверка в задаче 10.
4. Игра не найдена или изменился формат `data.win`: извлекатель падает с понятным сообщением, а не с трейсбеком из середины парсинга. Тест в задаче 1.
5. Нелатинский текст (русский, CJK, корейский): GML-строковые функции считают символы, а не байты. Тест в задаче 7: `string_length("Мицелий") == 7`, `string_char_at("Мицелий", 1) == "М"`.

---

## File Structure

```
.gitignore                      build/ dist/ re/ghidra/ .venv/ game/assets
tools/requirements.txt          pytest, Pillow
tools/myco_extract/__init__.py
tools/myco_extract/datawin.py   контейнер FORM, чанки, строки, списки указателей
tools/myco_extract/qoi.py       BZ2+QOI GameMaker → RGBA
tools/myco_extract/resources.py TPAG, SPRT, FONT, SOND/AUDO, SHDR, ROOM, OBJT → dict
tools/myco_extract/extract.py   CLI: python3 -I -m myco_extract.extract → build/assets/
tools/tests/test_*.py           pytest
tools/ghidra/NameYYC.java       подписывает gml_* функции, F_<builtin>, слоты переменных
tools/ghidra/ExportDecomp.java  выгружает декомпиляцию gml_* в re/decomp/
tools/ghidra/run.sh             headless-запуск
tools/inventory.py              re/decomp → re/inventory.md, re/builtins.json
tools/package.sh                build/assets + game/ → dist/Mycelarium.app
re/decomp/{objects,scripts}/*.c выгрузка (коммитится)
re/inventory.md, re/builtins.json
re/reference/*.png              скриншоты оригинала (коммитятся)
game/conf.lua, game/main.lua
game/gm/init.lua                собирает модули в таблицу gm
game/gm/display_math.lua        чистая математика вписывания/мыши (без love)
game/gm/display.lua             окно, фуллскрин, application surface
game/gm/value.lua               array, struct, method, typeof, string-функции UTF-8
game/gm/instance.lua            объекты, экземпляры, with/other, события, комнаты
game/gm/assets.lua              загрузка sprites/fonts/sounds/rooms из assets/
game/gm/draw.lua                draw_*, цвета, alpha, blend, шрифты, текст
game/gm/input.lua               mouse_check_button*, keyboard_*, device_mouse_*_to_gui
game/gm/audio.lua               audio_play_sound и т.д. (по инвентарю)
game/gm/file.lua                ini_*, json_*, file_text_*, сохранения в love.filesystem
game/gm/steam.lua               заглушки steam_*
game/myco/objects/*.lua         переведённые объекты
game/myco/scripts/*.lua         переведённые скрипты, по файлу на scr_*
tests/run.lua                   мини-раннер для luajit
tests/test_*.lua
```

---

### Task 1: Каркас репозитория и парсер контейнера `data.win`

**Files:**
- Create: `.gitignore`, `tools/requirements.txt`, `tools/myco_extract/__init__.py`, `tools/myco_extract/datawin.py`
- Test: `tools/tests/test_datawin.py`, `tools/tests/conftest.py`

**Interfaces:**
- Produces:
  - `DataWin.open(path: str) -> DataWin`, где `path` — файл `data.win`. Ошибки: `DataWinError("not a GameMaker data.win: <path>")`, если нет `FORM`; `DataWinError("game not found: <path>")`, если файла нет.
  - `DataWin.chunks: dict[str, tuple[int, int]]` — имя → (смещение данных, размер).
  - `DataWin.u8/u16/u32/i32/f32(off) -> int|float`, `DataWin.string(ptr) -> str | None` (GM-строка: длина u32 по `ptr-4`, UTF-8; `ptr == 0` → `None`).
  - `DataWin.pointer_list(off) -> list[int]` (u32 count + count×u32).
  - `DataWin.chunk_list(name) -> list[int]` = `pointer_list(chunks[name][0])`.
  - `DataWin.gen8() -> dict` с ключами `name`, `filename`, `window_w`, `window_h`, `bytecode`.
  - `game_dir() -> pathlib.Path`: `$MYCO_GAME_DIR` или `~/GameHub/steamapps/common/Mycelarium`.
  - фикстура pytest `datawin`: открывает реальный `data.win`, `pytest.skip`, если игры нет.

- [ ] **Step 1:** Создать `.venv` (`python3 -m venv .venv && .venv/bin/pip install -r tools/requirements.txt`) и `.gitignore` со строками из File Structure.
- [ ] **Step 2: Write the failing tests**

```python
def test_gen8_real(datawin):
    g = datawin.gen8()
    assert (g["name"], g["filename"], g["window_w"], g["window_h"], g["bytecode"]) == ("MYCO", "MYCO", 1920, 1080, 17)

def test_chunks_real(datawin):
    for name in ["GEN8", "SPRT", "ROOM", "OBJT", "TPAG", "TXTR", "AUDO", "SHDR", "FONT", "STRG"]:
        assert name in datawin.chunks
    assert "CODE" not in datawin.chunks          # YYC

def test_chunk_counts_real(datawin):
    counts = {n: len(datawin.chunk_list(n)) for n in ["OBJT", "ROOM", "SPRT", "SOND", "SHDR", "FONT", "TXTR"]}
    assert counts == {"OBJT": 10, "ROOM": 3, "SPRT": 167, "SOND": 24, "SHDR": 3, "FONT": 4, "TXTR": 5}

def test_not_datawin(tmp_path):
    p = tmp_path / "data.win"; p.write_bytes(b"NOPE" + b"\0" * 8)
    with pytest.raises(DataWinError, match="not a GameMaker data.win"):
        DataWin.open(str(p))

def test_missing(tmp_path):
    with pytest.raises(DataWinError, match="game not found"):
        DataWin.open(str(tmp_path / "nope.win"))
```

- [ ] **Step 3:** `cd tools && ../.venv/bin/python -I -m pytest tests/test_datawin.py -v`. Ожидается FAIL (ImportError).
- [ ] **Step 4:** Реализовать `datawin.py`. Раскладка GEN8 (смещения от начала данных чанка): `+0` debug u8, `+1` bytecode u8, `+4` filename ptr, `+0x28` name ptr, `+0x3C` window_w u32, `+0x40` window_h u32. Чтение через `memoryview`, весь файл в памяти.
- [ ] **Step 5:** Повторить запуск из шага 3. Ожидается PASS (5 passed).
- [ ] **Step 6:** Коммит `chore: scaffold tools and data.win container parser`.

---

### Task 2: Декодер текстур BZ2+QOI и выгрузка страниц

**Files:**
- Create: `tools/myco_extract/qoi.py`
- Test: `tools/tests/test_qoi.py`

**Interfaces:**
- Consumes: `DataWin` (задача 1).
- Produces:
  - `decode_gm_qoi(buf: bytes) -> tuple[int, int, bytes]` — поток `fioq` → (w, h, RGBA).
  - `decode_texture(dw: DataWin, txtr_entry_ptr: int) -> tuple[int, int, bytes]` — поддерживает `2zoq` (BZ2+QOI), `fioq` (QOI) и `\x89PNG` (через Pillow).
  - `texture_pages(dw) -> list[dict]` — для каждой записи TXTR `{"index", "w", "h", "data_ptr"}`. Запись TXTR (8×u32): `[scaled, generated_mips, data_len, w, h, index_in_group, data_ptr, unk]`.

- [ ] **Step 1: Write the failing tests**

```python
def _fioq(w, h, ops: bytes) -> bytes:
    return b"fioq" + struct.pack("<HHI", w, h, len(ops)) + ops

def test_color_and_run():
    # COLOR(r,g,b,a=все) затем RUN_8 на 2 повтора → 3 одинаковых пикселя
    w, h, px = decode_gm_qoi(_fioq(3, 1, bytes([0xF0 | 0xF, 10, 20, 30, 40, 0x40 | 1])))
    assert (w, h) == (3, 1) and px == bytes([10, 20, 30, 40]) * 3

def test_diff8_and_index():
    # старт (0,0,0,255) → индекс 63; DIFF_8 r-1,g+1,b+1 → (255,1,1,255) → индекс 0; INDEX 63 → снова старт
    start_hash = (0 ^ 0 ^ 0 ^ 255) & 63
    ops = bytes([0xF0 | 0x1, 255, 0x80 | (3 << 4) | (1 << 2) | 1, start_hash])
    _, _, px = decode_gm_qoi(_fioq(3, 1, ops))
    assert px == bytes([0, 0, 0, 255, 255, 1, 1, 255, 0, 0, 0, 255])

def test_real_page_dims(datawin):
    pages = texture_pages(datawin)
    assert [(p["w"], p["h"]) for p in pages] == [(64, 64), (2048, 2048), (2048, 2048), (2048, 2048), (2048, 1024)]
    w, h, px = decode_texture(datawin, datawin.chunk_list("TXTR")[4])
    assert (w, h, len(px)) == (2048, 1024, 2048 * 1024 * 4)
```

- [ ] **Step 2:** Запустить `test_qoi.py`. Ожидается FAIL.
- [ ] **Step 3:** Реализовать. Заголовок `2zoq`: `magic(4) u16 w u16 h u32 uncompressed_len`, затем bz2-поток (`bz2.BZ2Decompressor`, хвост после потока игнорировать). Заголовок `fioq`: `magic(4) u16 w u16 h u32 ops_len`. Алгоритм QOI GameMaker (старый черновик QOI, проверен на странице 4 этой игры). Начальный пиксель (0,0,0,255), индекс из 64 нулевых записей, `s(v,n)` — знаковое n-битное:

```
if run > 0: run -= 1
else: b1 = next
  b1&0xC0==0x00: px = index[b1&63]
  b1&0xE0==0x40: run = b1&31
  b1&0xE0==0x60: run = ((b1&31)<<8 | next) + 32
  b1&0xC0==0x80: r+=s(b1>>4&3,2); g+=s(b1>>2&3,2); b+=s(b1&3,2)
  b1&0xE0==0xC0: m=b1<<8|next;  r+=s(m>>8&31,5); g+=s(m>>4&15,4); b+=s(m&15,4)
  b1&0xF0==0xE0: m=b1<<16|next<<8|next; r+=s(m>>15&31,5); g+=s(m>>10&31,5); b+=s(m>>5&31,5); a+=s(m&31,5)
  else (0xF0): флаги 8,4,2,1 → читать r,g,b,a
  все компоненты & 255; index[(r^g^b^a)&63] = px
emit px
```

- [ ] **Step 4:** Повторить запуск из шага 2. Ожидается PASS.
- [ ] **Step 5:** Коммит `feat: decode GameMaker BZ2/QOI texture pages`.

---

### Task 3: Извлечение ресурсов в `build/assets/`

**Files:**
- Create: `tools/myco_extract/resources.py`, `tools/myco_extract/extract.py`
- Test: `tools/tests/test_resources.py`

**Interfaces:**
- Consumes: `DataWin`, `decode_texture`, `texture_pages`.
- Produces (это контракт для `game/gm/assets.lua`, задача 8):
  - `build/assets/pages/page<N>.png`
  - `build/assets/sprites.json`: `{name: {"w","h","origin":[x,y],"bbox":[l,t,r,b],"speed","speed_type","frames":[{"page","src":[x,y,w,h],"dst":[x,y],"bound":[w,h]}]}}`. TPAG: `src x,y,w,h u16`, `target x,y,w,h u16`, `bound w,h u16`, `page u16`.
  - `build/assets/fonts.json` + страницы: `{name: {"size","bold","italic","first","last","line_height"?, "glyphs": {"<codepoint>": {"x","y","w","h","shift","offset"}}, "frame": <tpag как в sprites>}}`
  - `build/assets/sounds/<name>.ogg|.wav` + `sounds.json`: `{name: {"file","volume","pitch","group","streamed": bool}}` (`hang_guitar_01` — внешний файл из папки игры, копируется).
  - `build/assets/shaders/<name>.vsh|.fsh` (GLSL ES-версии из SHDR).
  - `build/assets/rooms.json`: `{name: {"w","h","speed","color","views":[...], "layers":[{"name","depth","type","instances"|"background"|...}], "instances":[{"id","object","x","y","scale_x","scale_y","angle","color","creation_code"}]}}`
  - `build/assets/objects.json`: `{name: {"sprite","visible","solid","persistent","depth","parent","events":[[type,subtype],...]}}`; `type` — числовой тип события GM (Create 0, Step 3, Other 7, Draw 8, CleanUp 12), как в суффиксах `gml_Object_<obj>_<Event>_<subtype>`.
  - Копии внешних файлов: `localization.csv`, `basic_items.json`, `fonts/*`, `splash.png`.
  - Симлинк `game/assets -> ../build/assets`.
  - CLI: `.venv/bin/python -I -m myco_extract.extract [--game-dir DIR] [--out build/assets]`. При отсутствии игры выход с кодом 2 и сообщением `game not found: <path> (set MYCO_GAME_DIR)`.

Раскладки SPRT/FONT/ROOM/OBJT/SHDR для GMS2 2024+ сверять по исходникам UndertaleModTool (`UndertaleModLib/Models/Undertale{Sprite,Font,Room,GameObject,Shader}.cs`). Если версия формата новее известной, правильность подтверждается тестами на реальных числах ниже.

- [ ] **Step 1: Write the failing tests**

```python
def test_rooms(assets):        # assets: фикстура, однократно запускает extract во временный каталог
    r = assets["rooms"]
    assert {k: (v["w"], v["h"]) for k, v in r.items()} == {"Room1": (35040, 3744), "Main_menu": (1920, 1080), "UI": (1920, 1080)}
    assert any(i["object"] == "obj_game" for room in r.values() for i in room["instances"])

def test_objects(assets):
    assert sorted(assets["objects"]) == ["obj_forest_controller", "obj_game", "obj_gm_button", "obj_gm_button1", "obj_gm_text",
                                         "obj_gm_text1", "obj_gm_textbox", "obj_hud", "obj_life", "obj_mycelium_controller"]
    assert [3, 1] in assets["objects"]["obj_hud"]["events"]      # Step(3), Begin(1) → gml_Object_obj_hud_Step_1

def test_sprites(assets):
    s = assets["sprites"]
    assert len(s) == 167 and "apple" in s and "background_B_layer1" in s
    for spr in s.values():
        for fr in spr["frames"]:
            assert 0 <= fr["page"] < 5 and fr["src"][2] > 0

def test_sounds_shaders_fonts(assets, out_dir):
    assert len(assets["sounds"]) == 24
    assert all((out_dir / "sounds" / v["file"]).exists() for v in assets["sounds"].values())
    assert sorted(p.stem for p in (out_dir / "shaders").glob("*.fsh")) == ["sh_blackmetal", "sh_glow_blur", "sh_stinky"]
    assert sorted(assets["fonts"]) == ["Font1", "fnt_gm_15", "fnt_gm_20", "fnt_gm_30"]

def test_cli_missing_game(tmp_path):
    r = subprocess.run([sys.executable, "-I", "-m", "myco_extract.extract", "--game-dir", str(tmp_path)], capture_output=True, text=True, cwd=TOOLS)
    assert r.returncode == 2 and "game not found" in r.stderr
```

- [ ] **Step 2:** Запустить `test_resources.py`. Ожидается FAIL.
- [ ] **Step 3:** Реализовать `resources.py` (по функции на тип ресурса, возвращают dict) и `extract.py` (пишет файлы, создаёт симлинк).
- [ ] **Step 4:** Повторить запуск из шага 2. Ожидается PASS. Затем запустить CLI на реальной игре и открыть `build/assets/pages/page1.png`: глазами должна быть видна осмысленная графика.
- [ ] **Step 5:** Коммит `feat: extract sprites, fonts, sounds, shaders, rooms, objects`.

---

### Task 4: Символизация YYC и выгрузка декомпиляции (Ghidra)

**Files:**
- Create: `tools/ghidra/NameYYC.java`, `tools/ghidra/ExportDecomp.java`, `tools/ghidra/run.sh`
- Output (коммит): `re/decomp/objects/*.c`, `re/decomp/scripts/*.c`, `re/decomp/index.tsv`

**Interfaces:**
- Produces:
  - Функции в проекте Ghidra `re/ghidra/Myco` названы так: `gml_*` (символ `@` заменён на `__`), `F_<builtin>` для встроенных функций рантайма; слоты id переменных — метки `vid_<name>`.
  - `re/decomp/index.tsv`: `name\taddress\tsize\tstatus` (`ok` | `timeout` | `error`).
  - Файл на функцию: `re/decomp/objects/<gml_Object_...>.c`, `re/decomp/scripts/<остальные>.c`.

Факты из разведки, на которые опираются скрипты:
- Каждая GML-функция в прологе делает `LEA reg, "<её gml-имя>"` для кадра стек-трейса (например, `gml_Object_obj_game_Create_0` → функция с LEA по адресу `0x1405f881e`, строка `0x140db0250`). Правило: строка `gml_*` с xref из кода в пределах 0x100 байт от начала содержащей функции → переименовать эту функцию.
- Встроенные функции регистрируются как `lea rdx, <F_func>; mov r8d, <argc>; lea rcx, "<name>"; call <Function_Add>` (пример: `"draw_sprite_ext"` по `0x140cb6388`, LEA в `0x14074de6c`). Цель вызова, следующего за этим LEA, — это `Function_Add`. Для всех его вызовов взять RCX (строку) и RDX (функцию) из предшествующих 6 инструкций и назвать функцию `F_<name>`.
- Имена пользовательских переменных лежат в `.data` таблицей записей по 16 байт `{char* name; int32 id(=-1); pad}` (пример: `"chunk_w"` → запись в районе `0x140ff8850`). Пройти таблицу в обе стороны, пока поле +8 равно `0xffffffff`, и поставить метку `vid_<name>` на поле id.
- Если в выгрузке `gml_Object_obj_game_Create_0` встроенные переменные (x, y, image_index…) остаются безымянными вызовами, найти функцию регистрации встроенных переменных (паттерн аналогичен `Function_Add`, строки вроде `"image_index"`) и добавить третий проход.

- [ ] **Step 1:** `brew install ghidra luajit && brew install --cask love`. Проверить: `analyzeHeadless` в PATH (или `$(brew --prefix ghidra)/libexec/support/analyzeHeadless`), `love --version` → 11.5.
- [ ] **Step 2:** Написать `run.sh`: `analyzeHeadless re/ghidra Myco -import "$MYCO_GAME_DIR/Mycelarium.exe" -overwrite -scriptPath tools/ghidra -postScript NameYYC.java -postScript ExportDecomp.java re/decomp -max-cpu 8`. Повторный запуск без `-import`: `-process Mycelarium.exe -noanalysis`. Анализ долгий, запускать в фоне.
- [ ] **Step 3:** Реализовать `NameYYC.java` (три прохода выше; в конце печатает `named gml=<N> builtins=<M> vars=<K>`) и `ExportDecomp.java` (`DecompInterface`, таймаут 120 с на функцию).
- [ ] **Step 4:** Запустить. Ожидается: `gml >= 700`, `builtins >= 1000`, `vars >= 300`; в `index.tsv` не меньше 95% строк со статусом `ok`.
- [ ] **Step 5:** Открыть `re/decomp/objects/gml_Object_obj_game_Create_0.c`: в нём должны быть видны вызовы `F_*` и обращения к `vid_*`. Если нет — исправить проходы и повторить шаг 4.
- [ ] **Step 6:** Коммит `feat: symbolize YYC functions and export decompilation`.

---

### Task 5: Инвентарь

**Files:**
- Create: `tools/inventory.py`, `tools/tests/test_inventory.py`
- Output (коммит): `re/inventory.md`, `re/builtins.json`

**Interfaces:**
- Consumes: `re/decomp/**`, `re/decomp/index.tsv`.
- Produces:
  - `re/builtins.json`: `{"<builtin>": {"calls": int, "functions": [gml-имена]}}` — все `F_*`, вызываемые из `gml_*`, без префикса `F_`.
  - `re/inventory.md`: таблица функций по группам (объект/событие; скрипт → файл `scr_*`, определяемый по суффиксу `@scr_*`, ссылке из `gml_GlobalScript_scr_*` или соседству адресов), граф «кто кого вызывает» для каждого события объекта, список встроенных функций по убыванию частоты.
  - `callees(c_source: str) -> set[str]` и `builtins_used(c_source: str) -> set[str]` в `inventory.py`.

- [ ] **Step 1: Write the failing tests**

```python
SRC = "void gml_Script_x(void){ F_draw_sprite_ext(a,b); gml_Script_mycelium_add_length(c); F_draw_sprite_ext(d); }"
def test_builtins_used(): assert builtins_used(SRC) == {"draw_sprite_ext"}
def test_callees():       assert callees(SRC) == {"gml_Script_mycelium_add_length"}
```

- [ ] **Step 2:** Запустить. Ожидается FAIL. **Step 3:** Реализовать (регулярки по идентификаторам). **Step 4:** PASS. Сгенерировать файлы и проверить, что в `builtins.json` есть `draw_sprite_ext` и `surface_create`.
- [ ] **Step 5:** Коммит `feat: function and builtin inventory`.

---

### Task 6: Скелет LÖVE, экран и мышь

**Files:**
- Create: `game/conf.lua`, `game/main.lua`, `game/gm/init.lua`, `game/gm/display_math.lua`, `game/gm/display.lua`, `tests/run.lua`, `tests/test_display.lua`

**Interfaces:**
- Produces:
  - `display_math.fit(win_w, win_h, safe_x, safe_y, safe_w, safe_h, base_w, base_h) -> {x, y, w, h, scale}` — прямоугольник вывода application surface в пикселях окна, внутри safe area, с сохранением пропорций (letterbox). При нулевых размерах `scale = 0`, а не NaN.
  - `display_math.screen_to_game(fit, px, py) -> gx, gy` — обратное преобразование. При `scale == 0` возвращает `-1, -1`.
  - `display.init(settings)`, `display.set_fullscreen(bool)`, `display.begin_frame()` / `display.end_frame()` (рисование в canvas 1920×1080 и вывод по `fit`), `display.mouse_game() -> gx, gy`, `display.mouse_gui() -> gx, gy`.
  - `display.load_settings() -> {fullscreen: bool, window_w, window_h}` и `display.save_settings(t)` в `love.filesystem` (`settings.ini`, секция `[display]`, те же ключи, что у оригинала).
  - `tests/run.lua`: `luajit tests/run.lua` находит `tests/test_*.lua`, каждый возвращает таблицу `name → function`; печатает `N passed, M failed`, при ошибках выходит с кодом 1.
  - Отладка: `MYCO_SHOT=<path>:<frame>` — сохранить скриншот на кадре N и выйти.
- `conf.lua`: `t.identity = "Mycelarium"`, `t.window.highdpi = true`, `t.window.usedpiscale = false`, `t.window.title = "Mycelarium"`, resizable окно.

Политику размеров окна и фуллскрина (как оригинал выбирает размер, что делает с соотношением сторон, отличным от 16:9) взять из декомпиляции функций, читающих `display_sizes` / `display_mode` / `window_size` (найти через `re/inventory.md`). Если оригинал не letterbox'ит, изменить `fit` под его логику и поправить тесты; letterbox остаётся решением по умолчанию.

- [ ] **Step 1: Write the failing tests**

```lua
local dm = require("game.gm.display_math")
return {
  exact_16_9 = function()
    local f = dm.fit(3840, 2160, 0, 0, 3840, 2160, 1920, 1080)
    assert(f.scale == 2 and f.x == 0 and f.y == 0)
  end,
  notch_retina = function()            -- 3024×1964 пикс, safe area ниже выреза (74 пикс при dpi 2)
    local f = dm.fit(3024, 1964, 0, 74, 3024, 1890, 1920, 1080)
    assert(f.y >= 74 and f.y + f.h <= 1964 and math.abs(f.w / f.h - 16/9) < 1e-3)
    local gx, gy = dm.screen_to_game(f, f.x + f.w - 1, f.y)   -- правый верхний угол картинки
    assert(gx > 1918 and gx < 1920 and gy >= 0 and gy < 1)
  end,
  outside_picture = function()
    local f = dm.fit(1000, 1000, 0, 0, 1000, 1000, 1920, 1080)
    local _, gy = dm.screen_to_game(f, 500, 5)                 -- в верхней чёрной полосе
    assert(gy < 0)
  end,
  minimized = function()
    local f = dm.fit(0, 0, 0, 0, 0, 0, 1920, 1080)
    assert(f.scale == 0); local gx, gy = dm.screen_to_game(f, 10, 10); assert(gx == -1 and gy == -1)
  end,
}
```

и `test_settings_roundtrip` в отдельном файле: сериализация и разбор `settings.ini` (чистые функции `display_math.settings_to_ini(t) / ini_to_settings(s)`), включая `fullscreen="0.000000"` из оригинального формата.

- [ ] **Step 2:** `luajit tests/run.lua`. Ожидается FAIL.
- [ ] **Step 3:** Реализовать `display_math.lua`, `display.lua`, `conf.lua`, `main.lua` (тестовая сцена: canvas 1920×1080 с рамкой и крестиком в позиции `mouse_game()`).
- [ ] **Step 4:** `luajit tests/run.lua`. Ожидается PASS.
- [ ] **Step 5:** Ручная проверка на MacBook: `love game` в окне и в фуллскрине (F11 / Cmd+Ctrl+F), крестик точно под курсором, рамка целиком видна и не уходит под вырез. Записать фактические значения `love.window.getSafeArea()` и `love.graphics.getDimensions()` в фуллскрине в `re/notes/display.md`.
- [ ] **Step 6:** Коммит `feat: LÖVE skeleton with letterboxed app surface and exact mouse mapping`.

---

### Task 7: Ядро совместимости GML: значения, экземпляры, события, комнаты

**Files:**
- Create: `game/gm/value.lua`, `game/gm/instance.lua`, `game/gm/steam.lua`
- Test: `tests/test_value.lua`, `tests/test_instance.lua`

**Interfaces:**
- Produces (`gm.*`, экспортируется через `game/gm/init.lua`, без зависимости от `love` для тестируемых частей):
  - `gm.array(...)`, `gm.array_create(n, v)`, `a[i]` с 0-индексацией (метатаблица), `array_length`, `array_push`, `array_delete`, `array_insert`, `array_sort` и остальные `array_*` из `re/builtins.json`.
  - `gm.struct(t)`, методы: `gm.method(self, fn)` — вызов `m(args)` подставляет связанный `self`. `typeof`, `is_struct`, `is_array`, `is_undefined`, `undefined` (уникальное значение, а не `nil`, чтобы `struct.x = undefined` хранился).
  - Строки (UTF-8, индексы с 1, как в GML): `string_length`, `string_char_at`, `string_copy`, `string_pos`, `string_replace_all`, `string_upper`, `string`, `real`, `string_format`, и т.д. по инвентарю.
  - `gm.object(name, def)` — регистрация объекта: `def.parent`, события как поля `Create_0`, `Step_0`, `Step_1`, `Draw_0`, `Draw_64`, `Draw_73`, `Draw_74`, `Draw_75`, `Draw_77`, `CleanUp_0`, `Other_3`.
  - `instance_create_depth(x, y, depth, obj)`, `instance_create_layer`, `instance_destroy`, `instance_exists`, `instance_find`, `instance_number`, `gm.with(target, fn)` (`target`: экземпляр, объект или `all`; внутри `fn(self, other)`), `event_inherited`.
  - `gm.step()` и `gm.draw()` выполняют события в порядке из Global Constraints, по возрастанию `depth` в обратном порядке для Draw (большая глубина рисуется раньше).
  - `room_goto(name)`, `room_restart`, `game_end` (выполняет `Other_3` и `CleanUp_0`), `global` (таблица).
  - `steam.lua`: все `steam_*` из `re/builtins.json` возвращают нейтральные значения (`false`, `0`, `""`) без побочных эффектов.

- [ ] **Step 1: Write the failing tests**

```lua
zero_based = function() local a = gm.array(10, 20, 30); assert(a[0] == 10 and gm.array_length(a) == 3) end,
utf8_strings = function() assert(gm.string_length("Мицелий") == 7 and gm.string_char_at("Мицелий", 1) == "М") end,
undefined_kept = function() local s = gm.struct({}); s.x = gm.undefined; assert(gm.variable_struct_exists(s, "x")) end,
method_binds_self = function() local s = gm.struct({v = 5}); local m = gm.method(s, function(self) return self.v end); assert(m() == 5) end,
event_order = function() -- объект пишет в лог имя события; один кадр step+draw даёт порядок из Global Constraints
  ... assert(table.concat(log, ",") == "Step_1,Step_0,Draw_0,Draw_64") end,
draw_depth = function() -- два экземпляра с depth 10 и -10: Draw сначала у depth 10
  ... end,
with_other = function() -- внутри gm.with(obj_b, fn) other — вызывающий экземпляр
  ... end,
```

- [ ] **Step 2:** FAIL. **Step 3:** Реализовать. **Step 4:** `luajit tests/run.lua` → PASS.
- [ ] **Step 5:** Коммит `feat: GML core runtime (values, instances, events, rooms)`.

---

### Task 8: Ресурсы и рисование

**Files:**
- Create: `game/gm/assets.lua`, `game/gm/draw.lua`, `game/gm/input.lua`, `game/gm/audio.lua`, `game/gm/file.lua`
- Test: `tests/test_draw_math.lua`

**Interfaces:**
- Consumes: формат `build/assets/*.json` (задача 3), `display` (задача 6).
- Produces:
  - `gm.assets.load()` — читает JSON, создаёт `love.graphics.newImage` по страницам и `Quad` по кадрам. Ресурсы доступны как глобальные идентификаторы игры (`apple`, `fnt_gm_20`, `bird`, `sh_glow_blur`, `Room1`), как в GML.
  - `draw_sprite(spr, sub, x, y)`, `draw_sprite_ext(spr, sub, x, y, xs, ys, rot, col, alpha)` — с origin, обрезкой TPAG (`dst`/`bound`), поворотом против часовой стрелки в градусах, `sub` по модулю числа кадров, дробный `sub` округляется вниз.
  - Цвета GM — целые BGR: `make_colour_rgb`, `c_white` и т.д.; `draw_set_colour`, `draw_set_alpha`, `draw_set_font`, `draw_set_halign`/`valign`, `draw_text`, `draw_text_ext`, `draw_text_transformed`, `draw_rectangle`, `draw_line`, `draw_circle` и т.д. по `re/builtins.json`.
  - `surface_create/set_target/reset_target/free/exists`, `draw_surface*` на `love.graphics.newCanvas`. `shader_set/reset`, `shader_get_uniform`, `shader_set_uniform_f` — GLSL ES из SHDR адаптирован к LÖVE.
  - `font_add(path, size, ...)` для внешних TTF/OTF из `fonts/`.
  - `input.lua`: `mouse_check_button(_pressed/_released)(mb_left|mb_right)`, `mouse_x/mouse_y` (комнатные координаты через камеру), `device_mouse_x_to_gui`, `keyboard_check*`, колесо `mouse_wheel_up/down`.
  - `audio.lua`, `file.lua`: функции из `re/builtins.json` (`audio_*`, `ini_*`, `json_stringify/parse`, `file_*`) поверх `love.audio` и `love.filesystem`.
  - Чистые функции для тестов: `draw.bgr_to_rgb(c) -> r,g,b` (0..1), `draw.frame_offset(spr_meta, frame, xs, ys) -> ox, oy`.
- Объём: реализовать функции, которые реально вызывают объекты меню (`obj_game`, `obj_gm_*`) по `re/inventory.md`. Для остальных из `re/builtins.json` — заглушка с `error("gm: not implemented: <name>")`, чтобы пробелы были видны сразу.

- [ ] **Step 1: Write the failing tests**

```lua
bgr = function() local r, g, b = draw.bgr_to_rgb(0x0000FF); assert(r == 1 and g == 0 and b == 0) end,
origin_with_trim = function()   -- origin (32,32), кадр обрезан: dst=(4,6)
  local meta = {origin = {32, 32}, frames = {{dst = {4, 6}}}}
  local ox, oy = draw.frame_offset(meta, 0, 1, 1); assert(ox == -28 and oy == -26)
end,
```

- [ ] **Step 2:** FAIL. **Step 3:** Реализовать. **Step 4:** `luajit tests/run.lua` → PASS.
- [ ] **Step 5:** Визуальная проверка: временная сцена с `draw_sprite` для `apple`, `hedgehog`-подобного анимированного спрайта и `draw_text` шрифтом `fnt_gm_30` с русской строкой из `localization.csv`; `MYCO_SHOT=build/shot_draw.png:30 love game`, открыть PNG.
- [ ] **Step 6:** Коммит `feat: sprites, text, surfaces, shaders, input, audio and file APIs`.

---

### Task 9: Сборка `Mycelarium.app`

**Files:**
- Create: `tools/package.sh`, `tools/Info.plist.in`, `tools/icon.icns` (сгенерировать из `splash.png` или спрайта логотипа через `sips`/`iconutil`, в `build/`, не коммитить, если взят из игры)

**Interfaces:**
- Produces: `dist/Mycelarium.app` — копия `love.app` из brew cask, `Contents/Resources/Mycelarium.love` (zip из `game/` + `build/assets/`), `CFBundleName=Mycelarium`, `CFBundleIdentifier=local.dga.mycelarium`, `LSMinimumSystemVersion=12.0`, `NSHighResolutionCapable=true`; `codesign --force --deep -s -`.

- [ ] **Step 1:** Написать `package.sh` (останавливается с ошибкой, если нет `build/assets` или `love.app`).
- [ ] **Step 2:** Запустить `tools/package.sh && open dist/Mycelarium.app`. Ожидается: приложение стартует с тестовой сценой задачи 8, `codesign -v dist/Mycelarium.app` без ошибок.
- [ ] **Step 3:** Коммит `chore: package Mycelarium.app`.

---

### Task 10: Главное меню (веха фазы 1)

**Files:**
- Create: `game/myco/objects/obj_game.lua`, `game/myco/objects/obj_gm_button.lua`, `obj_gm_button1.lua`, `obj_gm_text.lua`, `obj_gm_text1.lua`, `obj_gm_textbox.lua`, `game/myco/scripts/scr_*.lua` (только функции, достижимые из меню), `game/myco/init.lua`
- Modify: `game/main.lua` (вместо тестовой сцены — `room_goto(Main_menu)`, или стартовая комната оригинала по `rooms.json`/порядку комнат)
- Reference: `re/reference/menu_*.png`

**Interfaces:**
- Consumes: всё из задач 6–8.
- Produces: `game/myco/init.lua` регистрирует объекты и скрипты; каждая Lua-функция начинается комментарием `-- gml_Script_<name> @ 0x<адрес>`, чтобы её можно было сверить с декомпиляцией.

Объём: события объектов, экземпляры которых стоят в стартовой комнате и в `Main_menu` (по `rooms.json`), и транзитивно все `gml_Script_*`, которые они вызывают (граф из `re/inventory.md`). Сюда входят локализация (`scr_locale`), логотип (`scr_logo`, если вызывается из меню), музыка меню (`scr_audio`), чтение и запись настроек и списка сохранений (`scr_save`, только то, что нужно меню). Переход «новая игра» в фазе 1 может вести на заглушку комнаты `Room1`.

- [ ] **Step 1:** Снять эталон: попросить владельца запустить оригинал в GameHub и пройти по экранам меню. Скриншоты (`screencapture -x`) положить в `re/reference/menu_*.png`. Записать в `re/notes/menu.md` список элементов и их поведение при клике и наведении.
- [ ] **Step 2:** Перевести функции из объёма, файл за файлом, сверяя с `re/decomp/`.
- [ ] **Step 3:** `MYCO_SHOT=build/menu.png:120 love game`, сравнить с `re/reference/menu_main.png`: расположение, шрифты, цвета, анимация логотипа. Расхождения исправить.
- [ ] **Step 4:** Ручная проверка владельцем на `dist/Mycelarium.app` (после `tools/package.sh`): каждая кнопка меню срабатывает при клике по её краям, в окне и в фуллскрине; переключение языка; громкость; выход. Найденные отличия записать в `re/notes/menu.md` и закрыть.
- [ ] **Step 5:** Коммит `feat: main menu ported`.

---

### Task 11: Планы фаз 2 и 3

**Files:**
- Create: `docs/superpowers/plans/2026-MM-DD-mycelarium-love-port-phase2.md` (мир: `Room1`, камера, `scr_world`, `scr_worldgen`, `obj_forest_controller`, `obj_mycelium_controller`, `obj_hud`, `scr_visual`, `scr_balance`, базовое сохранение), `...-phase3.md` (`obj_life`/`scr_life`, `scr_stones`, `scr_weather`, `scr_upgrades`, `scr_skilltree`, полный `scr_save`, шейдеры, звук мира, финальная сверка с оригиналом).

- [ ] **Step 1:** По `re/inventory.md` разбить функции фаз 2–3 на задачи по подсистемам. Решить, воспроизводить ли ГСЧ WELL512: если `scr_worldgen` зависит от `random_set_seed`/`irandom`, добавить задачу на точную реализацию с тестом против известных значений из декомпиляции рантайма.
- [ ] **Step 2:** Показать планы владельцу и коммит `docs: phase 2 and 3 plans`.
