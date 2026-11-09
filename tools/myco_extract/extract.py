"""Extract Mycelarium resources from the installed game into build/assets/."""

import argparse
import json
import os
import shutil
import sys
from pathlib import Path

from PIL import Image

from . import resources
from .datawin import DataWin, DataWinError, game_dir
from .qoi import decode_texture

REPO = Path(__file__).resolve().parents[2]
EXTERNAL_FILES = ["localization.csv", "basic_items.json", "splash.png"]


def _write_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=1))


def extract_all(src: Path, out: Path, link: bool = True) -> None:
    dw = DataWin.open(str(src / "data.win"))
    out.mkdir(parents=True, exist_ok=True)

    pages = out / "pages"
    pages.mkdir(exist_ok=True)
    for i, ptr in enumerate(dw.chunk_list("TXTR")):
        w, h, px = decode_texture(dw, ptr)
        Image.frombytes("RGBA", (w, h), px).save(pages / f"page{i}.png")

    for name, fn in [("sprites", resources.sprites), ("fonts", resources.fonts), ("objects", resources.objects),
                     ("rooms", resources.rooms), ("game", resources.game)]:
        _write_json(out / f"{name}.json", fn(dw))

    meta, blobs = resources.sounds(dw)
    snd = out / "sounds"
    snd.mkdir(exist_ok=True)
    for name, info in meta.items():
        if blobs[name] is not None:
            (snd / info["file"]).write_bytes(blobs[name])
        else:
            shutil.copyfile(src / info["source"], snd / info["file"])
    _write_json(out / "sounds.json", meta)

    sh = out / "shaders"
    sh.mkdir(exist_ok=True)
    for name, code in resources.shaders(dw).items():
        (sh / f"{name}.vsh").write_text(code["vsh"])
        (sh / f"{name}.fsh").write_text(code["fsh"])

    for name in EXTERNAL_FILES:
        shutil.copyfile(src / name, out / name)
    shutil.copytree(src / "fonts", out / "fonts", dirs_exist_ok=True)

    if link:
        target = REPO / "game" / "assets"
        target.parent.mkdir(exist_ok=True)
        if target.is_symlink() or target.exists():
            target.unlink()
        target.symlink_to(os.path.relpath(out, target.parent))


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--game-dir", type=Path, default=game_dir())
    ap.add_argument("--out", type=Path, default=REPO / "build" / "assets")
    args = ap.parse_args(argv)
    if not (args.game_dir / "data.win").exists():
        print(f"game not found: {args.game_dir} (set MYCO_GAME_DIR)", file=sys.stderr)
        return 2
    try:
        extract_all(args.game_dir, args.out)
    except DataWinError as e:
        print(f"extract failed: {e}", file=sys.stderr)
        return 1
    print(f"assets written to {args.out}")
    return 0
