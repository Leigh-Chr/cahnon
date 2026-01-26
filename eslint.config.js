import eslint from '@eslint/js';
import simpleImportSort from 'eslint-plugin-simple-import-sort';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
	eslint.configs.recommended,
	...tseslint.configs.recommended,
	...svelte.configs['flat/recommended'],
	{
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node,
			},
		},
	},
	{
		files: ['**/*.svelte'],
		languageOptions: {
			parserOptions: {
				parser: tseslint.parser,
			},
		},
	},
	{
		// Svelte 5 runes modules (.svelte.ts files) should be parsed as TypeScript
		files: ['**/*.svelte.ts'],
		languageOptions: {
			parser: tseslint.parser,
		},
	},
	{
		plugins: {
			'simple-import-sort': simpleImportSort,
		},
		rules: {
			'simple-import-sort/imports': 'error',
			'simple-import-sort/exports': 'error',
			// Allow unused vars starting with underscore
			'@typescript-eslint/no-unused-vars': [
				'warn',
				{
					argsIgnorePattern: '^_',
					varsIgnorePattern: '^_',
					caughtErrorsIgnorePattern: '^_',
				},
			],
			// Disable some rules that are too strict for this project
			'@typescript-eslint/no-explicit-any': 'warn',
			'@typescript-eslint/no-empty-object-type': 'off',
			// Svelte-specific rules
			'svelte/require-each-key': 'warn',
			'svelte/prefer-svelte-reactivity': 'warn',
			'svelte/no-unused-svelte-ignore': 'warn',
			// Allow case declarations with proper scoping
			'no-case-declarations': 'off',
		},
	},
	{
		ignores: ['build/', '.svelte-kit/', 'dist/', 'src-tauri/target/', 'node_modules/'],
	}
);
