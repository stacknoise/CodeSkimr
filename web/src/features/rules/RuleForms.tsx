import { createSignal, For, type JSX } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { INSIDE_ROWS, KIND, LINE_BREAK, NAMESPACES_LIST_ID, SELF, TICK } from "./consts";
import {
  drop,
  forbidEntry,
  forbidOf,
  forbidToml,
  inputOf,
  limitsOf,
  linesOf,
  publicApiOf,
  publicApiToml,
  put,
} from "./edit";
import { editorNames } from "./editor";
import type { RuleOut, RulesInput, RulesOut } from "./types";
import styles from "./RuleDialog.module.css";

/** A form of the dialog: what it shows, the input it would write (`null` while it is not a valid
 * rule), and how to find the rule it makes in the answer of the preview. */
export interface Draft {
  body: JSX.Element;
  build: () => RulesInput | null;
  match?: (rule: RuleOut) => boolean;
}

/** The list the fields of a prefix offer: the namespaces of the scan. */
function Namespaces() {
  return (
    <datalist id={NAMESPACES_LIST_ID}>
      <For each={editorNames().namespaces}>{(space) => <option value={space.name} />}</For>
    </datalist>
  );
}

/** A labelled text field that offers the namespaces of the scan. */
function PrefixField(props: {
  label: string;
  value: string;
  onInput: (value: string) => void;
  hint?: boolean;
}) {
  return (
    <label class={styles.fld}>
      <span class={styles.label}>{props.label}</span>
      <input
        class={styles.input}
        type="text"
        list={NAMESPACES_LIST_ID}
        value={props.value}
        spellcheck={false}
        onInput={(event) => props.onInput(event.currentTarget.value)}
      />
      {props.hint === true && (
        <span class={styles.hint}>
          {t(I18N_KEYS.RULES_FIELD_HINT, { count: editorNames().namespaces.length })}
        </span>
      )}
    </label>
  );
}

/** The text the entry becomes in the file. */
function Toml(props: { text: string }) {
  return (
    <>
      <span class={styles.label}>{t(I18N_KEYS.RULES_TOML_LABEL)}</span>
      <pre class={styles.toml}>{props.text}</pre>
    </>
  );
}

/** The optional reason of a ban. */
function ReasonField(props: { value: string; onInput: (value: string) => void }) {
  return (
    <label class={styles.fld}>
      <span class={styles.label}>{t(I18N_KEYS.RULES_FIELD_REASON)}</span>
      <input
        class={styles.input}
        type="text"
        value={props.value}
        onInput={(event) => props.onInput(event.currentTarget.value)}
      />
    </label>
  );
}

/** The form of a ban: two modules and an optional reason. */
export function ForbidForm(project: number, data: RulesOut, index: number | null): Draft {
  const old = index === null ? undefined : forbidOf(data)[index];
  const [from, setFrom] = createSignal(old?.from ?? "");
  const [to, setTo] = createSignal(old?.to ?? "");
  const [reason, setReason] = createSignal(old?.reason ?? "");
  const entry = () => forbidEntry(from(), to(), reason());
  const build = () =>
    from().trim() === "" || to().trim() === ""
      ? null
      : { ...inputOf(project, data), forbid: put(forbidOf(data), index, entry()) };
  return {
    build,
    match: (rule) =>
      rule.kind === KIND.forbid && rule.from === from().trim() && rule.to === to().trim(),
    body: (
      <>
        <Namespaces />
        <div class={styles.two}>
          <PrefixField
            label={t(I18N_KEYS.RULES_FIELD_FROM)}
            value={from()}
            onInput={setFrom}
            hint
          />
          <PrefixField label={t(I18N_KEYS.RULES_FIELD_TO)} value={to()} onInput={setTo} />
        </div>
        <ReasonField value={reason()} onInput={setReason} />
        <Toml text={forbidToml(entry())} />
      </>
    ),
  };
}

/** The form of the public interface: one module and the prefixes of its inside. */
export function PublicApiForm(project: number, data: RulesOut, index: number | null): Draft {
  const old = index === null ? undefined : publicApiOf(data)[index];
  const [module, setModule] = createSignal(old?.module ?? "");
  const [inside, setInside] = createSignal((old?.internal ?? []).join(LINE_BREAK));
  const entry = () => ({ module: module().trim(), internal: linesOf(inside()) });
  const build = () =>
    entry().module === "" || entry().internal.length === 0
      ? null
      : { ...inputOf(project, data), public_api: put(publicApiOf(data), index, entry()) };
  return {
    build,
    match: (rule) => rule.kind === KIND.publicApi && rule.from === module().trim(),
    body: (
      <>
        <Namespaces />
        <PrefixField
          label={t(I18N_KEYS.RULES_FIELD_MODULE)}
          value={module()}
          onInput={setModule}
          hint
        />
        <label class={styles.fld}>
          <span class={styles.label}>{t(I18N_KEYS.RULES_FIELD_INTERNAL)}</span>
          <textarea
            class={styles.input}
            rows={INSIDE_ROWS}
            spellcheck={false}
            value={inside()}
            onInput={(event) => setInside(event.currentTarget.value)}
          />
        </label>
        <Toml text={publicApiToml(entry())} />
      </>
    ),
  };
}

