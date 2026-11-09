# Экран: оригинал и порт

## Как это делает оригинал (gml_Script_display_fit, восстановлено из декомпиляции)

```
ww, wh = window_get_width(), window_get_height()
if ww <= 0 or wh <= 0 or !surface_exists(application_surface): return false
aspect = clamp(ww / wh, 1.25, 32/9)
if aspect >= 16/9: vh = 1080; vw = round(1080 * aspect)
else:              vw = 1920; vh = round(1920 / aspect)
k = min(ww / vw, wh / vh, sqrt(3840*2160 / (vw*vh)))
surf_w = max(round(vw*k), 1); surf_h = max(round(vh*k), 1)
if ничего не поменялось: return false
global.view_w/view_h/surf_w/surf_h = ...
global.ui_ox = (vw - 1920) * 0.5;  global.ui_by = vh - 1080
surface_resize(application_surface, surf_w, surf_h); display_set_gui_size(vw, vh)
освободить global.logo_surface, если есть; return true
```

Рамок нет: вид растёт по соотношению сторон окна, 1920×1080 — минимум. `application_surface_draw_enable(false)`
в `obj_game.Create`; полотно выводит сам `obj_game.Draw_77` → `post_layer_draw` по `application_get_position`.
`display_step` применяет отложенный размер окна (`display_window_size`, `window_set_size`, `window_center`)
и вызывает `display_fit`; `display_apply` переключает фуллскрин по `global.display_fullscreen`.

Вероятная причина бага под Wine: `window_get_width/height` возвращали не тот размер, что реально у окна
(Retina/вырез), и вид/GUI считались от него, а мышь — от настоящего окна.

## Замеры LÖVE 11.5 на MacBook Pro 14" (2026-11-09)

Фуллскрин (`setFullscreen(true, "desktop")`, `highdpi=true`, `usedpiscale=false`):
- окно и drawable: 3024×1898 пикселей — macOS сама кладёт окно ниже выреза;
- `getSafeArea()` = 0,0 3024×1898, `getDPIScale()` = 1.00 (координаты уже в пикселях);
- `view_for_window(3024, 1898)` → вид 1920×1205, полотно 3024×1898, `ui_by` = 125.
