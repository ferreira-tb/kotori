import tailwind from 'tailwindcss';
import { resolve } from 'node:path';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import autoprefixer from 'autoprefixer';
import dev from 'vite-plugin-vue-devtools';
import { URL, fileURLToPath } from 'node:url';
import autoImport from '@tb-dev/auto-import-config';

export default defineConfig({
  clearScreen: false,
  plugins: [
    vue(),
    dev(),
    autoImport({
      presets: {
        manatsu: true,
        tauri: true,
        vueuseRouter: true
      }
    })
  ],
  css: {
    postcss: {
      plugins: [tailwind(), autoprefixer()]
    }
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('src', import.meta.url))
    }
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  build: {
    emptyOutDir: true,
    minify: true,
    rollupOptions: {
      input: {
        main: entry('main'),
        reader: entry('reader')
      }
    }
  }
});

function entry(name) {
  return resolve(__dirname, `src/windows/${name}/index.html`);
}
