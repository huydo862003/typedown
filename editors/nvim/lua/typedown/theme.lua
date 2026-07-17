-- Semantic token highlight groups for typedown files.
-- Only TYPE is provided by the LSP; all other highlighting comes from syntactic grammars.

local M = {}

function M.setup()
  vim.api.nvim_set_hl(0, "@lsp.type.type.tdr", { link = "Type" })

  -- Default highlights for tree-sitter captures
  -- Only apply if not already set by the user's colorscheme

  -- key: value
  vim.api.nvim_set_hl(0, "@property.tdr_yaml", { link = "Label", default = true })
  vim.api.nvim_set_hl(0, "@property.tdr_md_inline", { link = "Label", default = true })

  -- ${...}
  vim.api.nvim_set_hl(0, "@punctuation.special.tdr_yaml", { link = "Special", default = true })
  vim.api.nvim_set_hl(0, "@punctuation.special.tdr_md_inline", { link = "Special", default = true })

  -- # heading
  vim.api.nvim_set_hl(0, "@markup.heading.tdr_md", { link = "Title", default = true })

  -- **bold** or *italic*
  vim.api.nvim_set_hl(0, "@markup.strong.tdr_md_inline", { bold = true, default = true })
  vim.api.nvim_set_hl(0, "@markup.italic.tdr_md_inline", { italic = true, default = true })

  -- > blockquote
  vim.api.nvim_set_hl(0, "@markup.quote.tdr_md", { link = "Comment", default = true })

  -- `code` or ```code block```
  vim.api.nvim_set_hl(0, "@markup.raw.tdr_md_inline", { link = "String", default = true })
  vim.api.nvim_set_hl(0, "@markup.raw.block.tdr_md", { link = "String", default = true })

  -- $math$
  vim.api.nvim_set_hl(0, "@markup.math.tdr_md_inline", { link = "Special", default = true })

  -- [label](url)
  vim.api.nvim_set_hl(0, "@markup.link.label.tdr_md_inline", { link = "Underlined", default = true })
  vim.api.nvim_set_hl(0, "@markup.link.url.tdr_md_inline", { link = "Underlined", default = true })

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
