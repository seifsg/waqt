import { defineConfig } from "vite";

// Settings window bundle, served to Tauri on a fixed port in dev.
export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true, target: "es2022" },
});
