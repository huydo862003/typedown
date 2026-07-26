-- Semantic token highlight groups for typedown files.
-- Only TYPE is provided by the LSP; all other highlighting comes from syntactic grammars.

local M = {}

function M.setup()
  vim.api.nvim_set_hl(0, "@lsp.type.type.td", { link = "Type" })

  -- Default highlights for tree-sitter captures
  -- Only apply if not already set by the user's colorscheme

  -- key: value
  vim.api.nvim_set_hl(0, "@property.typedown_yaml", { link = "Label", default = true })
  vim.api.nvim_set_hl(0, "@property.typedown_md_inline", { link = "Label", default = true })

  -- ${...}
  vim.api.nvim_set_hl(0, "@punctuation.special.typedown_yaml", { link = "Special", default = true })
  vim.api.nvim_set_hl(0, "@punctuation.special.typedown_md_inline", { link = "Special", default = true })

  -- # heading
  vim.api.nvim_set_hl(0, "@markup.heading.typedown_md", { link = "Title", default = true })

  -- **bold** or *italic*
  vim.api.nvim_set_hl(0, "@markup.strong.typedown_md_inline", { bold = true, default = true })
  vim.api.nvim_set_hl(0, "@markup.italic.typedown_md_inline", { italic = true, default = true })

  -- > blockquote
  vim.api.nvim_set_hl(0, "@markup.quote.typedown_md", { link = "Comment", default = true })

  -- `code` or ```code block```
  vim.api.nvim_set_hl(0, "@markup.raw.typedown_md_inline", { link = "String", default = true })
  vim.api.nvim_set_hl(0, "@markup.raw.block.typedown_md", { link = "String", default = true })

  -- $math$
  vim.api.nvim_set_hl(0, "@markup.math.typedown_md_inline", { link = "Special", default = true })

  -- [label](url)
  vim.api.nvim_set_hl(0, "@markup.link.label.typedown_md_inline", { link = "Underlined", default = true })
  vim.api.nvim_set_hl(0, "@markup.link.url.typedown_md_inline", { link = "Underlined", default = true })

  -- File icon for .td files
  local ok_devicons, devicons = pcall(require, "nvim-web-devicons")
  if ok_devicons then
    devicons.set_icon({
      typedown = {
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
    mini_icons.config.extension.td = { glyph = "", hl = "MiniIconsGrey" }
  end

end

return M
