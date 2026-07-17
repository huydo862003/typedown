-- Load the plugin and auto-download the staging binary matching version.lua.
-- Usage: nvim -u editors/nvim/staging_init.lua path/to/file.tdr
vim.cmd("source " .. vim.fn.stdpath("config") .. "/init.lua")

local plugin_dir = vim.fn.getcwd() .. "/editors/nvim"
vim.opt.runtimepath:append(plugin_dir)

vim.cmd("source " .. plugin_dir .. "/ftdetect/typedown.lua")
vim.cmd("source " .. plugin_dir .. "/plugin/tdr-lsp.lua")
vim.cmd("source " .. plugin_dir .. "/plugin/tdr-tree-sitter.lua")
