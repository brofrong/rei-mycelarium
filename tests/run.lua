-- Minimal test runner: luajit tests/run.lua [pattern]
-- Each tests/test_*.lua returns a table of name -> function.
package.path = "game/?.lua;game/?/init.lua;tests/?.lua;" .. package.path

local pattern = arg[1]
local files = {}
for f in io.popen("ls tests/test_*.lua 2>/dev/null"):lines() do
  if not pattern or f:find(pattern, 1, true) then files[#files + 1] = f end
end
table.sort(files)

local passed, failed = 0, 0
for _, file in ipairs(files) do
  local ok, suite = pcall(dofile, file)
  if not ok then
    failed = failed + 1
    print("ERROR loading " .. file .. ": " .. tostring(suite))
  else
    local names = {}
    for name in pairs(suite) do names[#names + 1] = name end
    table.sort(names)
    for _, name in ipairs(names) do
      local tok, err = xpcall(suite[name], debug.traceback)
      if tok then
        passed = passed + 1
      else
        failed = failed + 1
        print("FAIL " .. file .. " :: " .. name .. "\n" .. tostring(err))
      end
    end
  end
end
print(string.format("%d passed, %d failed", passed, failed))
os.exit(failed == 0 and 0 or 1)
