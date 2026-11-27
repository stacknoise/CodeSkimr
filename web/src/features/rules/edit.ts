import { KIND, LINE_BREAK, TOML_API_HEAD, TOML_FORBID_HEAD } from "./consts";
import type {
  ForbidInput,
  LimitInput,
  PublicApiInput,
  RuleOut,
  RulesInput,
  RulesOut,
} from "./types";

/** The entries of `[[rules.forbid]]` of an answer, in the order of the project file. */
export function forbidOf(data: RulesOut): ForbidInput[] {
  return data.rules
    .filter((rule) => rule.kind === KIND.forbid)
    .map((rule) =>
      rule.reason === null
        ? { from: rule.from, to: rule.to }
        : { from: rule.from, to: rule.to, reason: rule.reason },
    );
}

/** The entries of `[[rules.public_api]]` of an answer, in the order of the project file. */
export function publicApiOf(data: RulesOut): PublicApiInput[] {
  return data.rules
    .filter((rule) => rule.kind === KIND.publicApi)
    .map((rule) => ({ module: rule.from, internal: rule.internal }));
}

/** The rules of an answer as the input that writes them again, unchanged. */
export function inputOf(project: number, data: RulesOut): RulesInput {
  return {
    project,
    cycles: data.cycles,
    forbid: forbidOf(data),
    public_api: publicApiOf(data),
    limits: [],
  };
}

/** `list` with `entry` in place of the one at `index`, or at the end if `index` is `null`. */
export function put<T>(list: T[], index: number | null, entry: T): T[] {
  if (index === null) {
    return [...list, entry];
  }
  return list.map((old, at) => (at === index ? entry : old));
}

/** `list` without the entry at `index`. */
export function drop<T>(list: T[], index: number): T[] {
  return list.filter((_, at) => at !== index);
}

/** The position of `rule` among the rules of its kind in `data`; `null` for a rule that is no
 * entry of the project file (the layers, the cycle rule). */
export function indexOfRule(data: RulesOut, rule: RuleOut): number | null {
  if (rule.kind !== KIND.forbid && rule.kind !== KIND.publicApi) {
    return null;
  }
  const same = data.rules.filter((other) => other.kind === rule.kind);
  const at = same.findIndex((other) => other.id === rule.id);
  return at < 0 ? null : at;
}

/** The prefixes of a field, one per line; empty lines are dropped. */
export function linesOf(text: string): string[] {
  return text
    .split(LINE_BREAK)
    .map((line) => line.trim())
    .filter((line) => line !== "");
}

/** The text a forbid entry becomes in the file. */
export function forbidToml(entry: ForbidInput): string {
  const reason = entry.reason?.trim() ? [`reason = ${JSON.stringify(entry.reason.trim())}`] : [];
  return [
    TOML_FORBID_HEAD,
    `from = ${JSON.stringify(entry.from)}`,
    `to = ${JSON.stringify(entry.to)}`,
    ...reason,
  ].join(LINE_BREAK);
}

/** The text an entry of the public interface becomes in the file. */
export function publicApiToml(entry: PublicApiInput): string {
  return [
    TOML_API_HEAD,
    `module = ${JSON.stringify(entry.module)}`,
    `internal = ${JSON.stringify(entry.internal)}`,
  ].join(LINE_BREAK);
}

/** What `limits` say about each layer, as the matrix shows it: `null` is no limit. */
export function limitsOf(data: RulesOut, layers: string[]): LimitInput[] {
  const allowed = data.rules.find((rule) => rule.kind === KIND.layers)?.allowed ?? [];
  return layers.map((layer) => ({
    layer,
    may_use: allowed.find((entry) => entry.layer === layer)?.may_use ?? null,
  }));
}

/** The entry of a ban from the fields: trimmed, the reason only if there is one. */
export function forbidEntry(from: string, to: string, reason: string): ForbidInput {
  const why = reason.trim();
  return why === ""
    ? { from: from.trim(), to: to.trim() }
    : { from: from.trim(), to: to.trim(), reason: why };
}
