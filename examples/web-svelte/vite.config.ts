import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  root: "demo",
  plugins: [svelte()],
  resolve: {
    alias: {
      "unode-core": new URL("../../packages/unode-core/src", import.meta.url).pathname,
      "unode-renderer": new URL("../../packages/unode-renderer/src", import.meta.url).pathname,
      "unode-svelte": new URL("../../packages/unode-svelte/src", import.meta.url).pathname,
    },
  },
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
