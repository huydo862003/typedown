import path from 'node:path';
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import {
  defineConfig,
} from 'vitest/config';

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],
  build: {
    lib: {
      entry: {
        index: path.resolve(__dirname, 'src/index.ts'),
        vite: path.resolve(__dirname, 'src/node/plugin/index.ts'),
        client: path.resolve(__dirname, 'src/client/index.ts'),
        'client/theme-default': path.resolve(__dirname, 'src/client/theme-default/index.ts'),
      },
      formats: ['es'],
    },
    rollupOptions: {
      external: [
        'vite',
        'vue',
        '@vitejs/plugin-vue',
        '@typerighter/rpc-client',
        '@typerighter/rpc-server',
        'markdown-it',
        'markdown-it-anchor',
        'markdown-it-container',
        'markdown-it-emoji',
        'markdown-it-task-lists',
        'shiki',
        '@shikijs/markdown-it',
        'tailwindcss',
        '@tailwindcss/vite',
        '@vueuse/core',
        '@vueuse/shared',
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
