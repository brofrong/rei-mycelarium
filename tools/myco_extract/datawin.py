"""Reader for the GameMaker data.win container (FORM + chunks)."""

import os
import struct
from pathlib import Path


class DataWinError(Exception):
    pass


def game_dir() -> Path:
    env = os.environ.get("MYCO_GAME_DIR")
    if env:
        return Path(env).expanduser()
    return Path.home() / "GameHub/steamapps/common/Mycelarium"


class DataWin:
    def __init__(self, path: str, data: bytes):
        self.path = path
        self.data = data
        self.mv = memoryview(data)
        self.chunks: dict[str, tuple[int, int]] = {}
        pos, end = 8, 8 + struct.unpack_from("<I", data, 4)[0]
        while pos + 8 <= min(end, len(data)):
            name = bytes(data[pos:pos + 4]).decode("ascii", "replace")
            size = struct.unpack_from("<I", data, pos + 4)[0]
            self.chunks[name] = (pos + 8, size)
            pos += 8 + size

    @classmethod
    def open(cls, path: str) -> "DataWin":
        if not os.path.exists(path):
            raise DataWinError(f"game not found: {path}")
        with open(path, "rb") as f:
            data = f.read()
        if len(data) < 8 or data[:4] != b"FORM":
            raise DataWinError(f"not a GameMaker data.win: {path}")
        return cls(path, data)

    def u8(self, off: int) -> int:
        return self.data[off]

    def u16(self, off: int) -> int:
        return struct.unpack_from("<H", self.data, off)[0]

    def u32(self, off: int) -> int:
        return struct.unpack_from("<I", self.data, off)[0]

    def i32(self, off: int) -> int:
        return struct.unpack_from("<i", self.data, off)[0]

    def f32(self, off: int) -> float:
        return struct.unpack_from("<f", self.data, off)[0]

    def string(self, ptr: int) -> str | None:
        if ptr == 0:
            return None
        n = self.u32(ptr - 4)
        return bytes(self.mv[ptr:ptr + n]).decode("utf-8", "replace")

    def pointer_list(self, off: int) -> list[int]:
        n = self.u32(off)
        return list(struct.unpack_from(f"<{n}I", self.data, off + 4))

    def chunk_list(self, name: str) -> list[int]:
        if name not in self.chunks:
            raise DataWinError(f"chunk {name} missing in {self.path}")
        return self.pointer_list(self.chunks[name][0])

    def gen8(self) -> dict:
        o = self.chunks["GEN8"][0]
        return {
            "bytecode": self.u8(o + 1),
            "filename": self.string(self.u32(o + 4)),
            "name": self.string(self.u32(o + 0x28)),
            "window_w": self.u32(o + 0x3C),
            "window_h": self.u32(o + 0x40),
        }
