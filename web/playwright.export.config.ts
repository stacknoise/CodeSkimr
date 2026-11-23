import { defineConfig } from "@playwright/test";

// The check of one exported page that the program itself wrote (`e2e-export`, issue #670). It
// needs no server: the page is opened from disk. CODESKIMR_CHROMIUM names a Chromium that is
// installed already.
const executablePath = process.env["CODESKIMR_CHROMIUM"];

export default defineConfig({
  testDir: "e2e-export",
  workers: 1,
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
