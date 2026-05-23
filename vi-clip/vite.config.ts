import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    fs: {
      allow: ["."],
      deny: ["**/src-tauri/target/**"],
    },
  },
  optimizeDeps: {
    exclude: ["src-tauri"],
    entries: ["./src-web/main.tsx"],
  },
});
