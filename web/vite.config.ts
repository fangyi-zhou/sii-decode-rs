/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import comlink from "vite-plugin-comlink";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait(), comlink()],
  worker: {
    plugins: () => [wasm(), topLevelAwait(), comlink()],
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
