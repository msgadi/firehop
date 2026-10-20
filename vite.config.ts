import { resolve } from "node:path";
import { defineConfig } from "vite";

// Two small local pages: the account strip and the account editor. Tauri
// serves the built files in production.
export default defineConfig({
  root: "ui",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2023",
    rollupOptions: {
      input: {
        strip: resolve(import.meta.dirname, "ui/index.html"),
        editor: resolve(import.meta.dirname, "ui/editor.html"),
      },
    },
  },
});
