/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  test: {
    setupFiles: [
      "tests/vitest-setup-dom.ts",
      "tests/vitest-cleanup-after-each.ts",
    ],
    environment: "happy-dom",
    // https://github.com/vitest-dev/vitest/issues/2150
    deps: {
      inline: [/\?url$/],
    },
  },
});
