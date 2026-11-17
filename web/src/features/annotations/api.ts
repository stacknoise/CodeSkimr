import { runCommand } from "../../shared/api/commands";
import { COMMAND, KIND, MAX_PAGES, PAGE_SIZE } from "./consts";
import { rememberAuthor } from "./me";
import type { Annotation, Draft, Page, SummaryRow, Target } from "./types";

/** Reads every page of a list. `read` gets the offset of the page. */
async function allPages<Item>(read: (offset: number) => Promise<Page<Item>>): Promise<Item[]> {
  const items: Item[] = [];
  let offset: number | null = 0;
  for (let page = 0; offset !== null && page < MAX_PAGES; page += 1) {
    const answer: Page<Item> = await read(offset);
    items.push(...answer.items);
    offset = answer.next_offset;
  }
  return items;
}

/** The counters of every annotated symbol of the project: one call for each page of `PAGE_SIZE`
 * rows, never one for each symbol. */
export function readSummary(project: number): Promise<SummaryRow[]> {
  return allPages((offset) =>
    runCommand<Page<SummaryRow>>(COMMAND.summary, { project, offset, limit: PAGE_SIZE }),
  );
}

/** The annotations of one symbol. */
export function readSymbol(project: number, symbol: Target): Promise<Annotation[]> {
  return allPages((offset) =>
    runCommand<Page<Annotation>>(COMMAND.list, { project, symbol, offset, limit: PAGE_SIZE }),
  );
}

/** The notes and tasks of the whole project, for the text under the rows. */
export async function readTexts(project: number): Promise<Annotation[]> {
  const lists = await Promise.all(
    [KIND.note, KIND.task].map((kind) =>
      allPages((offset) =>
        runCommand<Page<Annotation>>(COMMAND.list, { project, kind, offset, limit: PAGE_SIZE }),
      ),
    ),
  );
  return lists.flat();
}

/** Remembers who the server named as author of what was made, then passes it on. */
async function made(work: Promise<Annotation>): Promise<Annotation> {
  const annotation = await work;
  if (annotation.author) {
    rememberAuthor(annotation.author);
  }
  return annotation;
}

/** All annotations of one kind of the whole project, for the board. */
export function readKind(project: number, kind: string): Promise<Annotation[]> {
  return allPages((offset) =>
    runCommand<Page<Annotation>>(COMMAND.list, { project, kind, offset, limit: PAGE_SIZE }),
  );
}

/** How many annotations of a kind the project has: a call for one row, which carries the total. */
export async function countKind(project: number, kind: string): Promise<number> {
  const page = await runCommand<Page<Annotation>>(COMMAND.list, { project, kind, limit: 1 });
  return page.total;
}

/** Marks a symbol as system critical; the reason is optional. */
export function markCritical(project: number, target: Target, text: string): Promise<Annotation> {
  return made(
    runCommand<Annotation>(COMMAND.markCritical, {
      project,
      target,
      ...(text === "" ? {} : { text }),
    }),
  );
}

/** Adds a note to a symbol. */
export function addNote(project: number, target: Target, text: string): Promise<Annotation> {
  return made(runCommand<Annotation>(COMMAND.addNote, { project, target, text }));
}

/** Adds a task to a symbol. */
export function addTask(project: number, target: Target, draft: Draft): Promise<Annotation> {
  const fields = Object.entries(draft).filter(([, value]) => value !== null && value !== undefined);
  return made(
    runCommand<Annotation>(COMMAND.addTask, {
      project,
      target,
      ...Object.fromEntries(fields),
    }),
  );
}

/** Changes an annotation. A field that is left out stays; `null` clears text, due date and
 * assignee. */
export function updateAnnotation(project: number, id: number, draft: Draft): Promise<Annotation> {
  return runCommand<Annotation>(COMMAND.update, { project, id, ...draft });
}

/** Deletes an annotation. */
export function deleteAnnotation(project: number, id: number): Promise<unknown> {
  return runCommand<unknown>(COMMAND.remove, { project, id });
}
