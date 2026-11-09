"""Parse GameMaker 2024+ resource chunks (bytecode 17) into plain dicts.

Layouts were checked against this game's data.win; field offsets are relative
to each resource entry.
"""

import struct

from .datawin import DataWin, DataWinError

LAYER_TYPES = {1: "background", 2: "instances", 3: "assets", 4: "tiles", 6: "effect"}


def _names(dw: DataWin, chunk: str) -> list[str]:
    return [dw.string(dw.u32(p)) for p in dw.chunk_list(chunk)]


def _ref(names: list[str], idx: int) -> str | None:
    return names[idx] if 0 <= idx < len(names) else None


def tpag(dw: DataWin, ptr: int) -> dict:
    sx, sy, sw, sh, tx, ty, tw, th, bw, bh, page = struct.unpack_from("<11H", dw.data, ptr)
    return {"page": page, "src": [sx, sy, sw, sh], "dst": [tx, ty], "bound": [bw, bh]}


def sprites(dw: DataWin) -> dict:
    out = {}
    for p in dw.chunk_list("SPRT"):
        name = dw.string(dw.u32(p))
        if dw.i32(p + 56) != -1 or dw.u32(p + 60) != 3 or dw.u32(p + 64) != 0:
            raise DataWinError(f"unsupported sprite layout for {name}")
        out[name] = {
            "w": dw.u32(p + 4),
            "h": dw.u32(p + 8),
            # margins are stored left, right, bottom, top
            "bbox": [dw.i32(p + 12), dw.i32(p + 24), dw.i32(p + 16), dw.i32(p + 20)],
            "origin": [dw.i32(p + 48), dw.i32(p + 52)],
            "speed": dw.f32(p + 68),
            "speed_type": dw.u32(p + 72),
            "frames": [tpag(dw, t) for t in dw.pointer_list(p + 84)],
        }
    return out


def fonts(dw: DataWin) -> dict:
    out = {}
    for p in dw.chunk_list("FONT"):
        name = dw.string(dw.u32(p))
        size = dw.f32(p + 8)
        glyphs = {}
        for g in dw.pointer_list(p + 56):
            ch, x, y, w, h, shift, offset = struct.unpack_from("<HHHHHhh", dw.data, g)
            glyphs[str(ch)] = {"x": x, "y": y, "w": w, "h": h, "shift": shift, "offset": offset}
        out[name] = {
            "display_name": dw.string(dw.u32(p + 4)),
            "size": -size if size < 0 else size,
            "bold": bool(dw.u32(p + 12)),
            "italic": bool(dw.u32(p + 16)),
            "first": dw.u16(p + 20),
            "last": dw.u32(p + 24),
            "frame": tpag(dw, dw.u32(p + 28)),
            "scale": [dw.f32(p + 32), dw.f32(p + 36)],
            "ascender_offset": dw.i32(p + 40),
            "ascender": dw.u32(p + 44),
            "line_height": dw.u32(p + 52),
            "glyphs": glyphs,
        }
    return out


def sounds(dw: DataWin) -> tuple[dict, dict]:
    """Returns (metadata, {name: embedded bytes or None for external files})."""
    audo = dw.chunk_list("AUDO")
    meta, blobs = {}, {}
    for p in dw.chunk_list("SOND"):
        name = dw.string(dw.u32(p))
        audio_id, group = dw.i32(p + 32), dw.i32(p + 28)
        source = dw.string(dw.u32(p + 12))
        if audio_id >= 0 and group == 0:
            a = audo[audio_id]
            blob = bytes(dw.mv[a + 4:a + 4 + dw.u32(a)])
            ext = ".ogg" if blob[:4] == b"OggS" else ".wav" if blob[:4] == b"RIFF" else ".mp3"
            blobs[name] = blob
            streamed = False
        else:
            blobs[name] = None
            ext = "." + source.rsplit(".", 1)[-1]
            streamed = True
        meta[name] = {"file": name + ext, "source": source, "volume": dw.f32(p + 20), "pitch": dw.f32(p + 24),
                      "group": group, "streamed": streamed}
    return meta, blobs


def shaders(dw: DataWin) -> dict:
    return {dw.string(dw.u32(p)): {"vsh": dw.string(dw.u32(p + 8)), "fsh": dw.string(dw.u32(p + 12))}
            for p in dw.chunk_list("SHDR")}


