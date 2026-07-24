# Rolldown Plugin

References:

- [rolldown.rs](https://rolldown.rs/apis/plugin-api)
- [Plugin compatibility tracking](https://github.com/rolldown/rolldown/issues/819).

## Overview

Plugins allow customizing Rolldown's behavior. Some use cases:

1. Transpile code before bundling.
2. Shim built-in modules.
3. Inject virtual modules.

Rolldown's plugin interface is almost fully compatible with Rollup's. (For context, Rolldown is a rust migration of Rollup, if I recall correctly)

By definition:

1. A plugin is just an object that satisfies the specific plugin interface of Rolldown.
2. Typically it is distributed as a package that exports a factory function: The function takes plugin-specific options, returns the plugin object.

> Remark: I have seen plugins registered like this

```json
{
  plugins: [
    plugin(options) // `plugin` is the factory function that creates a plugin object
  ]
}
```

See an example here: https://rolldown.rs/apis/plugin-api#example (there's a notice about using **hook filters** where possible). Essentially:

1. The plugin package exports a plugin factory.
2. The plugin factory returns a plugin object.

## Conventions

1.  Naming: Plugin names should be prefixed with `rolldown-plugin-`.
2.  `package.json` keywords: Include `rolldown-plugin`.
3.  Source mappings should be correctly output.
4.  Virtual modules have their own conventions (see below).

    4.1. User-facing ID should be prefixed with `virtual:`.

         Example: `virtual:example`, `virtual:posts/helpers`.

    4.2. Use the plugin name as a namespace to avoid collisions.

         Example: `rolldown-plugin-posts` uses `virtual:posts`.

    4.3. Prefix the resolved ID with `\0` (null byte).

         -> This tells other plugins and Rolldown itself "this is virtual, don't try to resolve it on disk".S

         -> Sourcemaps also use this to distinguish virtual modules from real files.

    > Note:
    >
    > - Modules derived from a real file (like submodules from `.vue` or `.svelte` SFCs) should NOT use the `\0` prefix.
    > - Using it would break sourcemaps, since those submodules can be mapped back to the actual file on disk.
