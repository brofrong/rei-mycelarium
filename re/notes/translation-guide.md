# Руководство по переводу YYC → Lua (LÖVE)

Цель: построчно верный перевод логики Mycelarium из декомпиляции на Lua поверх слоя `game/gm/`.
Не оптимизировать, не «улучшать», не придумывать. Если что-то неясно — `-- TODO(port): <что неясно>`.

## Что читать

- Твой список функций: `re/chunks/<chunk>.txt` (имя, путь к очищенному коду, размер).
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

**Одна GML-функция = одна глобальная функция Lua с тем же именем.** Это касается всех функций из твоего
списка, включая события объектов, анонимные функции и конструкторы литералов структур:

| GML-символ | имя функции Lua |
|---|---|
| `gml_Script_hedgehog_frame_h` | `hedgehog_frame_h` (префикс `gml_Script_` убирается) |
| `gml_Script_anon__799__gml_Object_obj_forest_controller_Create_0` | `anon__799__gml_Object_obj_forest_controller_Create_0` |
| `gml_Script____struct___367__gml_Object_obj_hud_Create_0` | `___struct___367__gml_Object_obj_hud_Create_0` (после префикса — три подчёркивания) |
| `gml_Object_obj_hud_Create_0` | `gml_Object_obj_hud_Create_0` (имя целиком) |
| `gml_GlobalScript_scr_world` | `gml_GlobalScript_scr_world` (верхнеуровневый код файла, вызывается при старте) |

Сигнатура всегда `(self, other, ...аргументы GML)`:
```lua
-- gml_Script_hedgehog_frame_h @ 0x1400a9780
function hedgehog_frame_h(self, other)
  return gm.sprite_get_height(hedgehog) * vis(self, other, "hedgehog_scale")
end
```
Вызов другой функции (даже из чужого файла — все глобальные): `f(self, other, a, b)`.

**Где функция создаёт метод/анонимную функцию** (в YYC: ссылка на `gml_Script_anon__…` + создание метода) →
`gm.method(self, anon__799__gml_Object_obj_forest_controller_Create_0)` (привязка к текущему self; если метод
создаётся внутри литерала структуры — привязка к этой структуре).
**Где функция создаёт литерал структуры** (вызов конструктора `gml_Script____struct___N__…` с захваченными значениями) →
`gm.new(___struct___N__…, args...)`. Сам конструктор переводится как обычная функция, которая заполняет поля `self`
(и привязывает свои методы к `self` через `gm.method(self, …)`).
Именованные конструкторы (`function Foo() constructor`) — так же: `gm.new(Foo, ...)`.
Вызов метода или переменной-функции: `m(self, other, args)` — одинаково для функций и методов.

События объектов регистрирует интегратор (`gm.object(...)` с этими функциями) — тебе нужно только написать
функции `gml_Object_<obj>_<Event>_<n>(self, other)`.

**Файл.** Пиши только тот файл, который указан в задании (`game/myco/gen/<chunk>.lua`), начиная с
`local gm = require("gm")`. Не трогай `game/gm/`, `game/main.lua`, чужие файлы, не делай git commit,
не создавай git worktree и **не запускай других агентов** — переводи сам.

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
2. Каждая функция из твоего списка переведена полностью. Заглушки, «упрощённые версии» и `return false` вместо
   логики недопустимы. Единственная допустимая пометка — `-- TODO(port): <конкретная неясность>` внутри переведённой
   функции. Счёт в отчёте должен быть честным.
3. В отчёте: число функций, список встроенных функций `gm.*`, которых нет в `game/gm/`, список TODO
   с именами функций, неясности в соглашениях.