def objects(dw: DataWin) -> dict:
    names = _names(dw, "OBJT")
    sprite_names = _names(dw, "SPRT")
    out = {}
    for p in dw.chunk_list("OBJT"):
        name = dw.string(dw.u32(p))
        nverts = dw.u32(p + 68)
        ev_off = p + 84 + nverts * 8
        events = []
        for etype, lp in enumerate(dw.pointer_list(ev_off)):
            for ep in dw.pointer_list(lp):
                events.append([etype, dw.u32(ep)])
        out[name] = {
            "sprite": _ref(sprite_names, dw.i32(p + 4)),
            "visible": bool(dw.u32(p + 8)),
            "solid": bool(dw.u32(p + 16)),
            "depth": dw.i32(p + 20),
            "persistent": bool(dw.u32(p + 24)),
            "parent": _ref(names, dw.i32(p + 28)),
            "mask": _ref(sprite_names, dw.i32(p + 32)),
            "events": events,
        }
    return out


def _layer(dw: DataWin, p: int, sprite_names: list[str]) -> dict:
    ltype = LAYER_TYPES.get(dw.u32(p + 8), str(dw.u32(p + 8)))
    layer = {"name": dw.string(dw.u32(p)), "id": dw.u32(p + 4), "type": ltype, "depth": dw.i32(p + 12),
             "offset": [dw.f32(p + 16), dw.f32(p + 20)], "speed": [dw.f32(p + 24), dw.f32(p + 28)],
             "visible": bool(dw.u32(p + 32))}
    d = p + 48
    if ltype == "instances":
        layer["instances"] = dw.pointer_list(d)
    elif ltype == "background":
        layer["background"] = {"visible": bool(dw.u32(d)), "foreground": bool(dw.u32(d + 4)),
                               "sprite": _ref(sprite_names, dw.i32(d + 8)), "tiled": [bool(dw.u32(d + 12)), bool(dw.u32(d + 16))],
                               "stretch": bool(dw.u32(d + 20)), "color": dw.u32(d + 24), "first_frame": dw.f32(d + 28),
                               "anim_speed": dw.f32(d + 32), "anim_speed_type": dw.u32(d + 36)}
    elif ltype == "assets":
        layer["sprites"] = [{"name": dw.string(dw.u32(s)), "sprite": _ref(sprite_names, dw.i32(s + 4)),
                             "x": dw.i32(s + 8), "y": dw.i32(s + 12), "scale": [dw.f32(s + 16), dw.f32(s + 20)],
                             "color": dw.u32(s + 24), "anim_speed": dw.f32(s + 28), "anim_speed_type": dw.u32(s + 32),
                             "frame": dw.f32(s + 36), "angle": dw.f32(s + 40)}
                            for s in dw.pointer_list(dw.u32(d + 4))]
    return layer


def rooms(dw: DataWin) -> dict:
    obj_names = _names(dw, "OBJT")
    sprite_names = _names(dw, "SPRT")
    out = {}
    for p in dw.chunk_list("ROOM"):
        name = dw.string(dw.u32(p))
        views = []
        for v in dw.pointer_list(dw.u32(p + 44)):
            vals = [dw.i32(v + 4 * i) for i in range(14)]
            views.append({"enabled": bool(vals[0]), "view": vals[1:5], "port": vals[5:9], "border": vals[9:11],
                          "speed": vals[11:13], "object": _ref(obj_names, vals[13])})
        instances = []
        for i in dw.pointer_list(dw.u32(p + 48)):
            instances.append({"x": dw.i32(i), "y": dw.i32(i + 4), "object": _ref(obj_names, dw.i32(i + 8)),
                              "id": dw.u32(i + 12), "creation_code": dw.i32(i + 16),
                              "scale_x": dw.f32(i + 20), "scale_y": dw.f32(i + 24), "image_speed": dw.f32(i + 28),
                              "image_index": dw.i32(i + 32), "color": dw.u32(i + 36), "angle": dw.f32(i + 40),
                              "pre_create_code": dw.i32(i + 44)})
        flags = dw.u32(p + 36)
        out[name] = {
            "w": dw.u32(p + 8), "h": dw.u32(p + 12), "speed": dw.u32(p + 16), "persistent": bool(dw.u32(p + 20)),
            "color": dw.u32(p + 24), "draw_color": bool(dw.u32(p + 28)), "creation_code": dw.i32(p + 32),
            "flags": flags, "views_enabled": bool(flags & 1), "views": views, "instances": instances,
            "creation_order": dw.pointer_list(dw.u32(p + 56)) if dw.u32(dw.u32(p + 56)) == len(instances) else [],
            "gravity": [dw.f32(p + 80), dw.f32(p + 84)],
            "layers": [_layer(dw, l, sprite_names) for l in dw.pointer_list(dw.u32(p + 92))],
        }
    return out


def game(dw: DataWin) -> dict:
    g = dw.gen8()
    o = dw.chunks["GEN8"][0]
    room_names = _names(dw, "ROOM")
    g["room_order"] = [room_names[i] for i in dw.pointer_list(o + 128)]
    return g
