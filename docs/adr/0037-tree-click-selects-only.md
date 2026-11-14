# 0037 – A click on an unselected type selects it and does not open it

Status: accepted

## Context

Blueprint 15.4 had two tree rules that cannot both hold (issue #140):

1. "A click on an unselected type selects and expands it."
2. "A selection from inside the tree never overrides the open/closed state."

The explorer (WP-45) follows rule 2 and has done so since the first release; the keyboard tests (WP-43) and the screenshots of the mocks rest on it. Rule 1 would make the first click change two things at once and would break rule 2 for the most common click.

## Decision

1. **The first click on an unselected type only selects it.** A click on the selected type opens or closes it; the caret only opens and closes; groups open and close on their own.
2. **Only external navigation opens nodes** (link, chip, graph, search, usages), as the last rule of 15.4 already says.
3. Blueprint 15.4 is changed to say so. The rule "selects and expands it" is removed.
4. **Accessibility of the tree** (the remarks of issue #140): a row of the virtualised tree carries `aria-level`, `aria-setsize` and `aria-posinset` computed from the whole tree, not from the rows in the window, so a screen reader announces the real position. Group rows stay expandable `treeitem`s; a flat tree with `aria-level` is the pattern the WAI-ARIA authoring guide gives for a virtualised tree, and `group` wrappers would need the rows of one parent to be neighbours in the DOM, which a window of rows cannot promise.

## Consequences

- No code change for the click rules; the deviation note in `docs/ui.md` is replaced by a pointer to this ADR.
- The Playwright keyboard test covers arrows and Enter on the tree (`web/e2e/ui-keyboard.spec.ts`).
