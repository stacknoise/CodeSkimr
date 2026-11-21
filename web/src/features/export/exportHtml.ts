import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { activeProject } from "../../shared/project/active";
import { failed, projects, type Failure } from "../projects/projects";
import {
  EXPORT_COMMAND,
  EXPORT_EXTENSION,
  EXPORT_FALLBACK_NAME,
  EXPORT_LINK_TAG,
  EXPORT_MEDIA_TYPE,
  EXPORT_NAME_EDGES,
  EXPORT_NAME_MAX,
  EXPORT_NAME_FILL,
  EXPORT_NAME_UNSAFE,
  EXPORT_RESERVED_NAME,
  EXPORT_RESERVED_PREFIX,
  EXPORT_REVOKE_DELAY_MS,
} from "./consts";

/** The answer of `export.html` (`ExportOut` in crates/app/src/ui_commands/export.rs). */
interface Exported {
  html: string;
  symbols: number;
}

const [exporting, setExporting] = createSignal(false);
const [exportFailure, setExportFailure] = createSignal<Failure | null>(null);

/** True while a page is being made. */
export const isExporting = exporting;
/** The failure of the last export, `null` after a success or before the first try. */
export const lastExportFailure = exportFailure;

/** Forgets the failure of the last export, for example because another project or screen is shown. */
export function clearExportFailure(): void {
  setExportFailure(null);
}

/** True if the open project has a scan, so that an export can start. */
export function canExportActive(): boolean {
  const id = activeProject();
  return projects().some((project) => project.id === id && project.scan !== null);
}

/** The name of the downloaded file: the project name made safe for every file system. */
export function exportFileName(project: string): string {
  const cleaned = project
    .replace(EXPORT_NAME_UNSAFE, EXPORT_NAME_FILL)
    .slice(0, EXPORT_NAME_MAX)
    .replace(EXPORT_NAME_EDGES, "");
  const named = cleaned === "" ? EXPORT_FALLBACK_NAME : cleaned;
  const safe = EXPORT_RESERVED_NAME.test(named) ? `${EXPORT_RESERVED_PREFIX}${named}` : named;
  return `${safe}${EXPORT_EXTENSION}`;
}

/** Hands the page to the browser as a download. The file lands in the download folder of the
 * browser; nothing is sent anywhere. */
function save(html: string, fileName: string): void {
  const url = URL.createObjectURL(new Blob([html], { type: EXPORT_MEDIA_TYPE }));
  const link = document.createElement(EXPORT_LINK_TAG);
  link.href = url;
  link.download = fileName;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), EXPORT_REVOKE_DELAY_MS);
}

/** Exports the newest scan of a project as one HTML page and offers it as a download (blueprint
 * 10.2). Source text is not part of it. A second call while one runs does nothing. */
export async function exportProject(project: number, name: string): Promise<void> {
  if (exporting()) {
    return;
  }
  setExporting(true);
  setExportFailure(null);
  try {
    const page = await runCommand<Exported>(EXPORT_COMMAND, { project });
    save(page.html, exportFileName(name));
  } catch (error) {
    setExportFailure(failed(error));
  } finally {
    setExporting(false);
  }
}

/** The quick-search command: exports the open project, if it has a scan. */
export function exportActiveProject(): void {
  const id = activeProject();
  const row = projects().find((project) => project.id === id);
  if (row !== undefined && canExportActive()) {
    void exportProject(row.id, row.name);
  }
}
