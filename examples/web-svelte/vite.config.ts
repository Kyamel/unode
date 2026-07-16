import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  root: "demo",
  plugins: [svelte()],
  resolve: {
    alias: {
      "unode-web-core": new URL("../../packages/unode-web-core/src", import.meta.url).pathname,
      "unode-web-renderer": new URL("../../packages/unode-web-renderer/src", import.meta.url).pathname,
      "unode-svelte": new URL("../../packages/unode-svelte/src", import.meta.url).pathname,
    },
  },
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
