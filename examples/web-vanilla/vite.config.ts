import { defineConfig } from "vite";

export default defineConfig({
  root: "demo",
  resolve: {
    alias: {
      "unode-web-core": new URL("../../packages/unode-web-core/src", import.meta.url).pathname,
      "unode-web-renderer": new URL("../../packages/unode-web-renderer/src", import.meta.url).pathname,
    },
  },
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
