import { createSignal } from "solid-js";
import { api } from "../api/client";
import { UI_CONFIG_PATH } from "../api/consts";
import type { EditorConfig } from "../editor/editor";
import { useDefaultTheme } from "../theme/theme";

/** What the server tells the UI about its settings (`GET /api/v1/ui-config`). */
export interface UiConfig {
  editor: EditorConfig;
  default_theme: string;
}

const [config, setConfig] = createSignal<UiConfig | null>(null);

/** The settings of the server, once they are loaded. */
export const uiConfig = config;

/** Loads the settings. They are not secret, so no token is needed. A server that does not
 * answer leaves the built-in defaults of the UI in place. */
export async function loadUiConfig(): Promise<void> {
  try {
    const loaded = await api<UiConfig>(UI_CONFIG_PATH, { token: null });
    setConfig(loaded);
    useDefaultTheme(loaded.default_theme);
  } catch {
    // The connection screen reports an unreachable server.
  }
}
