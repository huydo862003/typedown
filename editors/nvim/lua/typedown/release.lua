-- Shared release/download helpers for typedown nvim plugins.

local M = {}

--- Returns the repository root by walking up from the calling plugin file.
--- Beware of changing folder structure.
function M.repo_root()
  -- caller is 2 levels up: release.lua -> plugin/*.lua
  local source = debug.getinfo(2, "S").source:sub(2)
  return vim.fn.fnamemodify(source, ":h:h:h:h")
end

--- Returns the GitHub release tag for the current version.
--- Pre-release versions (e.g. "0.1.1-rc.0") use staging tags.
function M.release_tag()
  local version = require("typedown.version")
  if version:find("-") then
    return "staging/v" .. version, version
  else
    return "v" .. version, version
  end
end

--- Returns the os-arch string matching the release artifact convention,
--- or nil + error message on unsupported platforms.
--- Examples: "linux-x86_64", "darwin-aarch64", "windows-x86_64"
function M.os_arch()
  local uname = vim.uv.os_uname()
  if uname.sysname == "Linux" and uname.machine == "x86_64" then
    return "linux-x86_64"
  elseif uname.sysname == "Darwin" and uname.machine == "arm64" then
    return "darwin-aarch64"
  elseif uname.sysname == "Darwin" and uname.machine == "x86_64" then
    return "darwin-x86_64"
  elseif uname.sysname:find("Windows") then
    return "windows-x86_64"
  else
    return nil, uname.sysname .. " " .. uname.machine
  end
end

--- Returns the cache directory for a given version.
function M.cache_dir(version)
  return vim.fn.stdpath("data") .. "/typedown/" .. version
end

--- Returns the base URL for downloading release artifacts.
function M.release_base_url(tag)
  return "https://github.com/huydo862003/typedown/releases/download/" .. tag
end

--- Downloads a file via curl. Returns true on success, false + stderr on failure.
function M.download(url, dest)
  local result = vim.system({ "curl", "-fsSL", "-o", dest, url }):wait()
  if result.code ~= 0 then
    return false, result.stderr or ""
  end
  return true
end

return M
