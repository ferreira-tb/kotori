import config from '@tb-dev/eslint-config';

export default config({
  vue: true,
  project: ['desktop/tsconfig.json'],
  ignores: ['desktop/src/components/ui/*'],
  overrides: {
    javascript: {
      'no-undefined': 'off'
    },
    typescript: {
      '@typescript-eslint/no-explicit-any': 'off'
    },
    vue: {
      'vue/component-name-in-template-casing': [
        'error',
        'PascalCase',
        { registeredComponentsOnly: false }
      ]
    }
  }
});
