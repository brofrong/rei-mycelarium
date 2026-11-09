-- Converts GameMaker GLSL ES fragment shaders into LÖVE 11 pixel shaders.
local M = {}

function M.fragment(src)
  local out = src
  out = out:gsub("varying%s+vec2%s+v_vTexcoord%s*;", "")
  out = out:gsub("varying%s+vec4%s+v_vColour%s*;", "")
  out = out:gsub("uniform%s+sampler2D%s+gm_BaseTexture%s*;", "")
  out = out:gsub("void%s+main%s*%(%s*%)", "void effect()")
  out = out:gsub("gl_FragColor", "love_PixelColor")
  local header = table.concat({
    "#define v_vTexcoord VaryingTexCoord.xy",
    "#define v_vColour VaryingColor",
    "#define gm_BaseTexture MainTex",
    "",
  }, "\n")
  return header .. out
end

return M