/** What the matrix knows about one layer: whether it is limited and what it may use. */
type Row = {
  limited: () => boolean;
  uses: () => string[];
  toggle: () => void;
  flip: (name: string) => void;
};

function rowOf(old: string[] | null): Row {
  const [limited, setLimited] = createSignal(old !== null);
  const [uses, setUses] = createSignal(old ?? []);
  return {
    limited,
    uses,
    toggle: () => setLimited(!limited()),
    flip: (name) =>
      setUses((now) => (now.includes(name) ? now.filter((n) => n !== name) : [...now, name])),
  };
}

/** One row of the matrix: the layer, its limit and a tick for each layer it may use. */
function MatrixRow(props: { name: string; names: string[]; row: Row }) {
  return (
    <tr>
      <th scope="row">{props.name}</th>
      <td>
        <input
          type="checkbox"
          aria-label={`${props.name} ${t(I18N_KEYS.RULES_LAYERS_LIMIT)}`}
          checked={props.row.limited()}
          onChange={() => props.row.toggle()}
        />
      </td>
      <For each={props.names}>
        {(other) => (
          <td>
            {other === props.name ? (
              SELF
            ) : (
              <input
                type="checkbox"
                aria-label={`${props.name} ${TICK} ${other}`}
                disabled={!props.row.limited()}
                checked={props.row.uses().includes(other)}
                onChange={() => props.row.flip(other)}
              />
            )}
          </td>
        )}
      </For>
    </tr>
  );
}

/** The matrix: a row for each layer with its limit and a tick for each layer it may use. */
function Matrix(props: { names: string[]; rows: Row[] }) {
  return (
    <table class={styles.matrix}>
      <thead>
        <tr>
          <th>{t(I18N_KEYS.RULES_LAYERS_USES)}</th>
          <th>{t(I18N_KEYS.RULES_LAYERS_LIMIT)}</th>
          <For each={props.names}>{(name) => <th>{name}</th>}</For>
        </tr>
      </thead>
      <tbody>
        <For each={props.names}>
          {(name, at) => (
            <MatrixRow name={name} names={props.names} row={props.rows[at()] as Row} />
          )}
        </For>
      </tbody>
    </table>
  );
}

/** The form of the layer rule: what each layer may use. */
export function LayersForm(project: number, data: RulesOut): Draft {
  const names = editorNames().layers.map((layer) => layer.name);
  const rows = limitsOf(data, names).map((limit) => rowOf(limit.may_use));
  const build = () => ({
    ...inputOf(project, data),
    limits: names.map((layer, at) => ({
      layer,
      may_use: rows[at]?.limited() ? (rows[at]?.uses() ?? []).filter((use) => use !== layer) : null,
    })),
  });
  return {
    build: () => (names.length === 0 ? null : build()),
    match: (rule) => rule.kind === KIND.layers,
    body: (
      <>
        <p class={styles.help}>{t(I18N_KEYS.RULES_LAYERS_HELP)}</p>
        {names.length === 0 ? (
          <p class={styles.help}>{t(I18N_KEYS.RULES_LAYERS_NONE)}</p>
        ) : (
          <Matrix names={names} rows={rows} />
        )}
      </>
    ),
  };
}

/** The rule as the confirmation names it. */
function ruleName(rule: RuleOut | null): string {
  if (rule === null) {
    return "";
  }
  if (rule.kind === KIND.layers) {
    return t(I18N_KEYS.RULES_LAYERS_TITLE);
  }
  if (rule.kind === KIND.cycles) {
    return t(I18N_KEYS.RULES_CYCLES_TITLE);
  }
  return rule.kind === KIND.forbid
    ? t(I18N_KEYS.RULES_FORBID_TITLE, { from: rule.from, to: rule.to })
    : t(I18N_KEYS.RULES_PUBLIC_API_TITLE);
}

/** The input without `rule`: an entry is dropped, the cycle rule switched off, the limits of the
 * layers lifted. */
function without(project: number, data: RulesOut, rule: RuleOut, index: number | null): RulesInput {
  const base = inputOf(project, data);
  if (rule.kind === KIND.cycles) {
    return { ...base, cycles: false };
  }
  if (rule.kind === KIND.layers) {
    return {
      ...base,
      limits: limitsOf(
        data,
        editorNames().layers.map((layer) => layer.name),
      ).map((limit) => ({ ...limit, may_use: null })),
    };
  }
  if (index === null) {
    return base;
  }
  return rule.kind === KIND.forbid
    ? { ...base, forbid: drop(base.forbid, index) }
    : { ...base, public_api: drop(base.public_api, index) };
}

/** The confirmation of deleting a rule. */
export function RemoveBody(
  project: number,
  data: RulesOut,
  rule: RuleOut | null,
  index: number | null,
): Draft {
  return {
    build: () => (rule === null ? null : without(project, data, rule, index)),
    body: (
      <p class={styles.help}>{t(I18N_KEYS.RULES_DIALOG_DELETE_BODY, { rule: ruleName(rule) })}</p>
    ),
  };
}
