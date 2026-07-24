# Vite Plugin

References: [vite.dev/guide/api-plugin](https://vite.dev/guide/api-plugin).

## Overview

Vite's plugin interface is just Rolldown's interface with a few extra hooks on top. So if you can write a Rolldown plugin, you can write a Vite plugin.

The extra hooks are mostly about dev server integration: HMR, configuring the dev server, etc. The core hooks (`resolveId`, `load`, `transform`) are all inherited from Rolldown.

It is recommended to read the [Rolldown plugin page](./rolldown-plugin.md) first, as the fundamentals are defined there.
