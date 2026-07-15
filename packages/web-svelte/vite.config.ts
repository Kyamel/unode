import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  root: "demo",
  plugins: [svelte()],
  resolve: {
    alias: {
      "unode-core": new URL("../unode-core/src", import.meta.url).pathname,
      "unode-renderer": new URL("../unode-renderer/src", import.meta.url).pathname,
    },
  },
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
