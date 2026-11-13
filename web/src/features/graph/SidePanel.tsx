import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { count } from "../projects/format";
import type { Neighbours } from "./neighbours";
import type { LayerLink } from "./types";
import styles from "./SidePanel.module.css";

function Stat(props: { value: number; label: string; tone?: string | undefined }) {
  return (
    <div class={styles.stat}>
      <div class={cx(styles.number, props.tone)}>{count(props.value, currentLanguage())}</div>
      <div class={styles.label}>{props.label}</div>
    </div>
  );
}

function LinkList(props: {
  title: string;
  links: readonly LayerLink[];
  pick: (link: LayerLink) => string;
  onSelect: (name: string) => void;
  tone: string | undefined;
}) {
  return (
    <Show when={props.links.length > 0}>
      <section class={styles.group}>
        <h3 class={cx(styles.groupTitle, props.tone)}>{props.title}</h3>
        <For each={props.links}>
          {(link) => (
            <div class={styles.link}>
              <button
                type="button"
                class={styles.name}
                onClick={() => props.onSelect(props.pick(link))}
              >
                {props.pick(link)}
              </button>
              <span class={styles.detail}>
                {t(I18N_KEYS.GRAPH_LINK_REFERENCES, { count: link.count })}
              </span>
            </div>
          )}
        </For>
      </section>
    </Show>
  );
}

function Violations(props: { links: readonly LayerLink[]; onSelect: (name: string) => void }) {
  return (
    <Show when={props.links.length > 0}>
      <div class={styles.violation} role="note">
        <strong>{t(I18N_KEYS.GRAPH_VIOLATION_TITLE, { count: props.links.length })}</strong>
        <For each={props.links}>
          {(link) => (
            <div>
              {t(I18N_KEYS.GRAPH_VIOLATION_PAIR, {
                from: link.from,
                to: link.to,
                count: link.count,
              })}{" "}
              <button type="button" class={styles.show} onClick={() => props.onSelect(link.from)}>
                {t(I18N_KEYS.GRAPH_VIOLATION_SHOW)}
              </button>
            </div>
          )}
        </For>
      </div>
    </Show>
  );
}

/** The panel at the right (mock 3): the selected layer with its counts, what it uses and what
 * uses it, the breaks of the layering of the project, and the way to the explorer. */
export function SidePanel(props: {
  layer: string;
  neighbours: Neighbours;
  violations: readonly LayerLink[];
  onSelect: (name: string) => void;
  onOpen: () => void;
}) {
  return (
    <aside class={styles.panel} aria-label={t(I18N_KEYS.GRAPH_ARIA_SELECTION)}>
      <div>
        <div class={styles.kind}>{t(I18N_KEYS.GRAPH_SELECTED_KIND)}</div>
        <h2 class={styles.title}>{props.layer}</h2>
      </div>
      <div class={styles.stats}>
        <Stat value={props.neighbours.references} label={t(I18N_KEYS.GRAPH_STAT_REFERENCES)} />
        <Stat
          value={props.neighbours.uses.length}
          label={t(I18N_KEYS.GRAPH_STAT_USES)}
          tone={styles.out}
        />
        <Stat
          value={props.neighbours.usedBy.length}
          label={t(I18N_KEYS.GRAPH_STAT_USED_BY)}
          tone={styles.in}
        />
      </div>
      <LinkList
        title={t(I18N_KEYS.GRAPH_USES_TITLE, { count: props.neighbours.uses.length })}
        links={props.neighbours.uses}
        pick={(link) => link.to}
        onSelect={props.onSelect}
        tone={styles.out}
      />
      <LinkList
        title={t(I18N_KEYS.GRAPH_USED_BY_TITLE, { count: props.neighbours.usedBy.length })}
        links={props.neighbours.usedBy}
        pick={(link) => link.from}
        onSelect={props.onSelect}
        tone={styles.in}
      />
      <Violations links={props.violations} onSelect={props.onSelect} />
      <button type="button" class={styles.open} onClick={props.onOpen}>
        {t(I18N_KEYS.APP_ACTION_OPEN_IN_EXPLORER)} <kbd>{t(I18N_KEYS.GRAPH_OPEN_KEY)}</kbd>
      </button>
    </aside>
  );
}
