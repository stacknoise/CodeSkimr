import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { SCOPE_COMMAND } from "./consts";
import { failed, rescanProject, type Failure } from "./projects";
import type { ScopeAnswer, ScopeChoice } from "./scope";

/** What the scope page of one project shows: the answer of the server, what the user chose so
 * far, and whether it is being read, saved or failed. */
export function useScopeState(project: () => number) {
  const [answer, setAnswer] = createSignal<ScopeAnswer | null>(null);
  const [choice, setChoice] = createSignal<ScopeChoice | null>(null);
  const [failure, setFailure] = createSignal<Failure | null>(null);
  const [saving, setSaving] = createSignal(false);
  const load = async () => {
    setFailure(null);
    try {
      const read = await runCommand<ScopeAnswer>(SCOPE_COMMAND.get, { project: project() });
      setAnswer(read);
      setChoice(read.scope);
    } catch (error) {
      setFailure(failed(error));
    }
  };
  const save = async () => {
    const chosen = choice();
    if (chosen === null || saving()) {
      return;
    }
    setSaving(true);
    setFailure(null);
    try {
      await runCommand(SCOPE_COMMAND.set, { project: project(), ...chosen });
      await rescanProject(project());
      await load();
    } catch (error) {
      setFailure(failed(error));
    } finally {
      setSaving(false);
    }
  };
  return { answer, choice, setChoice, failure, saving, load, save };
}

/** The state of the scope page. */
export type ScopeState = ReturnType<typeof useScopeState>;
