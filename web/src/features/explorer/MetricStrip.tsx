import { onMount } from "solid-js";
import { currentLanguage } from "../../shared/i18n/i18n";
import { activeProject } from "../../shared/project/active";
import { projects, projectsLoaded, refreshProjects } from "../projects/projects";
import { tilesOf } from "./metrics";
import { TileGrid } from "./TileGrid";
import type { SymbolInfo, Thresholds } from "./types";
import { isUpperBound } from "./upperBound";
import styles from "./MetricStrip.module.css";

/** The grid of ten metrics of the selected symbol (mock 2); the colour follows the thresholds of
 * the configuration, the number says the same, so colour is not the only carrier. The coupling
 * tiles say "upper bound" while level 2 has not confirmed the language of the symbol (ADR 0037);
 * until the project list is read they say nothing. */
export function MetricStrip(props: { info: SymbolInfo; thresholds: Thresholds | null }) {
  onMount(() => {
    if (!projectsLoaded()) {
      void refreshProjects();
    }
  });
  const upperBound = () => {
    const row = projects().find((entry) => entry.id === activeProject());
    return projectsLoaded() && isUpperBound(row?.report, props.info.language);
  };
  const tiles = () => tilesOf(props.info, props.thresholds, currentLanguage(), upperBound());
  return <TileGrid tiles={tiles()} classes={styles} />;
}
