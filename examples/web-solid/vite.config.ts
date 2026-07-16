import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  root: "demo",
  plugins: [solid()],
  resolve: {
    alias: {
      "unode-web-core": new URL("../../packages/unode-web-core/src", import.meta.url).pathname,
      "unode-solid": new URL("../../packages/unode-solid/src", import.meta.url).pathname,
      "unode-web-renderer": new URL("../../packages/unode-web-renderer/src", import.meta.url).pathname,
    },
  },
  assetsInclude: ["**/*.wasm"],
});
