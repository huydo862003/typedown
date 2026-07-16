require("typedown.theme").setup()

-- Resolve the LSP binary once at plugin load, downloading it if necessary.
-- Returns the binary path, or nil if it could not be resolved.
local function resolve_tree_sitter_binaries()
  -- Local dev: use the debug build when explicitly requested via local_init.lua.
  --- Beware of changing folder structure
  local repo_root = vim.fn.fnamemodify(
    debug.getinfo(1, "S").source:sub(2),
    ":h:h:h:h"
  )

  if vim.g.typedown_dev then
    return {
      typedown = repo_root .. "/packages/tree-sitter/parser.so",
      typedown_md = repo_root .. "/packages/tree-sitter/typedown-md/parser.so",
      typedown_md_inline = repo_root .. "/packages/tree-sitter/typedown-md-inline/parser.so",
      typedown_yaml = repo_root .. "/packages/tree-sitter/typedown-yaml/parser.so",
    }
  end

  local version = require("typedown.version")

  -- Pre-release versions (e.g. "0.1.1-rc.0") use staging tags.
  local tag
  if version:find("-") then
    tag = "staging/v" .. version
  else
    tag = "v" .. version
  end

  local uname = vim.uv.os_uname()
  local root_artifact_dir
  if uname.sysname == "Linux" and uname.machine == "x86_64" then
    root_artifact_dir = "tree-sitter-so-linux-x64"
  elseif uname.sysname == "Darwin" and uname.machine == "arm64" then
    root_artifact_dir = "tree-sitter-so-darwin-arm64"
  elseif uname.sysname == "Darwin" and uname.machine == "x86_64" then
    root_artifact_dir = "tree-sitter-so-darwin-x64"
  elseif uname.sysname:find("Windows") then
    root_artifact_dir = "tree-sitter-so-win32-x64"
  else
    vim.notify("[typedown] Unsupported platform: " .. uname.sysname .. " " .. uname.machine, vim.log.levels.ERROR)
    return nil
  end

  local cache_dir = vim.fn.stdpath("data") .. "/typedown/" .. version

  local root_binary_dir = cache_dir .. "/" .. root_artifact_dir

  local root_url = "https://github.com/huydo862003/typedown/releases/download/" .. tag .. "/" .. root_artifact_dir

  if vim.uv.fs_stat(root_binary_dir) then
    return nil
  end

  vim.fn.mkdir(cache_dir, "p")

  vim.fn.mkdir(root_binary_dir)

  local tree_sitter_binaries = {}

  for filetype, binary_name in pairs({
    typedown = 'parser.so',
    typedown_md = 'typedown-md/parser.so',
    typedown_md_inline = 'typedown-md-inline/parser.so',
    typedown_yaml = 'typedown-yaml/parser.so',
  }) do
    vim.notify("[typedown] Downloading typedown tree-sitter " .. version .. "...", vim.log.levels.INFO)

    local result = vim.system({ "curl", "-fsSL", "-o", root_binary_dir .. binary_name, root_url .. binary_name }):wait()
    if result.code ~= 0 then
      vim.fn.delete(root_binary_dir, 'rf')
      vim.notify("[typedown] Download failed: " .. (result.stderr or ""), vim.log.levels.ERROR)
      return nil
    end

    tree_sitter_binaries[filetype] = root_binary_dir .. binary_name
  end

  return tree_sitter_binaries
end

local binaries = resolve_tree_sitter_binaries()

if binaries then
  for filetype, binary_path in pairs(binaries) do
    vim.treesitter.language.add(filetype, { path = binary_path })
  end

  vim.treesitter.language.register('typedown', { 'tdr', 'typedown' })
end
