-- GameMaker buffers on LuaJIT FFI byte arrays.
local ffi = require("ffi")

local M = {
  fixed = 0, grow = 1, wrap = 2, fast = 3,
  u8 = 1, s8 = 2, u16 = 3, s16 = 4, u32 = 5, s32 = 6, f16 = 7, f32 = 8, f64 = 9, bool = 10, string = 11, u64 = 12, text = 13,
  seek_start = 0, seek_relative = 1, seek_end = 2,
}

local CTYPES = { [1] = "uint8_t", [2] = "int8_t", [3] = "uint16_t", [4] = "int16_t", [5] = "uint32_t", [6] = "int32_t",
  [8] = "float", [9] = "double", [10] = "uint8_t", [12] = "uint64_t" }
local SIZES = { [1] = 1, [2] = 1, [3] = 2, [4] = 2, [5] = 4, [6] = 4, [7] = 2, [8] = 4, [9] = 8, [10] = 1, [12] = 8 }

local buffers, next_id = {}, 0

function M.create(size, kind, alignment)
  size = math.max(1, size)
  local b = { id = next_id, data = ffi.new("uint8_t[?]", size), cap = size, size = size, pos = 0, kind = kind, align = alignment or 1 }
  buffers[next_id] = b
  next_id = next_id + 1
  return b
end

function M.get(id) return type(id) == "table" and id or buffers[id] end
function M.exists(id) return M.get(id) ~= nil end
function M.delete(id) local b = M.get(id); if b then buffers[b.id] = nil end end
function M.get_size(id) return M.get(id).size end
function M.tell(id) return M.get(id).pos end

function M.resize(id, n)
  local b = M.get(id)
  if n > b.cap then
    local d = ffi.new("uint8_t[?]", n)
    ffi.copy(d, b.data, b.size)
    b.data, b.cap = d, n
  end
  b.size = n
  if b.pos > n then b.pos = n end
end

local function ensure(b, n)
  if b.pos + n <= b.size then return true end
  if b.kind == M.grow then
    local want = math.max(b.pos + n, b.size * 2)
    M.resize(b, want)
    return true
  end
  return false
end

function M.seek(id, base, offset)
  local b = M.get(id)
  if base == M.seek_start then b.pos = offset
  elseif base == M.seek_relative then b.pos = b.pos + offset
  else b.pos = b.size + offset end
  b.pos = math.max(0, math.min(b.pos, b.size))
end

local function write_at(b, off, t, v)
  if t == M.string or t == M.text then
    local s = tostring(v)
    ffi.copy(b.data + off, s, #s)
    if t == M.string then b.data[off + #s] = 0 end
    return
  end
  if t == M.bool then v = v and v ~= 0 and 1 or 0 end
  ffi.cast(CTYPES[t] .. "*", b.data + off)[0] = v
end

local function read_at(b, off, t)
  if t == M.string or t == M.text then
    local e = off
    while e < b.size and b.data[e] ~= 0 do e = e + 1 end
    return ffi.string(b.data + off, e - off), e - off + (t == M.string and 1 or 0)
  end
  local v = ffi.cast(CTYPES[t] .. "*", b.data + off)[0]
  if t == M.u64 then v = tonumber(v) end
  return v, SIZES[t]
end

local function size_of(t, v)
  if t == M.string then return #tostring(v) + 1 end
  if t == M.text then return #tostring(v) end
  return SIZES[t]
end

function M.write(id, t, v)
  local b = M.get(id)
  local n = size_of(t, v)
  if not ensure(b, n) then return -1 end
  write_at(b, b.pos, t, v)
  b.pos = b.pos + n
  return 0
end

function M.read(id, t)
  local b = M.get(id)
  local v, n = read_at(b, b.pos, t)
  b.pos = b.pos + n
  return v
end

function M.peek(id, off, t) return (read_at(M.get(id), off, t)) end
function M.poke(id, off, t, v)
  local b = M.get(id)
  local n = size_of(t, v)
  if off + n > b.size then
    if b.kind ~= M.grow then return false end
    M.resize(b, off + n)
  end
  write_at(b, off, t, v)
  return true
end

function M.copy(src, src_off, size, dest, dest_off)
  src, dest = M.get(src), M.get(dest)
  if dest_off + size > dest.size then M.resize(dest, dest_off + size) end
  ffi.copy(dest.data + dest_off, src.data + src_off, size)
end

function M.to_string(id) local b = M.get(id); return ffi.string(b.data, b.size) end
function M.from_string(s, kind)
  local b = M.create(#s, kind or M.grow, 1)
  ffi.copy(b.data, s, #s)
  return b
end

return M
