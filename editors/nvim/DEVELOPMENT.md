# Development

## Testing

To test without affecting your regular Neovim config:

```bash
cargo build
nvim -u editors/nvim/test_init.lua
```

Then run `:LspInfo` inside Neovim to verify the server attached.
