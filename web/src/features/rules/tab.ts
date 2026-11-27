import { createSignal } from "solid-js";
import { navigate } from "../../app/router";
import { RULES_ROUTE } from "../../app/consts";
import { RULES_TAB, type RulesTab } from "./consts";

const [tab, setTab] = createSignal<RulesTab>(RULES_TAB.rules);

/** The tab of the screen Rules that is shown. */
export const rulesTab = tab;

/** Shows a tab of the screen Rules. */
export function showRulesTab(next: RulesTab): void {
  setTab(next);
}

/** Opens the screen Rules on the tab of the layers: where the layers are set (ADR 0112). */
export function showLayers(): void {
  setTab(RULES_TAB.layers);
  navigate(RULES_ROUTE.path);
}
