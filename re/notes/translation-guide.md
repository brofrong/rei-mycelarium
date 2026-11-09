# Руководство по переводу YYC → Lua (LÖVE)

Цель: построчно верный перевод логики Mycelarium из декомпиляции на Lua поверх слоя `game/gm/`.
Не оптимизировать, не «улучшать», не придумывать. Если что-то неясно — `-- TODO(port): <что неясно>`.

## Что читать

- Твой список функций: `re/partition/<part>.txt` (имя, путь к очищенному коду, размер).
- Очищенный код: `re/lifted/**/<name>.txt` (сделан `tools/lift.py` из `re/decomp/`). Если в очищенном коде
  что-то потерялось — смотри исходную декомпиляцию `re/decomp/**/<name>.c`.
- Как читать YYC: `re/notes/yyc-runtime.md`. Кто кого вызывает: `re/callgraph.json`.
- Ресурсы (порядок = индексы): `build/assets/{sprites,sounds,fonts,objects,rooms}.json`.
- Слой совместимости (только читать!): `game/gm/*.lua`. Список того, что там есть: `grep -o "^function M\.[a-z_0-9]*\|^function gm\.[a-z_0-9]*" game/gm/*.lua`.

## Обозначения в очищенном коде

| в коде | значит |
|---|---|
| `// L123` | номер строки исходного GML (порядок операторов) |
| `&self.x`, `&other.x`, `&global.x`, `&V->x` | переменная экземпляра / other / глобальная / поле структуры V |
| `name!(n, args)` | вызов встроенной функции GML `name` с n аргументами; `args` — массив указателей на RValue, заполняемый строками выше |
| `gml_Script_f(param_1,param_2,&ret,n,args)` | вызов скриптовой функции `f` (self, other, результат, argc, args) |
| `GET_BUILTIN_VAR(self, vid_x, …, &dst)` | чтение встроенной переменной (`application_surface`, `room`, `mouse_x`, …) → `gm.<x>` |
| `CMP(a,b)` | сравнение: `-1` меньше, `0` равно, `1` больше, `-2` несравнимо |
| `OP_MUL/OP_DIV/OP_SUB(&a,&b)` | `a *= b`, `a /= b`, `a -= b` |
| `clamp/min_n/max_n` | встроенные clamp/min/max |
| `TO_BOOL(x)`, `TO_REAL(x)` | приведение к bool / числу |
| `"text"`, `1.0`, `true`, `undefined` | подставленные константы |
| `sprite(name)`, `sound(name)`, `object(name)`, `font(name)`, `shader(name)` | ссылка на ресурс (типы font/shader — догадка) |
| `param_3` | RValue результата: последнее присваивание `param_3 = …` перед `return` — это `return …` |
| `param_4` / `param_5[i]` | argc / аргумент i скрипта; `if (param_4 < i+1) → undefined` — значение по умолчанию |

Мусорные присваивания вида `local_1e4 = 5;` (тип RValue) и копирования временных можно пропускать.

## Соглашения Lua

**Файлы.** Каждый агент пишет только свои файлы:
- скрипты: `game/myco/scripts/<part>.lua` (например `scr_life_a.lua`; для части `core`/`meta`/`ui` —
  по файлу на исходный scr_*: `scr_game.lua`, `scr_logo.lua`, …);
- объекты: `game/myco/objects/<obj>.lua`.
Каждый файл начинается с `local gm = require("gm")`. Не трогай `game/gm/`, `game/main.lua`, чужие файлы,
не делай git commit.

**Скриптовые функции** — глобальные функции Lua с именем GML без префикса `gml_Script_`:
```lua
-- gml_Script_hedgehog_frame_h @ 0x1400a9780
function hedgehog_frame_h(self, other)
  return gm.sprite_get_height(hedgehog) * vis(self, other, "hedgehog_scale")
end
```
Первые два параметра всегда `self, other`; дальше аргументы GML. Вызов другой скриптовой функции:
`f(self, other, a, b)` — даже если она в чужом файле (все скрипты глобальные).

