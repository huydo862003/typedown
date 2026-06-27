-- test_init is used to load the plugin for testing
-- Load normal user config first so other plugins are available
vim.cmd("source " .. vim.fn.stdpath("config") .. "/init.lua")

local plugin_dir = vim.fn.getcwd() .. "/editors/nvim"
vim.opt.runtimepath:append(plugin_dir)

vim.cmd("source " .. plugin_dir .. "/ftdetect/typedown.lua")
vim.cmd("source " .. plugin_dir .. "/plugin/typedown-lsp.lua")
