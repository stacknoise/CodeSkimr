import { createSignal, For, type JSX, Show } from "solid-js";
import { Portal } from "solid-js/web";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, type MessageKey, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { useKeydown } from "../../shared/keys/keydown";
import { useLayer } from "../../shared/keys/layers";
import { COPY, TONE } from "../../shared/ui/consts";
import { createCopier } from "../../shared/ui/copier";
import { useDialogFocus } from "../../shared/ui/dialogFocus";
import { KindBadge } from "../explorer/KindBadge";
import { CANCEL_KEY } from "../scan/consts";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import {
  ARROW,
  BADGE,
  REQUEST_INVALID,
  RENAME_SHOWN_ROWS,
  SIGNATURE_CLOSE,
  SIGNATURE_OPEN,
  RENAME_SUBMIT_KEY,
  RENAME_TITLE_ID,
  TEXT_KIND,
} from "./consts";
import { jsonText, listText, siteBadge, type SiteBadge } from "./format";
import {
  checkRename,
  closeRename,
  renameChecking,
  renameFailure,
  renamePreview,
  type RenameTarget,
} from "./rename";
import type { Conflict, Preview, Site, TextHit } from "./types";
import styles from "./RenameDialog.module.css";

const SITE_TEXT: Record<SiteBadge, [MessageKey, PillTone]> = {
  [BADGE.declaration]: [I18N_KEYS.RENAME_SITE_DECLARATION, TONE.accent],
  [BADGE.sure]: [I18N_KEYS.RENAME_SITE_SURE, TONE.neutral],
  [BADGE.probable]: [I18N_KEYS.RENAME_SITE_PROBABLE, TONE.warn],
  [BADGE.unchecked]: [I18N_KEYS.RENAME_SITE_UNCHECKED, TONE.warn],
};

const TEXT_TEXT: Record<TextHit["kind"], [MessageKey, PillTone]> = {
  [TEXT_KIND.documentation]: [I18N_KEYS.RENAME_TEXT_DOCUMENTATION, TONE.warn],
  [TEXT_KIND.configuration]: [I18N_KEYS.RENAME_TEXT_CONFIGURATION, TONE.warn],
  [TEXT_KIND.string]: [I18N_KEYS.RENAME_TEXT_STRING, TONE.warn],
  [TEXT_KIND.nameof]: [I18N_KEYS.RENAME_TEXT_NAMEOF, TONE.neutral],
  [TEXT_KIND.comment]: [I18N_KEYS.RENAME_TEXT_COMMENT, TONE.neutral],
  [TEXT_KIND.code]: [I18N_KEYS.RENAME_TEXT_CODE, TONE.warn],
};

/** One tile of the summary: a number, a note beside it and a caption. */
function Tile(props: { value: number; note?: string; caption: string }) {
  return (
    <div class={styles.tile}>
      <div>
        <span class={styles.value}>{props.value}</span>
        <Show when={props.note}>
          <span class={styles.note}>{props.note}</span>
        </Show>
      </div>
      <div class={styles.caption}>{props.caption}</div>
    </div>
  );
}

/** The four tiles: places, files and layers, conflicts and text hits. */
function Summary(props: { preview: Preview }) {
  const p = () => props.preview;
  return (
    <div class={styles.tiles}>
      <Tile
        value={p().sites_total}
        note={t(I18N_KEYS.RENAME_TILE_SURE, { count: p().verified })}
        caption={t(I18N_KEYS.RENAME_TILE_PLACES)}
      />
      <Tile
        value={p().files}
        note={t(I18N_KEYS.RENAME_TILE_LAYERS, { count: p().layers.length })}
        caption={t(I18N_KEYS.RENAME_TILE_FILES)}
      />
      <Tile
        value={p().conflicts.length}
        caption={t(I18N_KEYS.RENAME_TILE_CONFLICTS, { count: p().conflicts.length })}
      />
      <Tile
        value={p().text_total}
        note={t(I18N_KEYS.RENAME_TILE_CHECK_YOURSELF)}
        caption={t(I18N_KEYS.RENAME_TILE_TEXTS)}
      />
    </div>
  );
}

/** The symbol that has the new name, with its signature if the index knows one. */
function conflictName(conflict: Conflict): string {
  const name = conflict.qualified_name;
  return conflict.signature === null
    ? name
    : `${name}${SIGNATURE_OPEN}${conflict.signature}${SIGNATURE_CLOSE}`;
}

/** The sentence about a symbol that already has the new name. */
function ConflictNote(props: { conflict: Conflict }) {
  const key = () =>
    props.conflict.same_signature
      ? I18N_KEYS.RENAME_CONFLICT_SAME
      : I18N_KEYS.RENAME_CONFLICT_OTHER;
  return (
    <div class={styles.conflict} role="note">
      <strong>{t(I18N_KEYS.RENAME_CONFLICT_LABEL)}</strong>{" "}
      {t(key(), { name: conflictName(props.conflict) })}
    </div>
  );
}

