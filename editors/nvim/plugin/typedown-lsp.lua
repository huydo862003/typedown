-- Beware of changing folder structure
local repo_root = vim.fn.fnamemodify(
  debug.getinfo(1, "S") -- Get the source file containing the current executing function
  .source:sub(2),       -- Strip @
  ":h:h:h:h"            -- Walk up parent -> parent -> parent. This reach up to '/lsp-clients/nvim/plugin/typedown-lsp.lua/../../../..'
)

local lsp_binary = repo_root .. "./target/release/typedown-lsp"

-- Trigger on FileType event change
vim.api.nvim_create_autocmd("FileType", {
  pattern = "typedown",
  callback = function()
    -- Start the LSP with root_dir containing typedown config file
    vim.lsp.start({
      name = "typedown-lsp",
      cmd = { binary = lsp_binary },
      root_dir = vim.fs.root(0, { "typedown.yaml", "typedown.yml" })
    })
  end,
})
