import { createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { Adder } from "./Adder";
import { addTask } from "./api";
import { TaskCard } from "./Cards";
import { KIND, STATUS } from "./consts";
import { TaskForm } from "./Forms";
import { report, useRequestedForm } from "./panelState";
import { change } from "./store";
import type { Annotation, Target } from "./types";
import styles from "./Tabs.module.css";

/** The tab "Tasks" of the side panel (mock 14): "Tasks (1 of 3 done)", the tasks and the button
 * "Add task". */
export function TasksTab(props: { project: number; target: Target; tasks: Annotation[] }) {
  const [adding, setAdding] = createSignal(false);
  useRequestedForm(KIND.task, () => setAdding(true));
  const done = () => props.tasks.filter((task) => task.status === STATUS.done).length;
  return (
    <div class={styles.content}>
      <Show
        when={props.tasks.length > 0}
        fallback={
          <Show when={!adding()}>
            <p class={styles.empty}>{t(I18N_KEYS.ANNOTATIONS_EMPTY_TASKS)}</p>
          </Show>
        }
      >
        <div class={styles.label}>
          {t(I18N_KEYS.ANNOTATIONS_TASKS_SUMMARY, { done: done(), total: props.tasks.length })}
        </div>
      </Show>
      <For each={props.tasks}>
        {(task) => (
          <TaskCard
            project={props.project}
            task={task}
            onFailed={(failure) => report(failure, null)}
          />
        )}
      </For>
      <Adder
        adding={adding()}
        label={t(I18N_KEYS.ANNOTATIONS_ADD_TASK)}
        onAdd={() => setAdding(true)}
      >
        <TaskForm
          save={(draft) => change(props.project, () => addTask(props.project, props.target, draft))}
          onClose={() => setAdding(false)}
        />
      </Adder>
    </div>
  );
}
