// @ts-check
import { fileURLToPath } from 'node:url';

import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://unode.dev',
	vite: {
		resolve: {
			alias: {
				'unode-core': fileURLToPath(new URL('../packages/unode-core/src/index.ts', import.meta.url)),
				'unode-react': fileURLToPath(new URL('../packages/unode-react/src/index.ts', import.meta.url)),
				'unode-renderer': fileURLToPath(
					new URL('../packages/unode-renderer/src/index.ts', import.meta.url),
				),
			},
		},
	},
	integrations: [
		react(),
		starlight({
			title: 'Unode',
			description:
				'A renderer-agnostic, plugin-first semantic UI protocol. Write UI once in a WASM sandbox; render it on web and terminal hosts.',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/Kyamel/unode' },
			],
			editLink: {
				baseUrl: 'https://github.com/Kyamel/unode/edit/main/website/',
			},
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started/introduction' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quickstart: a plugin', slug: 'getting-started/quickstart' },
					],
				},
				{
					label: 'Concepts',
					items: [
						{ label: 'Architecture', slug: 'concepts/architecture' },
						{ label: 'Runtime & Lifecycle', slug: 'concepts/runtime' },
						{ label: 'Reactivity', slug: 'concepts/reactivity' },
						{ label: 'WASM Sandbox', slug: 'concepts/wasm-sandbox' },
						{ label: 'Permissions', slug: 'concepts/permissions' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Monorepo Layout', slug: 'reference/monorepo' },
						{ label: 'Plugin WASM ABI', slug: 'reference/plugin-abi' },
						{ label: 'Roadmap', slug: 'reference/roadmap' },
					],
				},
			],
		}),
	],
});
