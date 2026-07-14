-- Semantic token highlight groups for typedown files.
-- Token types:     @lsp.type.<type>.typedown
-- Token modifiers: @lsp.mod.<mod>.typedown (applied as overlays on top)

local M = {}

function M.setup()
  -- Keyword: red
  vim.api.nvim_set_hl(0, "@lsp.type.keyword.typedown", { link = "Keyword" })
  -- Modifier: no color, font style applied via modifier overlays below
  vim.api.nvim_set_hl(0, "@lsp.type.modifier.typedown", { link = "Normal" })
  -- Type: yellow
  vim.api.nvim_set_hl(0, "@lsp.type.type.typedown", { link = "Type" })
  -- Property: blue
  vim.api.nvim_set_hl(0, "@lsp.type.property.typedown", { link = "Identifier" })
  -- Variable: default foreground
  vim.api.nvim_set_hl(0, "@lsp.type.variable.typedown", { link = "Normal" })
  -- String: green italic
  vim.api.nvim_set_hl(0, "@lsp.type.string.typedown", { link = "String" })
  -- Number: purple
  vim.api.nvim_set_hl(0, "@lsp.type.number.typedown", { link = "Number" })
  -- Comment: grey italic
  vim.api.nvim_set_hl(0, "@lsp.type.comment.typedown", { link = "Comment" })
  -- Operator: orange
  vim.api.nvim_set_hl(0, "@lsp.type.operator.typedown", { link = "Operator" })
  -- Heading: green bold
  vim.api.nvim_set_hl(0, "@lsp.type.heading.typedown", { link = "Title" })
  -- Function: aqua (macro-like)
  vim.api.nvim_set_hl(0, "@lsp.type.function.typedown", { link = "Macro" })
  -- Punctuation bracket: matches @punctuation.bracket from the active theme
  vim.api.nvim_set_hl(0, "@lsp.type.punctuation.bracket.typedown", { link = "@punctuation.bracket" })

  -- File icon for .tdr files
  local ok_devicons, devicons = pcall(require, "nvim-web-devicons")
  if ok_devicons then
    devicons.set_icon({
      tdr = {
        icon = "",
        color = "#519aba",
        cterm_color = "74",
        name = "Typedown",
      },
    })
  end

  local ok_mini, mini_icons = pcall(require, "mini.icons")
  if ok_mini and mini_icons.config then
    mini_icons.config.extension = mini_icons.config.extension or {}
    mini_icons.config.extension.tdr = { glyph = "", hl = "MiniIconsBlue" }
  end

  -- Modifiers: font style only
  vim.api.nvim_set_hl(0, "@lsp.mod.bold.typedown", { bold = true })
  vim.api.nvim_set_hl(0, "@lsp.mod.italic.typedown", { italic = true })
  vim.api.nvim_set_hl(0, "@lsp.mod.strikethrough.typedown", { strikethrough = true })
end

return M
