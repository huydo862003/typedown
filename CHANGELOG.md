## [0.1.2] - 2026-08-02

* rpc-server
  - Fall back to system PATH when typedown-rpc binary is not in node_modules

* editors/nvim
  - Add list[T] and dict[K,V] syntax highlighting for index expressions
  - Fix devicons icon key (use file extension 'td' instead of filetype 'typedown')

* editors/zed
  - Add list[T] and dict[K,V] syntax highlighting for index expressions

* editors/vscode
  - Add list[T] and dict[K,V] syntax highlighting via tmLanguage pattern


## [0.1.1] - 2026-08-01

- Report unresolved identifiers as errors when used as field values
- Skip diagnostics for non-td files
  (fixes false errors on typedown.yaml)
- Add Nix flake build for typedown-lsp and typedown-rpc
- Fix repo URLs in npm packages

## [0.1.0] - 2026-08-01

First major version with core compiler + language services and static site generator

