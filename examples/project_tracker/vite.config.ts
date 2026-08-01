import { defineConfig } from 'vite';
import { typedown } from 'typerighter/vite';

export default defineConfig({
  plugins: [typedown()],
});
