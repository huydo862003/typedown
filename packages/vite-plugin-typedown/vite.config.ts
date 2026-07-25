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
      external: ['vite'],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src/'),
    },
  },
  test: {},
});
