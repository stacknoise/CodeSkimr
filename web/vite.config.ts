import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";

// The built UI is served by the local server under a strict content security policy (ADR 0028):
// no inline script or style, and no `data:` URL, so nothing may be inlined by the build.
export default defineConfig({
  plugins: [solid()],
  build: {
    outDir: "dist",
    assetsInlineLimit: 0,
    cssCodeSplit: false,
    target: "es2022",
  },
  server: { fs: { allow: [".."] } },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    setupFiles: ["src/test-setup.ts"],
    css: { modules: { classNameStrategy: "non-scoped" } },
  },
  resolve: { conditions: ["browser"] },
});
