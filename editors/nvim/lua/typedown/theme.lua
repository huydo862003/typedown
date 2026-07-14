-- Semantic token highlight groups for typedown files.
-- Only TYPE is provided by the LSP; all other highlighting comes from syntactic grammars.

local M = {}

function M.setup()
  vim.api.nvim_set_hl(0, "@lsp.type.type.typedown", { link = "Type" })

  -- File icon for .tdr files
  local ok_devicons, devicons = pcall(require, "nvim-web-devicons")
  if ok_devicons then
    devicons.set_icon({
      tdr = {
        icon = "",
        color = "#a0522d",
        cterm_color = "130",
        name = "Typedown",
      },
    })
  end

  local ok_mini, mini_icons = pcall(require, "mini.icons")
  if ok_mini and mini_icons.config then
    mini_icons.config.extension = mini_icons.config.extension or {}
    mini_icons.config.extension.tdr = { glyph = "", hl = "MiniIconsGrey" }
  end

end

return M
