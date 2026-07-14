import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  root: "demo",
  plugins: [react()],
  // wasm assets are imported with `?url` and fetched at runtime.
  assetsInclude: ["**/*.wasm"],
});
