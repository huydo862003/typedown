## [0.1.6] - 2026-08-02

* packages/typerighter
  - No longer inject html

## [0.1.5] - 2026-08-02

* packages/typerighter
  - Split cli to a vite endtrypoint and properly import it

## [0.1.4] - 2026-08-02

* packages/rpc-server
  - Support unref() to avoid keeping the event loop alive

* packages/typerighter
  - Support CLI
  - Prevent building from keeping the event loop alive by unref() the server
  - Use random port for RPC server

* crates/typedown-lang
  - Support exporting assets to markdown

## [0.1.3] - 2026-08-02

* packages/rpc-server
  - Prioritize binary on PATH

## [0.1.2] - 2026-08-02

* packages/rpc-server
  - Fall back to system PATH when typedown-rpc binary is not in node_modules

* editors/nvim
  - Add list[T] and dict[K,V] syntax highlighting for index expressions
  - Fix devicons icon key (use file extension 'td' instead of filetype 'typedown')

* editors/zed
  - Add list[T] and dict[K,V] syntax highlighting for index expressions

* editors/vscode
  - Add list[T] and dict[K,V] syntax highlighting via tmLanguage pattern

## [0.1.1] - 2026-08-01

* crates/\*, packages/\*, editors/\*
  - Report unresolved identifiers as errors when used as field values
  - Skip diagnostics for non-td files (fixes false errors on typedown.yaml)
  - Add Nix flake build for typedown-lsp and typedown-rpc
  - Fix repo URLs in npm packages

## [0.1.0] - 2026-08-01

First major version with core compiler + language services and static site generator

