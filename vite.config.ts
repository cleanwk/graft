import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  publicDir: "assets",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    target: "safari26",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
});
