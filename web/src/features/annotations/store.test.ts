import { afterEach, describe, expect, it } from "vitest";
import { answer, fetchMock, serve, useServerSession } from "../../tests/commands";
import { callsOf, NOTE, page, SUMMARY } from "../../tests/annotations";
import { countersOf } from "./join";
import {
  annotationChanges,
  annotationIndex,
  annotationsAvailable,
  annotationTexts,
  change,
  forgetAnnotations,
  loadSummary,
  loadTexts,
} from "./store";

useServerSession();
afterEach(forgetAnnotations);

const ORDER = { kind: "class", name: "Shop.Domain.Order" };

describe("the summary", () => {
  it("is read once for a snapshot, however often a screen asks", async () => {
    serve({ "annotations.summary": () => answer(page(SUMMARY)) });
    await loadSummary(1, 5);
    await loadSummary(1, 5);
    await loadSummary(1, 5);
    expect(callsOf("annotations.summary")).toBe(1);
    expect(countersOf(annotationIndex(), ORDER)?.critical).toBe(true);
    expect(annotationsAvailable()).toBe(true);
  });

  it("is read again for a new snapshot and for another project", async () => {
    serve({ "annotations.summary": () => answer(page(SUMMARY)) });
    await loadSummary(1, 5);
    await loadSummary(1, 6);
    await loadSummary(2, 6);
    expect(callsOf("annotations.summary")).toBe(3);
  });

  it("shows nothing of another project while its own is read", async () => {
    serve({ "annotations.summary": () => answer(page(SUMMARY)) });
    await loadSummary(1, 5);
    const reading = loadSummary(2, 6);
    expect(annotationIndex().exact.size).toBe(0);
    await reading;
  });

  it("follows the pages until the server names no next one", async () => {
    let at = 0;
    fetchMock.mockImplementation(() => {
      const row = SUMMARY[at];
      at += 1;
      return Promise.resolve(answer(page(row ? [row] : [], at < SUMMARY.length ? at : null)));
    });
    await loadSummary(1, 5);
    expect(callsOf("annotations.summary")).toBe(SUMMARY.length);
    expect(annotationIndex().exact.size).toBe(SUMMARY.length);
  });

  it("marks the server as without annotations when the commands are missing", async () => {
    serve({});
    await loadSummary(1, 5);
    expect(annotationsAvailable()).toBe(false);
    expect(annotationIndex().exact.size).toBe(0);
  });

  it("reads nothing for a project that was never scanned", async () => {
    serve({ "annotations.summary": () => answer(page(SUMMARY)) });
    await loadSummary(1, null);
    expect(callsOf("annotations.summary")).toBe(0);
  });
});

describe("a change", () => {
  it("reads the summary again and tells the panels", async () => {
    serve({ "annotations.summary": () => answer(page(SUMMARY)) });
    await loadSummary(1, 5);
    const before = annotationChanges();
    expect(await change(1, () => Promise.resolve())).toBeNull();
    expect(callsOf("annotations.summary")).toBe(2);
    expect(annotationChanges()).toBe(before + 1);
  });

  it("returns the registered code of a refusal and reads the summary again", async () => {
    serve({
      "annotations.summary": () => answer(page(SUMMARY)),
      "annotations.mark_critical": () => answer({ code: "ANN-301" }, 409),
    });
    await loadSummary(1, 5);
    const { markCritical } = await import("./api");
    const failure = await change(1, () => markCritical(1, ORDER, ""));
    expect(failure).toEqual({ code: "ANN-301" });
    expect(callsOf("annotations.summary")).toBe(2);
  });

  it("returns no code for a server that cannot be reached", async () => {
    serve({ "annotations.summary": () => answer(page(SUMMARY)) });
    await loadSummary(1, 5);
    const failure = await change(1, () => Promise.reject(new TypeError("offline")));
    expect(failure).toEqual({ code: null });
  });
});

describe("the texts", () => {
  it("are read once for a snapshot and grouped by symbol", async () => {
    serve({
      "annotations.list": () => answer(page([NOTE])),
      "annotations.summary": () => answer(page(SUMMARY)),
    });
    await loadTexts(1, 5);
    await loadTexts(1, 5);
    // One list for the notes and one for the tasks, not one for each symbol.
    expect(callsOf("annotations.list")).toBe(2);
    expect(annotationTexts().get(["class", "Shop.Domain.Order"].join("\u0001"))?.length).toBe(2);
  });
});
