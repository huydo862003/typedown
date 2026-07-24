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
