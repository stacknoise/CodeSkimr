// The first import takes the launch code out of the address before any other script runs
// (ADR 0028). Imports run in this order.
import "./shared/session/capture";
import "./app/fonts.css";
import "./shared/theme/tokens.css";
import "./app/global.css";
import { render } from "solid-js/web";
import { App } from "./app/App";
import { ROOT_ID } from "./app/consts";
import { currentLanguage, setLanguage } from "./shared/i18n/i18n";
import { startSession } from "./shared/session/session";
import { loadUiConfig } from "./shared/settings/ui-config";
import { applyTheme } from "./shared/theme/theme";

const root = document.getElementById(ROOT_ID);
if (root) {
  applyTheme();
  setLanguage(currentLanguage());
  render(() => <App />, root);
  void loadUiConfig();
  void startSession();
}
