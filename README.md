# Typedown

![Status](https://img.shields.io/badge/status-active-brightblue)
![License](https://img.shields.io/badge/license-GPL-green)
<a href="https://github.com/huydo862003/Fck-AI-Slop#plan"><img src="https://img.shields.io/badge/Human%20slop-90EE90"></a>

A typed markdown language for structured content.

## Neovim Plugin

The Neovim LSP client lives in `editors/nvim/`. To test it without affecting your regular Neovim config:

```bash
nvim -u editors/nvim/test_init.lua
```

Then run `:LspInfo` inside Neovim to verify the server attached. Make sure `typedown-lsp` is built first:

```bash
cargo build --release
```

## Common Pitfalls (and Painful Lessons)

These are some lessons learnt during the development of the project. Some comments in the code are also marked with `TIL`.
