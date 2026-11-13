import { resolve } from "node:path";
import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";
import { exportPlugins } from "./vite.export";

/** Mode of the second build, the one that makes the page the HTML export starts from. */
const EXPORT_MODE = "export";

// The built UI is served by the local server under a strict content security policy (ADR 0028):
// no inline script or style, and no `data:` URL, so nothing may be inlined by the build.
const app = defineConfig({
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

// The export is one file that is opened from disk (blueprint 10.2): the script, the styles and
// every asset are inlined, the commands answer from the embedded data, and nothing is fetched.
// The build goes below `dist/export`; the server never serves that file as the UI.
const exportBuild = defineConfig({
  plugins: [...exportPlugins(resolve(import.meta.dirname, "src")), solid()],
  build: {
    outDir: "dist/export",
    copyPublicDir: false,
    emptyOutDir: true,
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    cssCodeSplit: false,
    modulePreload: false,
    target: "es2022",
    rollupOptions: { input: resolve(import.meta.dirname, "export.html") },
  },
  resolve: { conditions: ["browser"] },
});

export default defineConfig(({ mode }) => (mode === EXPORT_MODE ? exportBuild : app));
