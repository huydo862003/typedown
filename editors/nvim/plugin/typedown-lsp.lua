-- Resolve the LSP binary once at plugin load, downloading it if necessary.
-- Returns the binary path, or nil if it could not be resolved.
local function resolve_lsp_binary()
  -- Local dev: use the debug build when explicitly requested via local_init.lua.

  --- Beware of changing folder structure
  local repo_root = vim.fn.fnamemodify(
    debug.getinfo(1, "S").source:sub(2),
    ":h:h:h:h"
  )

  if vim.g.typedown_dev then
    return repo_root .. "/target/debug/typedown-lsp"
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
  local artifact
  if uname.sysname == "Linux" and uname.machine == "x86_64" then
    artifact = "typedown-lsp-linux-x86_64"
  elseif uname.sysname == "Darwin" and uname.machine == "arm64" then
    artifact = "typedown-lsp-macos-aarch64"
  elseif uname.sysname == "Darwin" and uname.machine == "x86_64" then
    artifact = "typedown-lsp-macos-x86_64"
  elseif uname.sysname:find("Windows") then
    artifact = "typedown-lsp-windows-x86_64.exe"
  else
    vim.notify("[typedown] Unsupported platform: " .. uname.sysname .. " " .. uname.machine, vim.log.levels.ERROR)
    return nil
  end

  local cache_dir = vim.fn.stdpath("data") .. "/typedown/" .. version
  local binary = cache_dir .. "/" .. artifact
  local url = "https://github.com/huydo862003/typedown/releases/download/" .. tag .. "/" .. artifact

  if vim.uv.fs_stat(binary) then
    return binary
  end

  vim.fn.mkdir(cache_dir, "p")
  vim.notify("[typedown] Downloading typedown-lsp " .. version .. "...", vim.log.levels.INFO)

  local result = vim.system({ "curl", "-fsSL", "-o", binary, url }):wait()
  if result.code ~= 0 then
    vim.notify("[typedown] Download failed: " .. (result.stderr or ""), vim.log.levels.ERROR)
    return nil
  end

  vim.uv.fs_chmod(binary, 493) -- 0755
  return binary
end

local binary = resolve_lsp_binary()

local function start_lsp()
  if not binary then return end
  -- The server resolves the project root per-file via multiproject,
  -- so root_dir just needs to be a valid directory for the client.
  local root = vim.fs.root(0, { "typedown.yaml", "typedown.yml" })
      or vim.fn.fnamemodify(vim.api.nvim_buf_get_name(0), ":h")
  vim.lsp.start({
    name = "typedown-lsp",
    cmd = { binary },
    root_dir = root,
    capabilities = vim.lsp.protocol.make_client_capabilities(),
  })
end

vim.api.nvim_create_autocmd("FileType", {
  pattern = "typedown",
  callback = start_lsp,
})

vim.api.nvim_create_autocmd("BufEnter", {
  pattern = { "typedown.yaml", "typedown.yml" },
  callback = start_lsp,
})