/** The two banners: conflicts, and that the symbol is part of a public interface. */
function Banners(props: { preview: Preview }) {
  return (
    <>
      <For each={props.preview.conflicts}>{(conflict) => <ConflictNote conflict={conflict} />}</For>
      <Show when={props.preview.public_interface}>
        <div class={styles.public} role="note">
          <strong>{t(I18N_KEYS.RENAME_PUBLIC_LABEL)}</strong> {t(I18N_KEYS.RENAME_PUBLIC_TEXT)}
        </div>
      </Show>
    </>
  );
}

/** A row of a list: where, the line, the text and a badge. */
function Row(props: { where: string; line: number; text: string; badge: JSX.Element }) {
  return (
    <li class={styles.row}>
      <span class={styles.path}>{props.where}</span>
      <span class={styles.line}>{props.line}</span>
      <span class={styles.code}>{props.text}</span>
      {props.badge}
    </li>
  );
}

function Badge(props: { entry: [MessageKey, PillTone] }) {
  return (
    <Pill tone={props.entry[1]} compact>
      {t(props.entry[0])}
    </Pill>
  );
}

/** The places of the code, the first ones; the rest is a count. */
function Sites(props: { preview: Preview }) {
  const shown = () => props.preview.sites.slice(0, RENAME_SHOWN_ROWS);
  const more = () => props.preview.sites_total - shown().length;
  return (
    <section>
      <h3 class={styles.listHead}>
        {t(I18N_KEYS.RENAME_SITES, { total: props.preview.sites_total })}
      </h3>
      <ul class={styles.list}>
        <For each={shown()}>
          {(site: Site) => (
            <Row
              where={site.path}
              line={site.line}
              text={site.code ?? site.from?.qualified_name ?? props.preview.symbol.qualified_name}
              badge={<Badge entry={SITE_TEXT[siteBadge(site)]} />}
            />
          )}
        </For>
      </ul>
      <Show when={more() > 0}>
        <p class={styles.more}>{t(I18N_KEYS.RENAME_MORE, { count: more() })}</p>
      </Show>
    </section>
  );
}

/** The lines with the old name that no place explains, or why there are none to show. */
function Texts(props: { preview: Preview }) {
  const shown = () => props.preview.text_hits.slice(0, RENAME_SHOWN_ROWS);
  const more = () => props.preview.text_total - shown().length;
  return (
    <section>
      <h3 class={styles.listHead}>
        {t(I18N_KEYS.RENAME_TEXTS, { total: props.preview.text_total })}
      </h3>
      <Show
        when={props.preview.text_searched}
        fallback={<p class={styles.more}>{t(I18N_KEYS.RENAME_NOT_SEARCHED)}</p>}
      >
        <ul class={styles.list}>
          <For each={shown()}>
            {(hit) => (
              <Row
                where={hit.path}
                line={hit.line}
                text={hit.text}
                badge={<Badge entry={TEXT_TEXT[hit.kind]} />}
              />
            )}
          </For>
        </ul>
        <Show when={more() > 0}>
          <p class={styles.more}>{t(I18N_KEYS.RENAME_MORE, { count: more() })}</p>
        </Show>
        <Show when={props.preview.text_skipped > 0}>
          <p class={styles.more}>
            {t(I18N_KEYS.RENAME_SKIPPED, { count: props.preview.text_skipped })}
          </p>
        </Show>
      </Show>
    </section>
  );
}

/** The text of a failure: the new name was refused, no code at all, or the registered message. */
function failureText(code: ErrorCode | null): string {
  if (code === null) {
    return t(I18N_KEYS.RENAME_FAILED);
  }
  return code === REQUEST_INVALID ? t(I18N_KEYS.RENAME_INVALID_NAME) : errorText(code);
}

/** What the answer says, or why there is none yet. */
function Result(props: { preview: Preview | null }) {
  const failure = () => renameFailure();
  return (
    <div class={styles.result}>
      <Show when={failure()}>
        {(found) => (
          <p class={styles.error} role="alert">
            {failureText(found().code)}
          </p>
        )}
      </Show>
      <Show when={props.preview} fallback={<p class={styles.hint}>{t(I18N_KEYS.RENAME_HINT)}</p>}>
        {(preview) => (
          <>
            <Summary preview={preview()} />
            <Banners preview={preview()} />
            <Sites preview={preview()} />
            <Texts preview={preview()} />
          </>
        )}
      </Show>
    </div>
  );
}

