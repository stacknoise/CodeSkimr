import { createEffect, createSignal, on, onCleanup, Show } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { refreshProjects } from "../projects/projects";
import { NEW_NAME_FIELD_ID } from "./consts";
import {
  addLayer,
  addToLayer,
  moveLayer,
  removeLayer,
  removeNamespace,
  renameLayer,
} from "./draft";
import { LayerList, type LayerActions } from "./LayerList";
import {
  layersChanged,
  layersData,
  layersDraft,
  layersFailure,
  layersProject,
  layersSavable,
  layersSaved,
  layersSaving,
  loadLayers,
  resetLayers,
  saveLayers,
  setLayersDraft,
} from "./layers";
import { NamespacePicker } from "./NamespacePicker";
import type { LayersData } from "./types";
import { ErrorDetails } from "../../shared/ui/ErrorDetails";
import styles from "./LayersScreen.module.css";

/** The choice, the name of a new layer, and what they do together. */
function useChoice() {
  const [chosen, setChosen] = createSignal<ReadonlySet<string>>(new Set());
  const [name, setName] = createSignal("");
  const picked = () => [...chosen()];
  const done = () => {
    setChosen(new Set<string>());
    setName("");
  };
  const actions: LayerActions = {
    onRename: (index, next) => setLayersDraft(renameLayer(layersDraft(), index, next)),
    onMove: (index, step) => setLayersDraft(moveLayer(layersDraft(), index, step)),
    onRemove: (index) => setLayersDraft(removeLayer(layersDraft(), index)),
    onRemoveNamespace: (index, prefix) =>
      setLayersDraft(removeNamespace(layersDraft(), index, prefix)),
    onAddSelection: (index) => {
      setLayersDraft(addToLayer(layersDraft(), index, picked()));
      done();
    },
  };
  const create = () => {
    setLayersDraft(addLayer(layersDraft(), name(), picked()));
    done();
  };
  return { chosen, setChosen, name, setName, actions, create, hasChoice: () => chosen().size > 0 };
}

type Choice = ReturnType<typeof useChoice>;

/** The field for the name of a new layer and the button that makes it from the choice. */
function NewLayerRow(props: { choice: Choice }) {
  const { choice } = props;
  const canCreate = () => choice.hasChoice() && choice.name().trim() !== "";
  return (
    <div class={styles.newRow}>
      <input
        id={NEW_NAME_FIELD_ID}
        class={styles.field}
        value={choice.name()}
        placeholder={t(I18N_KEYS.LAYERS_NEW_NAME)}
        aria-label={t(I18N_KEYS.LAYERS_NEW_NAME)}
        onInput={(event) => choice.setName(event.currentTarget.value)}
      />
      <button type="button" class={styles.primary} disabled={!canCreate()} onClick={choice.create}>
        {t(I18N_KEYS.LAYERS_NEW_ACTION)}
      </button>
    </div>
  );
}

/** The tree of namespaces with the field that names a new layer. */
function PickerPanel(props: { data: LayersData; choice: Choice }) {
  const { choice } = props;
  return (
    <section class={styles.panel}>
      <h2>{t(I18N_KEYS.LAYERS_NAMESPACES_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.LAYERS_NAMESPACES_NOTE)}</p>
      <Show
        when={props.data.namespaces.length > 0}
        fallback={<p class={styles.note}>{t(I18N_KEYS.LAYERS_NAMESPACES_EMPTY)}</p>}
      >
        <NamespacePicker
          namespaces={props.data.namespaces}
          layers={layersDraft()}
          chosen={choice.chosen()}
          onChosen={choice.setChosen}
        />
      </Show>
      <p class={styles.note}>
        {t(I18N_KEYS.LAYERS_SELECTED_COUNT, { count: choice.chosen().size })}
      </p>
      <NewLayerRow choice={choice} />
    </section>
  );
}

/** The editor itself: the tree on the left, the layers on the right, the save row below. */
function Editor(props: { data: LayersData; project: number }) {
  const choice = useChoice();
  return (
    <>
      <div class={styles.columns}>
        <PickerPanel data={props.data} choice={choice} />
        <section class={styles.panel}>
          <h2>{t(I18N_KEYS.LAYERS_LAYERS_TITLE)}</h2>
          <p class={styles.note}>{t(I18N_KEYS.LAYERS_ORDER_NOTE)}</p>
          <LayerList layers={layersDraft()} canAdd={choice.hasChoice()} actions={choice.actions} />
        </section>
      </div>
      <SaveRow project={props.project} />
    </>
  );
}

/** Save and discard, and what the last save led to. */
function SaveRow(props: { project: number }) {
  return (
    <div class={styles.saveRow}>
      <button
        type="button"
        class={styles.primary}
        disabled={!layersSavable()}
        onClick={() => void saveLayers(props.project)}
      >
        {layersSaving() ? t(I18N_KEYS.LAYERS_SAVING) : t(I18N_KEYS.LAYERS_SAVE)}
      </button>
      <button
        type="button"
        class={styles.secondary}
        disabled={!layersChanged() || layersSaving()}
        onClick={() => setLayersDraft(layersData()?.layers ?? [])}
      >
        {t(I18N_KEYS.LAYERS_RESET)}
      </button>
      <Show when={layersChanged() && !layersSavable() && !layersSaving()}>
        <span class={styles.note}>{t(I18N_KEYS.LAYERS_INVALID)}</span>
      </Show>
      <Show when={layersSaved() && !layersChanged()}>
        <span class={styles.note} role="status">
          {t(I18N_KEYS.LAYERS_SAVED)}
        </span>
      </Show>
      <p class={styles.note}>{t(I18N_KEYS.LAYERS_FILE_NOTE)}</p>
    </div>
  );
}

/** The text that says why there is nothing to edit, `null` if the editor can be shown. */
function blocked(data: LayersData): string | null {
  if (!data.editable) {
    return t(I18N_KEYS.LAYERS_NOT_EDITABLE);
  }
  return data.scanned ? null : t(I18N_KEYS.LAYERS_NOT_SCANNED);
}

/** The failure, or the editor, or the reason why there is nothing to edit. */
function ProjectView(props: { project: number }) {
  return (
    <>
      <Show when={layersFailure()}>
        {(failure) => (
          <div class={styles.alert} role="alert">
            {failure().code === null
              ? t(I18N_KEYS.LAYERS_LOAD_FAILED)
              : errorText(failure().code as ErrorCode)}
            <Show when={failure().code}>
              {(code) => <ErrorDetails code={code()} message={errorText(code())} />}
            </Show>
          </div>
        )}
      </Show>
      <Show when={layersData()}>
        {(data) => (
          <Show when={blocked(data())} fallback={<Editor data={data()} project={props.project} />}>
            {(text) => <p class={styles.note}>{text()}</p>}
          </Show>
        )}
      </Show>
    </>
  );
}

/** The screen that sets the layers of a project from its detected namespaces (ADR 0065). */
export function LayersScreen() {
  createEffect(
    on(layersProject, (id) => {
      resetLayers();
      if (id !== null) {
        void loadLayers(id);
      }
    }),
  );
  onCleanup(resetLayers);
  void refreshProjects();
  return (
    <>
      <header>
        <h1>{t(I18N_KEYS.LAYERS_TITLE)}</h1>
        <p class={styles.sub}>{t(I18N_KEYS.LAYERS_SUBTITLE)}</p>
      </header>
      <Show
        when={layersProject()}
        fallback={<p class={styles.note}>{t(I18N_KEYS.LAYERS_NO_PROJECT)}</p>}
      >
        {(project) => <ProjectView project={project()} />}
      </Show>
    </>
  );
}
