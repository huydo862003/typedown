# Vite Plugin

References: [vite.dev/guide/api-plugin](https://vite.dev/guide/api-plugin).

## Overview

Vite's plugin interface is just Rolldown's interface with a few extra hooks on top. So if you can write a Rolldown plugin, you can write a Vite plugin.

The extra hooks are mostly about dev server integration: HMR, configuring the dev server, etc. The core hooks (`resolveId`, `load`, `transform`) are all inherited from Rolldown.

It is recommended to read the [Rolldown plugin page](./rolldown-plugin.md) first, as the fundamentals are defined there.

## Plugin Config

Users will typically do this to register a plugin to Vite:

1. Add plugins to `devDependencies`.
2. Configure them via the `plugins` array in `vite.config.js`:

```js
import vitePlugin from "vite-plugin-feature";
import rollupPlugin from "rollup-plugin-feature";

export default defineConfig({
  plugins: [vitePlugin(), rollupPlugin()],
});
```

> Remark: Rollup plugins work in Vite too, since Vite's plugin interface is a superset of Rolldown's (which is Rollup-compatible).

> Convention: It is common to author a plugin as a factory function that returns the actual plugin object. The function accepts options, allowing users to customize the plugin's behavior. This is why plugins are called as `vitePlugin()` rather than passed as `vitePlugin`.

### Falsy Plugins

Falsy values in the `plugins` array are silently ignored. This is useful for conditionally enabling plugins:

```js
plugins: [isDev && myDevPlugin()];
```

### Presets

> Definition: Preset is like an opinionated bundle of plugins.

`plugins` also accepts **presets**: A single element that is itself an array of plugins. Vite flattens the array internally, so the user just calls one function. This is the recommended pattern for complex integrations like framework plugins:

```js
// framework-plugin package
export default function framework(config) {
  return [frameworkRefresh(config), frameworkDevtools(config)];
}
```

```js
// vite.config.js
import framework from "vite-plugin-framework";

export default defineConfig({
  plugins: [framework()],
});
```

> Remark: From the user's perspective, this is identical to a single plugin. The fact that it expands to multiple plugins is an implementation detail of the preset.

## Universal Hooks

During dev, the Vite dev server creates a **plugin container** (?) that invokes Rolldown build hooks the same way Rolldown does.

> A plugin container is an internal Vite abstraction that implements the same plugin runner interface as Rolldown.
> Since Vite doesn't bundle during dev (it serves modules individually on demand), the plugin container is what lets Rolldown-compatible plugins still work: it intercepts each module request and runs the relevant hooks (`resolveId`, `load`, `transform`) as if Rolldown were processing the module.

The following hooks are called once on **server start**:

- `options`
- `buildStart`

The following hooks are called on each **incoming module request**:

- `resolveId`
- `load`
- `transform`

> These hooks also have an extended options parameter with additional Vite-specific properties.

> Remark: Regarding `importer`...
>
> - Context, `importer` as in `resolveId(source, importer)`: the file containing the `import` statement being resolved.
> - In Rolldown, this is always known since the full graph is analyzed upfront.
> - In Vite's dev server, it is sometimes unknown (modules are processed on demand), so Vite falls back to the root `index.html` path.
> - For imports going through Vite's resolve pipeline, the real importer is available via import analysis.
>   Takeaways: Plugin authors should not assume `importer` is always accurate during dev.

The following hooks are called when **the server is closed**:

- `buildEnd`
- `closeBundle`

Some notes:

- `moduleParsed` is not called during dev, because Vite avoids full AST parses for better performance.
- Output generation hooks (except `closeBundle`) are not called during dev.

## Vite-Specific Hooks

These hooks are only meaningful to Vite and are ignored by Rollup.

| Hook                     | Kind                                                                                                       | Purpose                                                                                                                       |
| ------------------------ | ---------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `config`                 | [`async`](./rolldown-plugin.md#hook-kind-async), [`sequential`](./rolldown-plugin.md#hook-kind-sequential) | Modify the raw user config before it is resolved. Return a partial config to deep-merge, or mutate directly.                  |
| `configResolved`         | [`async`](./rolldown-plugin.md#hook-kind-async), [`parallel`](./rolldown-plugin.md#hook-kind-parallel)     | Called after config is fully resolved. Use this to read and store the final config for use in other hooks.                    |
| `configureServer`        | [`async`](./rolldown-plugin.md#hook-kind-async), [`sequential`](./rolldown-plugin.md#hook-kind-sequential) | Configure the dev server (e.g. add custom middlewares). Return a function to inject middleware after Vite's internal ones.    |
| `configurePreviewServer` | [`async`](./rolldown-plugin.md#hook-kind-async), [`sequential`](./rolldown-plugin.md#hook-kind-sequential) | Same as `configureServer` but for the preview server (`vite preview`).                                                        |
| `transformIndexHtml`     | [`async`](./rolldown-plugin.md#hook-kind-async), [`sequential`](./rolldown-plugin.md#hook-kind-sequential) | Transform HTML entry files (e.g. `index.html`). Can return a new HTML string, an array of tag descriptors to inject, or both. |
| `handleHotUpdate`        | [`async`](./rolldown-plugin.md#hook-kind-async), [`sequential`](./rolldown-plugin.md#hook-kind-sequential) | Custom HMR update handling. Can filter the affected module list, trigger a full reload, or send custom events to the client.  |
