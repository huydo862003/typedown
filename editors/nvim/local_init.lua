-- Load the plugin against the local debug build (target/debug/tdr-lsp).
-- Usage: nvim -u editors/nvim/local_init.lua path/to/file.tdr
vim.cmd("source " .. vim.fn.stdpath("config") .. "/init.lua")

local plugin_dir = vim.fn.getcwd() .. "/editors/nvim"
vim.opt.runtimepath:append(plugin_dir)

-- Signal to the plugin to use the local debug binary.
vim.g.typedown_dev = true

vim.cmd("source " .. plugin_dir .. "/ftdetect/typedown.lua")
vim.cmd("source " .. plugin_dir .. "/plugin/tdr-lsp.lua")
vim.cmd("source " .. plugin_dir .. "/plugin/tdr-tree-sitter.lua")
vim.cmd("source " .. plugin_dir .. "/plugin/tdr-paste.lua")
