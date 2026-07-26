import path from 'node:path';
import {
  defineConfig,
} from 'vitest/config';

export default defineConfig({
  build: {
    lib: {
      entry: path.resolve(__dirname, 'src/index.ts'),
      formats: [
        'es',
        'cjs',
      ],
      fileName: 'index',
    },
    rollupOptions: {
      external: [
        'vite',
        'vue',
        'rpc-client',
        'rpc-server',
        'markdown-it',
        'markdown-it-anchor',
        'markdown-it-container',
        'markdown-it-emoji',
        'markdown-it-task-lists',
        'shiki',
        '@shikijs/markdown-it',
        /^node:/,
      ],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src/'),
    },
  },
  test: {},
});
