import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html", "lcov"],
      exclude: [
        "dist",
        "node_modules",
        "**/*.d.ts",
        "**/*.config.*",
        "**/tests/**",
      ],
    },
  },
});
