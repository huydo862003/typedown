# Typedown for Neovim

LSP support for the Typedown language (`.tdr` files), with automatic server binary download.

## Requirements

- Neovim 0.10+
- `curl` or `wget` on your PATH

## Installation

### lazy.nvim

```lua
{
  "huydo862003/typedown",
  dir = "editors/nvim",  -- if using the monorepo directly
  -- or for standalone:
  -- "huydo862003/typedown-nvim",
  config = function()
    require("typedown").setup()
  end,
}
```

### mini.deps

```lua
MiniDeps.add("huydo862003/typedown-nvim")
require("typedown").setup()
```

### packer.nvim

```lua
use {
  "huydo862003/typedown-nvim",
  config = function()
    require("typedown").setup()
  end,
}
```

### Vanilla (no plugin manager)

Clone the plugin into your Neovim packages directory:

```bash
git clone https://github.com/huydo862003/typedown-nvim \
  ~/.local/share/nvim/site/pack/typedown/start/typedown-nvim
```

Then add to your `init.lua`:

```lua
require("typedown").setup()
```

## Configuration

`setup()` accepts an optional table:

```lua
require("typedown").setup({
  -- Override the binary path (disables auto-download)
  -- cmd = { "/path/to/tdr-lsp" },

  -- Extra capabilities to pass to the LSP
  -- capabilities = vim.lsp.protocol.make_client_capabilities(),
})
```

## How It Works

On first use, the plugin detects your OS and architecture, downloads the matching
`tdr-lsp` binary from the GitHub release that matches the plugin version, and
caches it in `~/.local/share/nvim/typedown/`. The binary is re-downloaded only when
the plugin version changes.