/** The foot: the promise about the compiler, and the buttons that take the list away. */
function Foot(props: { preview: Preview | null }) {
  const list = createCopier();
  const json = createCopier();
  const label = (state: string | null, idle: MessageKey) => {
    if (state === COPY.copied) {
      return t(I18N_KEYS.RENAME_COPIED);
    }
    return state === COPY.failed ? t(I18N_KEYS.RENAME_COPY_FAILED) : t(idle);
  };
  return (
    <div class={styles.foot}>
      <span class={styles.promise}>{t(I18N_KEYS.RENAME_FOOTER)}</span>
      <button
        type="button"
        class={styles.secondary}
        disabled={props.preview === null}
        onClick={() => props.preview !== null && list.copy(listText(props.preview))}
      >
        {label(list.state(), I18N_KEYS.RENAME_COPY)}
      </button>
      <button
        type="button"
        class={styles.secondary}
        disabled={props.preview === null}
        onClick={() => props.preview !== null && json.copy(jsonText(props.preview))}
      >
        {label(json.state(), I18N_KEYS.RENAME_JSON)}
      </button>
      <button type="button" class={styles.primary} onClick={closeRename}>
        {t(I18N_KEYS.RENAME_CLOSE)}
      </button>
    </div>
  );
}

/** The state of level 2 for the answer: all places checked, or how many are preliminary. */
function Level(props: { preview: Preview | null }) {
  return (
    <Show when={props.preview}>
      {(preview) => (
        <Pill tone={preview().preliminary ? TONE.warn : TONE.good}>
          {preview().preliminary
            ? t(I18N_KEYS.RENAME_LEVEL1, { unchecked: preview().sites_total - preview().verified })
            : t(I18N_KEYS.RENAME_LEVEL2)}
        </Pill>
      )}
    </Show>
  );
}

/** The head: the title and the promise that nothing is changed. */
function Head() {
  return (
    <div class={styles.head}>
      <h2 id={RENAME_TITLE_ID} class={styles.title}>
        {t(I18N_KEYS.RENAME_TITLE)}
      </h2>
      <span class={styles.subtitle}>{t(I18N_KEYS.RENAME_SUBTITLE)}</span>
      <kbd class={styles.esc}>{t(I18N_KEYS.RENAME_ESC)}</kbd>
    </div>
  );
}

/** The row under the head: the old name, the field for the new one and the button that checks. */
function NameBar(props: {
  target: RenameTarget;
  name: string;
  onName: (name: string) => void;
  onRun: () => void;
  field: (element: HTMLInputElement) => void;
}) {
  const shown = () => renamePreview()?.symbol ?? props.target;
  return (
    <div class={styles.names}>
      <KindBadge kind={shown().kind} />
      <code class={styles.old}>{shown().qualified_name}</code>
      <span aria-hidden="true">{ARROW}</span>
      <input
        ref={props.field}
        class={styles.field}
        type="text"
        value={props.name}
        aria-label={t(I18N_KEYS.RENAME_NEW_NAME)}
        spellcheck={false}
        onInput={(event) => props.onName(event.currentTarget.value)}
        onKeyDown={(event) => event.key === RENAME_SUBMIT_KEY && props.onRun()}
      />
      <button
        type="button"
        class={styles.secondary}
        disabled={renameChecking() || props.name.trim() === ""}
        onClick={props.onRun}
      >
        {renameChecking() ? t(I18N_KEYS.RENAME_CHECKING) : t(I18N_KEYS.RENAME_CHECK)}
      </button>
      <Level preview={renamePreview()} />
    </div>
  );
}

/** The dialog "Check a rename" (mock of the extension E4, ADR 0110): the old name, a field for the
 * new one, and what the new name would hit. It changes nothing. Enter in the field runs the check,
 * Esc closes the dialog. The symbol shown is the one the answer names, before the first
 * check the one the explorer gave. */
export function RenameDialog(props: { target: RenameTarget }) {
  let dialog: HTMLDivElement | undefined;
  let field: HTMLInputElement | undefined;
  const [name, setName] = createSignal("");
  useLayer(() => true);
  useKeydown((event) => {
    if (event.key === CANCEL_KEY) {
      closeRename();
    }
  });
  useDialogFocus(
    () => dialog,
    () => field,
  );
  const run = () => void checkRename(name().trim());
  return (
    <Portal>
      <div class={styles.scrim} onClick={closeRename} />
      <div
        ref={dialog}
        class={styles.dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby={RENAME_TITLE_ID}
      >
        <Head />
        <NameBar
          target={props.target}
          name={name()}
          onName={setName}
          onRun={run}
          field={(el) => (field = el)}
        />
        <Result preview={renamePreview()} />
        <Foot preview={renamePreview()} />
      </div>
    </Portal>
  );
}
