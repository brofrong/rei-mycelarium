-- Objects, instances, event dispatch and rooms with GameMaker semantics.
-- Event handlers are fn(self, other); event names follow the YYC suffixes
-- (Create_0, Step_1, Draw_64, CleanUp_0, Other_3, ...).
local M = {}

M.noone = -4
M.all = setmetatable({}, { __tostring = function() return "all" end })

local INSTANCE = { __name = "instance" }
M.INSTANCE = INSTANCE

local objects, rooms, instances, next_id
local current_event = {} -- stack of {object, event} for event_inherited
local pending_room

function M.reset()
  objects, rooms, instances, next_id = {}, {}, {}, 100000
  pending_room = nil
  M.global = {}
  M.room, M.room_width, M.room_height = nil, 0, 0
  M.room_def = nil
  M.on_game_end = nil
  M.on_room_start = nil
end
M.reset()

local function resolve_object(o)
  if type(o) == "string" then return objects[o] end
  return o
end

-- def: { parent = <object or name>, <EventName> = fn(self, other), ... }
function M.object(name, def)
  local obj = { name = name, events = {}, parent = def.parent, def = def }
  for k, v in pairs(def) do
    if type(v) == "function" then obj.events[k] = v end
  end
  objects[name] = obj
  return obj
end

function M.object_get(name) return objects[name] end

local function parent_of(obj)
  local p = obj.parent
  if p == nil then return nil end
  p = resolve_object(p)
  obj.parent = p
  return p
end

function M.object_is_ancestor(obj, ancestor)
  obj, ancestor = resolve_object(obj), resolve_object(ancestor)
  local p = parent_of(obj)
  while p do
    if p == ancestor then return true end
    p = parent_of(p)
  end
  return false
end

local function is_a(inst, obj)
  local o = inst.object_index
  while o do
    if o == obj then return true end
    o = parent_of(o)
  end
  return false
end

local function find_event(obj, ev)
  while obj do
    local fn = obj.events[ev]
    if fn then return fn, obj end
    obj = parent_of(obj)
  end
end

