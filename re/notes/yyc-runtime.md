# Как читать декомпиляцию YYC

Каждая GML-функция в `re/decomp/` скомпилирована YYC в C++. Сигнатуры:

- события объектов: `gml_Object_<obj>_<Event>_<n>(CInstance* self, CInstance* other)`;
- скрипты: `gml_Script_<name>(self, other, RValue* result, int argc, RValue** args)`,
  аргумент i — это `args[i]`; если `argc <= i`, подставляется константа undefined (`&DAT_140fd77d0`).

## RValue

16 байт: `+0` значение (double / указатель), `+0xc` kind. Kind: 0 real, 1 string, 2 array, 3 ptr,
5 undefined, 6 struct/object, 7 int32, 10 int64, 13 bool, 15 ref. Маска `0x46` (`kind` 1, 2, 6) —
типы со счётчиком ссылок: перед перезаписью значение освобождается через `FREE_RValue`.

## Известные помощники рантайма

| имя в Ghidra | что делает |
|---|---|
| `FREE_RValue` (0x1400243e0) | освободить RValue перед перезаписью |
| `COPY_RValue` (0x140024510) | присвоить RValue (dst, src) |
| `COPY_RValue_Post` (0x140024340) | копирование ссылочного RValue |
| `YYGML_CallBuiltin` (0x1406db110) | вызов встроенной функции GML: `(self, other, result, argc, id, args)`, где id — `vid_<имя функции>` |
| `YYCompareVal` (0x14070edc0) | сравнение двух RValue `(a, b, epsilon, ...)` → 0, если равны |
| `YYError` (0x1406dacd0) | ошибка рантайма (строка-сообщение) |
| `FUN_14072b4d0` | `abs` |
| `thunk_FUN_140b31e60` | `sin` |
| `FUN_14004ba30`, `FUN_1400a93b0` | целочисленное `div` |
| `FUN_140024e60` | сравнение «меньше» |
| `FUN_14004b9c0`, `FUN_14002f610` | сравнение «больше или равно» |
| `FUN_14002f5a0(x, 0)` | `x > 0` |
| `FUN_140045910` / `FUN_140024a60` / `FUN_14004b680` | `a - n` / `a + n` / `a - d` |
| `FUN_140024e30` / `FUN_14002f570` | `<` / `>` |
| `FUN_140b29df0` | `fmod` |
| `FUN_1408c9bd0` / `FUN_1408c9c40` / `FUN_1408c9c70` | `draw_set_font` / `draw_set_halign` / `draw_set_valign` |
| `FUN_140754f80` / `FUN_140756730` | `surface_set_target` / `surface_reset_target` |
| `g_pGlobal` (0x141020570) | глобальный экземпляр: `(*g_pGlobal->vtbl[1])(g_pGlobal, vid_x)` → ссылка на `global.x` |
| `g_pCurrentStackTrace` | кадр стек-трейса; строки `local_xx = N` рядом — номера строк GML |

`vid_<имя>` — слоты id из таблицы имён: переменные экземпляров и глобальные переменные, встроенные функции
(список в `re/runtime_builtins.txt`) и скрипты. Обращение к переменной экземпляра выглядит как вызов через
vtable `self` с `vid_<имя>`.

Числовые константы часто лежат в `.rdata` как double (`_DAT_140f....`). Значение смотреть в Ghidra
(`~/GhidraProjects/myco`) или через `tools/` скрипты по адресу.
