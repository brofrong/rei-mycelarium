-- Text layout with GameMaker rules (no love dependency).
local M = {}

-- Split on "\n", then wrap at spaces so each line fits max_w (max_w < 0: no wrapping).
function M.wrap(s, max_w, width_of)
  local out = {}
  for para in (s .. "\n"):gmatch("(.-)\n") do
    if max_w < 0 or width_of(para) <= max_w then
      out[#out + 1] = para
    else
      local line = ""
      for word in para:gmatch("%S+") do
        local candidate = line == "" and word or (line .. " " .. word)
        if line ~= "" and width_of(candidate) > max_w then
          out[#out + 1] = line
          line = word
        else
          line = candidate
        end
      end
      out[#out + 1] = line
    end
  end
  return out
end

-- halign: 0 left, 1 center, 2 right; valign: 0 top, 1 middle, 2 bottom
function M.align_x(halign, w) return halign == 1 and -w / 2 or halign == 2 and -w or 0 end
function M.align_y(valign, h) return valign == 1 and -h / 2 or valign == 2 and -h or 0 end

return M
