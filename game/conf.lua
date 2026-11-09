function love.conf(t)
  -- MYCO_IDENTITY=<name> redirects the save folder (automated runs must not touch real slots)
  t.identity = os.getenv("MYCO_IDENTITY") or "Mycelarium"
  t.version = "11.5"
  t.window.title = "Mycelarium"
  t.window.width = 1280
  t.window.height = 720
  t.window.resizable = true
  t.window.minwidth = 640
  t.window.minheight = 360
  t.window.highdpi = true
  t.window.usedpiscale = false
  t.window.vsync = 1
  t.modules.joystick = false
  t.modules.physics = false
end
