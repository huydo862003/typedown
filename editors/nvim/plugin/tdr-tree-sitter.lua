require("typedown.theme").setup()

local PARSERS = {
  tdr = 'tdr.so',
  tdr_md = 'tdr_md.so',
  tdr_md_inline = 'tdr_md_inline.so',
  tdr_yaml = 'tdr_yaml.so',
}

-- Resolve tree-sitter parser .so paths, downloading them if necessary.
-- Returns a table mapping parser name to .so path, or nil on failure.
local function resolve_tree_sitter_binaries()
  --- Beware of changing folder structure
  local repo_root = vim.fn.fnamemodify(
    debug.getinfo(1, "S").source:sub(2),
    ":h:h:h:h"
  )

  if vim.g.typedown_dev then
    local result = {}
    for name, filename in pairs(PARSERS) do
      result[name] = repo_root .. "/packages/tree-sitter/dist/tree-sitter-so/" .. filename
    end
    return result
  end

  local version = require("typedown.version")

  local tag
  if version:find("-") then
    tag = "staging/v" .. version
  else
    tag = "v" .. version
  end

  local uname = vim.uv.os_uname()
  local platform_dir
  if uname.sysname == "Linux" and uname.machine == "x86_64" then
    platform_dir = "tree-sitter-so-linux-x64"
  elseif uname.sysname == "Darwin" and uname.machine == "arm64" then
    platform_dir = "tree-sitter-so-darwin-arm64"
  elseif uname.sysname == "Darwin" and uname.machine == "x86_64" then
    platform_dir = "tree-sitter-so-darwin-x64"
  elseif uname.sysname:find("Windows") then
    platform_dir = "tree-sitter-so-win32-x64"
  else
    vim.notify("[typedown] Unsupported platform: " .. uname.sysname .. " " .. uname.machine, vim.log.levels.ERROR)
    return nil
  end

  local cache_dir = vim.fn.stdpath("data") .. "/typedown/" .. version
  local binary_dir = cache_dir .. "/" .. platform_dir
  local base_url = "https://github.com/huydo862003/typedown/releases/download/" .. tag .. "/" .. platform_dir

  -- Return cached binaries if already downloaded
  if vim.uv.fs_stat(binary_dir) then
    local result = {}
    for name, filename in pairs(PARSERS) do
      result[name] = binary_dir .. "/" .. filename
    end
    return result
  end

  vim.fn.mkdir(binary_dir, "p")

  vim.notify("[typedown] Downloading TDR tree-sitter " .. version .. "...", vim.log.levels.INFO)

  local result = {}
  for name, filename in pairs(PARSERS) do
    local dest = binary_dir .. "/" .. filename
    local url = base_url .. "/" .. filename
    local outcome = vim.system({ "curl", "-fsSL", "-o", dest, url }):wait()
    if outcome.code ~= 0 then
      vim.fn.delete(binary_dir, 'rf')
      vim.notify("[typedown] Download failed: " .. (outcome.stderr or ""), vim.log.levels.ERROR)
      return nil
    end
    result[name] = dest
  end

  return result
end

local binaries = resolve_tree_sitter_binaries()

if binaries then
  for parser_name, binary_path in pairs(binaries) do
    vim.treesitter.language.add(parser_name, { path = binary_path })
  end

  vim.treesitter.language.register('tdr', 'tdr')
end