local function run_event_from(inst, obj, ev, other)
  local fn, owner = find_event(obj, ev)
  if not fn then return false end
  current_event[#current_event + 1] = { owner, ev }
  fn(inst, other)
  current_event[#current_event] = nil
  return true
end

function M.event_perform(inst, ev, other)
  if inst.__destroyed and ev ~= "CleanUp_0" then return false end
  return run_event_from(inst, inst.object_index, ev, other)
end

function M.event_inherited(self, other)
  local top = current_event[#current_event]
  if not top then return end
  local parent = parent_of(top[1])
  if parent then run_event_from(self, parent, top[2], other) end
end

function M.has_event(inst, ev) return find_event(inst.object_index, ev) ~= nil end

local function new_instance(x, y, depth, obj)
  local inst = setmetatable({
    id = next_id, object_index = obj, x = x, y = y, xstart = x, ystart = y, xprevious = x, yprevious = y,
    depth = depth or 0, visible = obj.def.visible ~= false, persistent = obj.def.persistent or false,
    sprite_index = obj.def.sprite or -1, mask_index = -1,
    image_index = 0, image_speed = 1, image_xscale = 1, image_yscale = 1, image_angle = 0,
    image_blend = 0xFFFFFF, image_alpha = 1,
    speed = 0, direction = 0, hspeed = 0, vspeed = 0, alarm = {},
  }, INSTANCE)
  next_id = next_id + 1
  instances[#instances + 1] = inst
  return inst
end

function M.instance_create_depth(x, y, depth, obj, vars)
  obj = resolve_object(obj)
  local inst = new_instance(x, y, depth, obj)
  if vars then for k, v in pairs(vars) do inst[k] = v end end
  M.event_perform(inst, "Create_0")
  return inst
end

function M.instance_create_layer(x, y, layer, obj, vars)
  local depth = 0
  if M.layer_depth_of then depth = M.layer_depth_of(layer) or 0 end
  return M.instance_create_depth(x, y, depth, obj, vars)
end

local function live(inst) return not inst.__destroyed end

function M.instance_destroy(inst, run_destroy)
  if inst == nil then return end
  if getmetatable(inst) ~= INSTANCE then
    for _, i in ipairs(M.instances_of(inst)) do M.instance_destroy(i, run_destroy) end
    return
  end
  if inst.__destroyed then return end
  if run_destroy ~= false then M.event_perform(inst, "Destroy_0") end
  inst.__destroyed = true
  M.event_perform(inst, "CleanUp_0")
end

-- target: instance, object (incl. children), all
function M.instances_of(target)
  local out = {}
  target = resolve_object(target)
  if target == nil or target == M.noone then return out end
  if getmetatable(target) == INSTANCE then
    if live(target) then out[1] = target end
    return out
  end
  for _, inst in ipairs(instances) do
    if live(inst) and (target == M.all or is_a(inst, target)) then out[#out + 1] = inst end
  end
  return out
end

function M.instance_exists(target) return #M.instances_of(target) > 0 end
function M.instance_number(obj) return #M.instances_of(obj) end
function M.instance_find(obj, n)
  local list = M.instances_of(obj)
  return list[(n or 0) + 1] or M.noone
end

-- with (target) { ... } -> gm.with(self, target, function(self, other) ... end)
function M.with(caller, target, fn)
  for _, inst in ipairs(M.instances_of(target)) do
    if live(inst) then fn(inst, caller) end
  end
end

local function compact()
  local j = 0
  for i = 1, #instances do
    local inst = instances[i]
    instances[i] = nil
    if live(inst) then
      j = j + 1
      instances[j] = inst
    end
  end
end

local function each(ev)
  local list = { unpack(instances) }
  for _, inst in ipairs(list) do
    if live(inst) then M.event_perform(inst, ev) end
  end
end

-- One game step: Begin Step, Step, End Step, then sprite animation and room changes.
function M.step()
  for _, inst in ipairs(instances) do inst.xprevious, inst.yprevious = inst.x, inst.y end
  each("Step_1")
  each("Step_0")
  each("Step_2")
  if M.animate then M.animate(instances) end
  compact()
  if pending_room then
    local r = pending_room
    pending_room = nil
    M.apply_room(r)
  end
end

local function by_depth()
  local list = {}
  for _, inst in ipairs(instances) do
    if live(inst) then list[#list + 1] = inst end
  end
  -- higher depth draws first; stable by creation order
  local order = {}
  for i, inst in ipairs(list) do order[inst] = i end
  table.sort(list, function(a, b)
    if a.depth ~= b.depth then return a.depth > b.depth end
    return order[a] < order[b]
  end)
  return list
end

-- phase(name) is called before each drawing phase so the host can switch targets:
-- "pre", "world", "post", "gui".
function M.draw(phase)
  phase = phase or function() end
  local function run(ev, default_draw)
    for _, inst in ipairs(by_depth()) do
      if inst.visible and live(inst) then
        if not M.event_perform(inst, ev) and default_draw and M.draw_self then M.draw_self(inst) end
      end
    end
  end
  phase("pre")
  run("Draw_76")
  phase("world")
  run("Draw_72")
  run("Draw_0", true)
  run("Draw_73")
  phase("post")
  run("Draw_77")
  phase("gui")
  run("Draw_74")
  run("Draw_64")
  run("Draw_75")
end

-- Rooms -----------------------------------------------------------------------
function M.define_room(name, def) rooms[name] = def end

function M.room_goto(name) pending_room = name end
function M.room_restart() pending_room = M.room end

function M.apply_room(name)
  local def = assert(rooms[name], "unknown room " .. tostring(name))
  local first = M.room == nil
  for _, inst in ipairs(instances) do
    if live(inst) then M.event_perform(inst, "Other_5") end -- Room End
  end
  for _, inst in ipairs(instances) do
    if live(inst) and not inst.persistent then
      inst.__destroyed = true
      M.event_perform(inst, "CleanUp_0")
    end
  end
  compact()
  M.room, M.room_def, M.room_width, M.room_height = name, def, def.w, def.h
  local layer_depth = {}
  for _, l in ipairs(def.layers or {}) do
    for _, id in ipairs(l.instances or {}) do layer_depth[id] = l.depth end
  end
  local created = {}
  for _, spec in ipairs(def.instances or {}) do
    local obj = assert(objects[spec.object], "unknown object " .. tostring(spec.object))
    local inst = new_instance(spec.x, spec.y, layer_depth[spec.id] or 0, obj)
    inst.image_xscale, inst.image_yscale = spec.scale_x or 1, spec.scale_y or 1
    inst.image_speed, inst.image_index = spec.image_speed or 1, spec.image_index or 0
    inst.image_angle = spec.angle or 0
    created[#created + 1] = inst
  end
  for _, inst in ipairs(created) do M.event_perform(inst, "Create_0") end
  if first then
    for _, inst in ipairs(created) do M.event_perform(inst, "Other_2") end -- Game Start
  end
  for _, inst in ipairs(instances) do
    if live(inst) then M.event_perform(inst, "Other_4") end -- Room Start
  end
  if M.on_room_start then M.on_room_start(name, def) end
end

function M.game_end()
  each("Other_3")
  for _, inst in ipairs(instances) do
    if live(inst) then
      inst.__destroyed = true
      M.event_perform(inst, "CleanUp_0")
    end
  end
  compact()
  if M.on_game_end then M.on_game_end() end
end

function M.all_instances() return instances end

return M
