import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { AnnotationPanel } from "../annotations/Panel";
import { annotationsAvailable } from "../annotations/store";
import { memberTarget, shownTarget, shownType } from "./annotate";
import { LOCATION_SEPARATOR, PANEL_WIDTH_PX } from "./consts";
import { lineRange } from "./format";
import { shownMember } from "./explorer";
import { sourceWidth } from "./layout";
import { SourcePane } from "./SourcePane";
import type { Target } from "../annotations/types";
import type { MemberRow, SymbolInfo } from "./types";
import styles from "./SidePanel.module.css";

/** One fact of the first tab. */
interface Fact {
  label: string;
  value: string;
}

/** What the first tab says about a selected member, whose source the middle column shows. */
function factsOf(info: SymbolInfo, member: MemberRow): Fact[] {
  const facts: (Fact | null)[] = [
    { label: t(I18N_KEYS.ANNOTATIONS_INFO_QUALIFIED), value: memberTarget(member).name },
    { label: t(I18N_KEYS.ANNOTATIONS_INFO_KIND), value: member.kind },
    member.signature === null
      ? null
      : { label: t(I18N_KEYS.ANNOTATIONS_INFO_SIGNATURE), value: member.signature },
    {
      label: t(I18N_KEYS.ANNOTATIONS_INFO_LOCATION),
      value: [info.path, lineRange(member.first_line, member.last_line)].join(LOCATION_SEPARATOR),
    },
    info.layer === null ? null : { label: t(I18N_KEYS.ANNOTATIONS_INFO_LAYER), value: info.layer },
  ];
  return facts.filter((fact): fact is Fact => fact !== null);
}

/** The first tab of a selected member (its source has the middle column). */
function MemberFacts(props: { info: SymbolInfo; member: MemberRow }) {
  return (
    <dl class={styles.facts}>
      <For each={factsOf(props.info, props.member)}>
        {(fact) => (
          <>
            <dt>{fact.label}</dt>
            <dd>{fact.value}</dd>
          </>
        )}
      </For>
    </dl>
  );
}

/** The panel of a shown symbol: the tab Info holds the source of a type, or the facts of a member
 * (whose source has the middle column). */
function Tabbed(props: { project: number; target: Target; info: SymbolInfo }) {
  return (
    <AnnotationPanel
      project={props.project}
      target={props.target}
      info={
        <Show when={shownMember()} fallback={<SourcePane shown wide={false} bare />}>
          {(row) => <MemberFacts info={props.info} member={row()} />}
        </Show>
      }
    />
  );
}

/** What the column holds: the tabs for a shown symbol, else the source column as it was (nothing
 * selected, or a server without annotations). */
function Content() {
  const project = () => activeProject();
  return (
    <Show
      when={annotationsAvailable() !== false && project() !== null && shownTarget()}
      fallback={<SourcePane shown={shownType() !== null} wide={false} bare />}
    >
      {(target) => (
        <Tabbed project={project() ?? 0} target={target()} info={shownType() as SymbolInfo} />
      )}
    </Show>
  );
}

/** The right column of the explorer (mock 14): the source of the shown type, or the facts of the
 * shown member, in the tab Info, and the tabs Critical, Notes and Tasks next to it. Without a
 * selection it is only the hint of the source column. Next to a selected member the source has
 * the middle column, so the panel has a width of its own and is left out where it would only be
 * facts (no annotations on the server). */
export function SidePanel() {
  const member = () => shownMember() !== null;
  return (
    <Show when={annotationsAvailable() !== false || !member()}>
      <section
        class={styles.column}
        style={{ "flex-basis": `${member() ? PANEL_WIDTH_PX : sourceWidth.size()}px` }}
        aria-label={t(I18N_KEYS.ANNOTATIONS_PANEL_LABEL)}
      >
        <Content />
      </section>
    </Show>
  );
}
