import { createSignal, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { loadPlaces } from "./graph";
import type { ModuleEdge, PlacesData } from "./types";
import styles from "./ModulePlaces.module.css";

/** `path:line` of a place, as an editor takes it. */
function where(place: { path: string; line: number }): string {
  return `${place.path}:${place.line}`;
}

function List(props: { found: PlacesData }) {
  const left = () => props.found.total - props.found.places.length;
  return (
    <>
      <ul class={styles.list}>
        <For each={props.found.places}>
          {(place) => (
            <li>
              <span class={styles.where}>{where(place)}</span>
              <span class={styles.uses}>
                {t(I18N_KEYS.GRAPH_MODULES_PLACE_USES, { from: place.from, to: place.to })}
              </span>
            </li>
          )}
        </For>
      </ul>
      <Show when={left() > 0}>
        <p class={styles.more}>{t(I18N_KEYS.GRAPH_MODULES_PLACES_MORE, { count: left() })}</p>
      </Show>
    </>
  );
}

/** The button that opens the places of a dependency between two modules, with what it found. The
 * places are asked for once, when the button is pressed the first time. */
export function EdgePlaces(props: { project: number; edge: ModuleEdge }) {
  const [open, setOpen] = createSignal(false);
  const [found, setFound] = createSignal<PlacesData | null>(null);
  const [failed, setFailed] = createSignal(false);
  const toggle = async () => {
    setOpen(!open());
    if (open() && found() === null) {
      try {
        setFound(await loadPlaces(props.project, props.edge));
        setFailed(false);
      } catch {
        setFailed(true);
      }
    }
  };
  return (
    <div class={styles.places}>
      <button type="button" class={styles.show} aria-expanded={open()} onClick={toggle}>
        {open()
          ? t(I18N_KEYS.GRAPH_MODULES_PLACES_HIDE)
          : t(I18N_KEYS.GRAPH_MODULES_PLACES_SHOW, { count: props.edge.count })}
      </button>
      <Show when={open()}>
        <Show when={failed()}>
          <p class={styles.more} role="alert">
            {t(I18N_KEYS.GRAPH_MODULES_PLACES_FAILED)}
          </p>
        </Show>
        <Show when={found()}>{(list) => <List found={list()} />}</Show>
      </Show>
    </div>
  );
}
