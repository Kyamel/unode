import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  root: "demo",
  plugins: [vue()],
  resolve: {
    alias: {
      "unode-web-core": new URL("../../packages/unode-web-core/src", import.meta.url).pathname,
      "unode-vue": new URL("../../packages/unode-vue/src", import.meta.url).pathname,
      "unode-web-renderer": new URL("../../packages/unode-web-renderer/src", import.meta.url).pathname,
    },
  },
  assetsInclude: ["**/*.wasm"],
});
