import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  root: "demo",
  plugins: [react()],
  resolve: {
    alias: {
      "unode-core": new URL("../../packages/unode-core/src", import.meta.url).pathname,
      "unode-react": new URL("../../packages/unode-react/src", import.meta.url).pathname,
      "unode-renderer": new URL("../../packages/unode-renderer/src", import.meta.url).pathname,
    },
  },
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
