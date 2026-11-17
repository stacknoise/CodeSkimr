import { createSignal, type Accessor } from "solid-js";
import { readSetting, writeSetting } from "../../shared/settings/storage";
import { FLAG_OFF, FLAG_ON, HIGHLIGHT_KEY, NEIGHBOURS_KEY, SHOW_TEXT_KEY } from "./consts";

/** A switch that the browser remembers (`localStorage`, a convenience of this viewer only). */
function storedFlag(key: string, fallback: boolean): [Accessor<boolean>, (on: boolean) => void] {
  const stored = readSetting(key);
  const [on, setOn] = createSignal(stored === null ? fallback : stored === FLAG_ON);
  return [
    on,
    (next) => {
      writeSetting(key, next ? FLAG_ON : FLAG_OFF);
      setOn(next);
    },
  ];
}

const [textShown, setTextShown] = storedFlag(SHOW_TEXT_KEY, false);
const [highlighting, setHighlighting] = storedFlag(HIGHLIGHT_KEY, true);
const [neighboursOnly, setNeighboursOnly] = storedFlag(NEIGHBOURS_KEY, false);

/** The switch "Show annotations": the text of notes and tasks under the row of the member table. */
export const showAnnotationText = textShown;
export const setShowAnnotationText = setTextShown;
/** The switch "Highlight critical" of the dependency map. */
export const highlightCritical = highlighting;
export const setHighlightCritical = setHighlighting;
/** The switch "only critical and their neighbours" of the dependency map. */
export const onlyCriticalAndNeighbours = neighboursOnly;
export const setOnlyCriticalAndNeighbours = setNeighboursOnly;
