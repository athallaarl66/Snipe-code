import js from '@eslint/js';
import pluginVue from 'eslint-plugin-vue';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['.nuxt/**', '.output/**', 'dist/**', 'node_modules/**'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    files: ['**/*.vue'],
    languageOptions: { parserOptions: { parser: tseslint.parser } },
  },
  {
    // Nuxt page/layout files are named after their route, which is single-word.
    files: ['pages/**/*.vue', 'layouts/**/*.vue'],
    rules: { 'vue/multi-word-component-names': 'off' },
  },
);
