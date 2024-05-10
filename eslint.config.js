import config from '@tb-dev/eslint-config';

export default config({
  vue: true,
  project: ['tsconfig.json'],
  overrides: {
    unicorn: {
      'unicorn/consistent-function-scoping': 'off'
    }
  }
});
