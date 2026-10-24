import { defineConfig } from "vitest/config";

export default defineConfig({
  base: "./",
  build: {
    outDir: "../app/assets/score-view-web",
    emptyOutDir: true,
  },
  test: {
    passWithNoTests: true,
  },
});
