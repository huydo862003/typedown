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

  -- Modifiers: font style only
  vim.api.nvim_set_hl(0, "@lsp.mod.bold.typedown", { bold = true })
  vim.api.nvim_set_hl(0, "@lsp.mod.italic.typedown", { italic = true })
  vim.api.nvim_set_hl(0, "@lsp.mod.strikethrough.typedown", { strikethrough = true })
end

return M
