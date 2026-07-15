import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  root: "demo",
  plugins: [react()],
  resolve: {
    alias: {
      "unode-core": new URL("../unode-core/src", import.meta.url).pathname,
      "unode-renderer": new URL("../unode-renderer/src", import.meta.url).pathname,
    },
  },
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
