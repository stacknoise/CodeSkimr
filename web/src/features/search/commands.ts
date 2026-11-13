import { EXPLORER_ROUTE, PROJECTS_ROUTE, SETTINGS_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { rescanProject } from "../projects/projects";

/** An action of the UI that the quick search offers with `>` (blueprint 15.4: every action has a
 * quick-search entry). */
export interface UiCommand {
  /** The text of the entry; it is also the key of the entry in the list. */
  label: MessageKey;
  run: () => void;
}

/** The commands of the quick search. */
export const UI_COMMANDS: readonly UiCommand[] = [
  { label: I18N_KEYS.NAV_PROJECTS, run: () => navigate(PROJECTS_ROUTE.path) },
  { label: I18N_KEYS.NAV_EXPLORER, run: () => navigate(EXPLORER_ROUTE.path) },
  { label: I18N_KEYS.NAV_SETTINGS, run: () => navigate(SETTINGS_ROUTE.path) },
  {
    label: I18N_KEYS.PALETTE_COMMAND_RESCAN,
    run: () => {
      const project = activeProject();
      if (project !== null) {
        void rescanProject(project);
      }
    },
  },
];
