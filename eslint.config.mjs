// Shared flat ESLint config for every TypeScript package in the workspace.
// Each package exposes a `lint` script; ESLint resolves this file by walking
// up from the package directory.
import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';

export default tseslint.config(
	{
		ignores: [
			'**/node_modules/**',
			'**/dist/**',
			'**/target/**',
			// Deprecated legacy code kept only as migration reference.
			'ts-implementation/**',
			// Generated wasm-bindgen output.
			'**/pkg/**',
			'website/.astro/**',
			'**/*.astro',
			// SFCs need eslint-plugin-vue; the demo lint scripts target .ts files.
			'**/*.vue',
		],
	},
	js.configs.recommended,
	...tseslint.configs.recommended,
	...svelte.configs['flat/recommended'],
	{
		// Includes `.svelte.ts`/`.svelte.js` rune modules, which the Svelte
		// parser also claims and needs the TS parser for.
		files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				parser: tseslint.parser,
			},
		},
	},
	{
		files: ['**/*.{ts,tsx,mts,js,mjs,svelte}'],
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node,
			},
		},
		rules: {
			// The IR crosses a JSON boundary; narrow types are not always available.
			'@typescript-eslint/no-explicit-any': 'warn',
			'@typescript-eslint/no-unused-vars': [
				'error',
				{ argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
			],
		},
	},
	{
		files: [
			'packages/unode-react/**/*.tsx',
			'examples/web-react/**/*.tsx',
			'website/**/*.tsx',
		],
		plugins: { 'react-hooks': reactHooks },
		rules: reactHooks.configs.recommended.rules,
	},
);
