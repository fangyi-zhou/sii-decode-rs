/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  worker: {
    format: "es",
    plugins: () => [wasm()],
  },
  test: {
    setupFiles: [
      "tests/vitest-setup-dom.ts",
      "tests/vitest-cleanup-after-each.ts",
    ],
    environment: "happy-dom",
    browser: {
      provider: "playwright",
      enabled: true,
      headless: true,
      instances: [{ browser: "firefox" }],
    },
  },
});
