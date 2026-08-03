import {
  createRequire,
} from 'node:module';
import type {
  AliasOptions,
} from 'vite';

const require = createRequire(import.meta.url);

// Resolve vue from this package's own node_modules so users don't need to install vue separately
export function resolveAliases (): AliasOptions {
  return [
    {
      find: 'vue/server-renderer',
      replacement: require.resolve('vue/server-renderer'),
    },
    {
      find: 'vue',
      replacement: require.resolve('vue/dist/vue.runtime.esm-bundler.js'),
    },
  ];
}