**Объекты:**
```lua
obj_hud = gm.object("obj_hud", {
  -- gml_Object_obj_hud_Create_0 @ 0x...
  Create_0 = function(self, other) ... end,
  Step_1 = function(self, other) ... end,
  Draw_75 = function(self, other) ... end,
})
```
Имена событий = суффикс YYC (`Create_0`, `Step_0`, `Step_1`, `Draw_0`, `Draw_64`, `Draw_73..77`, `CleanUp_0`, `Other_3`).
Родитель (если есть, см. `build/assets/objects.json`) — поле `parent = "obj_x"`.

**Глобальные скрипты** `gml_GlobalScript_scr_x`: верхнеуровневый код файла (не объявления функций) →
`function __global_scr_x(self, other) ... end` (вызовется при старте). Если там только регистрация функций — пустая функция.

**Анонимные функции и методы** (`gml_Script_anon__NNN__<родитель>`) переводятся там, где создаются,
как замыкание, привязанное к self (GML `method`/`function` внутри события или структуры):
`self.on_click = gm.method(self, function(self, other, ...) ... end)`.
Литералы структур (`gml_Script____struct___N__<родитель>` — конструктор литерала `{...}`) →
`gm.struct({ a = 1, b = gm.method(nil, function(self, other) ... end) })`, затем методы привязываются к самой структуре:
`local s = gm.struct({...}); s.f = gm.method(s, function(self, other) ... end)`.
Именованные конструкторы (`function Foo() constructor`) → `function Foo(self, other, ...)` + `gm.new(Foo, ...)`.
Вызов метода/переменной-функции: `m(self, other, args)` — одинаково для функций и методов.

**Значения:**
- переменные экземпляра: `self.x`; глобальные: `global.x` (`global` — глобальная таблица Lua = `gm.global`); `other.x`.
- `var` → `local`. `static` → upvalue-таблица вне функции.
- массивы: `gm.array(1, 2)`, индексы с **0**: `a[0]`, `gm.array_length(a)`; литерал `[]` → `gm.array()`.
- структуры: `gm.struct({...})`; `undefined` → `gm.undefined`; проверка `gm.is_undefined(v)` (понимает и nil).
- **Истинность:** в GML числа — логические значения (`0` — ложь), в Lua `0` — истина! Условия по числам пиши
  `if gm.bool(x) then` или явно `x ~= 0`/`x > 0.5`; `!x` → `not gm.bool(x)`. Сравнения и функции, возвращающие
  bool, можно использовать напрямую. Арифметика с bool → `gm.real(b)`.
- строки: GML `+` над строками → `..`; число в строку → `gm.string(v)`.
- `div` → `gm.div(a, b)`, `mod`/`%` → `gm.mod(a, b)`, битовые операции → `bit.band/bor/bxor/lshift/rshift` (LuaJIT `bit`).
- `with (t) {…}` → `gm.with(self, t, function(self, other) … end)`.
- `repeat (n)` → `for _ = 1, n do`; `for` из GML с изменяемой границей → `while`.
- `switch` → `if/elseif`, сохраняя fallthrough.
- `try/catch` → `pcall`.

**Встроенные функции и константы GML** — `gm.<имя>` с теми же аргументами: `gm.draw_text(x, y, s)`,
`gm.c_white`, `gm.fa_center`, `gm.bm_add`, `gm.vk_escape`, `gm.mb_left`. Встроенные переменные:
`gm.room`, `gm.room_width`, `gm.mouse_x`, `gm.application_surface`, `gm.view_camera[0]`.
Ресурсы — глобальные имена: `hedgehog`, `fnt_gm_20`, `bird`, `sh_glow_blur`, `Room1`; объекты — `obj_hud`.
Если нужной функции нет в `game/gm/` — всё равно пиши `gm.<имя>(...)` и внеси её в отчёт.

**Комментарии:** перед каждой функцией/событием `-- gml_<полное имя> @ 0x<адрес>`. Внутри — ничего лишнего,
только `-- TODO(port): ...` там, где не уверен.

## Проверка перед сдачей

1. `luajit -bl game/myco/scripts/<file>.lua > /dev/null` (и для объектов) — без синтаксических ошибок.
2. Каждая функция из `re/partition/<part>.txt` переведена (или явно помечена `-- TODO(port): not translated`).
3. В отчёте: число функций, список встроенных функций `gm.*`, которых нет в `game/gm/`, список TODO
   с именами функций, неясности в соглашениях.
