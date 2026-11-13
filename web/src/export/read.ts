import { DATA_ELEMENT_ID, DATA_FORMAT } from "./consts";
import type { ExportData } from "./types";

/** The data block of the page, or `null` if it is missing, is not JSON or is of another format. */
export function readData(root: Document = document): ExportData | null {
  const text = root.getElementById(DATA_ELEMENT_ID)?.textContent;
  if (!text) {
    return null;
  }
  try {
    const data = JSON.parse(text) as ExportData;
    return data.format === DATA_FORMAT ? data : null;
  } catch {
    return null;
  }
}
