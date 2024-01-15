import { defineConfig } from 'taze';

export default defineConfig({
  force: true,
  write: true,
  includeLocked: true,
  install: false,
  packageMode: {
    vue: 'ignore',
    '@vitejs/plugin-vue': 'ignore',
    '/manatsu/': 'major'
  }
});
