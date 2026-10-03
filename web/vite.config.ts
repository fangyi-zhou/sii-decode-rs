/// <reference types="vitest" />
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";
import wasm from "vite-plugin-wasm";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), wasm()],
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
      provider: playwright(),
      enabled: true,
      headless: true,
      instances: [{ browser: "firefox" }],
    },
  },
});
