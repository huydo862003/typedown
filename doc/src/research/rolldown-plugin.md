# Rolldown Plugin

References:

- [rolldown.rs](https://rolldown.rs/apis/plugin-api)
- [Plugin compatibility tracking](https://github.com/rolldown/rolldown/issues/819).

## Overview

Rolldown's plugin interface is almost fully compatible with Rollup's. If you've written a Rollup plugin before, you already know how to write one for Rolldown.

A plugin is just an object that satisfies a specific interface. Typically you distribute it as a package that exports a factory function: The function takes plugin-specific options, returns the plugin object.

What can plugins do? Customize Rolldown's behavior: Transpile code before bundling, shim built-in modules, inject virtual modules, etc.
