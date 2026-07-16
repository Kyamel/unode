// The engine declaration: assemble the recipes into a renderer once and
// share it. Overriding a node's look is a recipes.ts edit; this file is the
// wiring, mirroring `ratatui_renderer().recipes([...]).build()` on the TUI.
import { defineRenderer } from 'unode-react';

import { fallbackRecipe, nodeRecipes, screenRecipe } from './recipes';

export const playgroundRenderer = defineRenderer()
	.screen(screenRecipe)
	.recipes(nodeRecipes)
	.fallback(fallbackRecipe)
	.build();
