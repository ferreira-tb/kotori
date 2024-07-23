import { defineConfig } from '@tb-dev/eslint-config';

export default defineConfig({
  project: ['desktop/tsconfig.json'],
  ignores: ['desktop/src/components/ui/*'],
  features: {
    vue: true,
    stylistic: true,
    jsonc: true,
    tailwind: true,
  },
  overrides: {
    javascript: {
      'no-undefined': 'off',
    },
    typescript: {
      '@typescript-eslint/no-explicit-any': 'off',
    },
    vue: {
      'vue/component-name-in-template-casing': [
        'error',
        'PascalCase',
        { registeredComponentsOnly: false },
      ],
    },
  },
});
