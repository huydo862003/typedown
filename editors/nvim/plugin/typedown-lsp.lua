-- Beware of changing folder structure
local repo_root = vim.fn.fnamemodify(
  debug.getinfo(1, "S") -- Get the source file containing the current executing function
  .source:sub(2),       -- Strip @
  ":h:h:h:h"            -- Walk up parent -> parent -> parent. This reach up to '/lsp-clients/nvim/plugin/typedown-lsp.lua/../../../..'
)

local lsp_binary = repo_root .. "/target/release/typedown-lsp"

local function start_lsp()
  -- Start the LSP with root_dir containing typedown config file
  vim.lsp.start({
    name = "typedown-lsp",
    cmd = { lsp_binary },
    root_dir = vim.fs.root(0, { "typedown.yaml", "typedown.yml" }),
  })
end

-- Trigger on FileType event change
vim.api.nvim_create_autocmd("FileType", {
  pattern = "typedown",
  callback = start_lsp,
})

-- Also attach to typedown.yaml / typedown.yml so diagnostics show there too.
vim.api.nvim_create_autocmd("BufEnter", {
  pattern = { "typedown.yaml", "typedown.yml" },
  callback = start_lsp,
})
