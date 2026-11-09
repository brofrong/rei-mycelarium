-- R27: gpu_get_tex_filter answers the state the last gpu_set_tex_filter left (the decompiled
-- repeller_clouds_draw / cave_perk_icons_draw save and restore it), not always true.
local tf = require("gm.texfilter")

return {
  tracks_the_last_set = function()
    tf.reset()
    assert(tf.get() == true, "starts linear, as LÖVE's default filter")
    tf.set(false)
    assert(tf.get() == false)
    local saved = tf.get()
    tf.set(true)
    tf.set(saved)
    assert(tf.get() == false, "a save/restore pair leaves the state as it was")
    tf.set(1)
    assert(tf.get() == true, "GML truthiness")
    tf.set(0)
    assert(tf.get() == false)
  end,
}
