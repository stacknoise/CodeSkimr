import { defineConfig } from "@playwright/test";

// The end-to-end tests run against the real server (see e2e/fixtures.ts). In a sandbox without
// the browsers of Playwright, CODESKIMR_CHROMIUM names a Chromium that is installed already.
const executablePath = process.env["CODESKIMR_CHROMIUM"];

export default defineConfig({
  testDir: "e2e",
  workers: 1,
  fullyParallel: false,
  retries: 0,
  timeout: 60_000,
  reporter: "list",
  use: {
    browserName: "chromium",
    locale: "en-US",
    trace: "off",
    launchOptions: executablePath ? { executablePath } : {},
  },
});
