import { beforeEach, describe, expect, it } from "vitest";
import { I18N_KEYS } from "./keys";
import { errorText, setLanguage, systemLanguage, t } from "./i18n";

beforeEach(() => {
  localStorage.clear();
  setLanguage("en");
});

describe("t", () => {
  it("speaks the chosen language and switches at once", () => {
    expect(t(I18N_KEYS.SETTINGS_TITLE)).toBe("Settings");
    setLanguage("de");
    expect(t(I18N_KEYS.SETTINGS_TITLE)).toBe("Einstellungen");
    expect(document.documentElement.getAttribute("lang")).toBe("de");
    expect(localStorage.getItem("codeskimr.language")).toBe("de");
  });

  it("fills in arguments without invisible marks around them", () => {
    expect(t(I18N_KEYS.PREVIEW_BADGE_NEW, { count: 12 })).toBe("12 new");
    setLanguage("de");
    expect(t(I18N_KEYS.PREVIEW_BADGE_NEW, { count: 12 })).toBe("12 neu");
  });

  it("falls back to the key for a message that does not exist", () => {
    expect(t("no-such-message" as never)).toBe("no-such-message");
  });
});

describe("errorText", () => {
  it("gives the text of a registered code in both languages", () => {
    expect(errorText("API-403")).toContain("launch code");
    setLanguage("de");
    expect(errorText("API-403")).toContain("Startcode");
  });
});

describe("systemLanguage", () => {
  it("is German for a German browser and English for everything else", () => {
    expect(systemLanguage(["de-AT", "en"])).toBe("de");
    expect(systemLanguage(["fr", "en-US"])).toBe("en");
    expect(systemLanguage([])).toBe("en");
  });
});
