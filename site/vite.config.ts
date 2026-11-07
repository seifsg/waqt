import { defineConfig } from "vite";

// Landing page: `npm run build` writes ../dist (index.html at its root) for static hosting.
export default defineConfig({
  build: { outDir: "../dist", emptyOutDir: true, target: "es2022" },
});
