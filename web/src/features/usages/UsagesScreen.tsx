import { createEffect, Match, on, Show, Switch } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import type { Failure } from "../explorer/failure";
import { GraphView } from "./GraphView";
import { HitList } from "./HitList";
import { ROLE } from "./consts";
import { listedHits, listedPlace } from "./hits";
import { nextHit, useUsagesKeys } from "./keys";
import { SidePanel } from "./SidePanel";
import { Toolbar } from "./Toolbar";
import type { Hit, UsagesAnswer } from "./types";
import {
  clearUsages,
  direction,
  focusSymbol,
  loadUsages,
  openInExplorer,
  selectedHit,
  setSelectedHit,
  showUsages,
  testsShown,
  usagesAnswer,
  usagesFailure,
  usagesLoading,
  walkDepth,
} from "./usages";
import styles from "./UsagesScreen.module.css";

function Message(props: { title?: string; text: string; alert?: boolean }) {
  return (
    <div class={styles.centered}>
      <div class={styles.message} role={props.alert ? ROLE.alert : undefined}>
        <Show when={props.title}>{(title) => <h2>{title()}</h2>}</Show>
        <p>{props.text}</p>
      </div>
    </div>
  );
}

function failureText(failure: Failure): string {
  return failure.code === null
    ? t(I18N_KEYS.USAGES_LOAD_FAILED)
    : errorText(failure.code as ErrorCode);
}

/** Reads the usages again whenever the project, the symbol or a setting of the toolbar changes. */
function useReading(): void {
  createEffect(on(activeProject, () => clearUsages(), { defer: true }));
  createEffect(() => {
    const project = activeProject();
    const symbol = focusSymbol();
    direction();
    walkDepth();
    testsShown();
    if (project !== null && symbol !== null) {
      void loadUsages(project, symbol);
    }
  });
}

function Body(props: { answer: UsagesAnswer }) {
  const hits = () => listedHits(props.answer);
  const project = () => activeProject();
  const selectedPlace = () => {
    const hit = hits()[selectedHit()];
    return hit ? listedPlace(props.answer, hit) : undefined;
  };
  const open = (place: Hit["from"]) => {
    const id = project();
    if (id !== null) {
      void openInExplorer(id, place);
    }
  };
  useUsagesKeys({
    step: (step) => setSelectedHit(nextHit(selectedHit(), step, hits().length)),
    open: () => open(selectedPlace() ?? props.answer.symbol),
  });
  return (
    <div class={styles.screen}>
      <section class={styles.main}>
        <Toolbar />
        <GraphView
          answer={props.answer}
          onPick={(card) => !card.focus && showUsages(card.place.id)}
        />
        <HitList
          answer={props.answer}
          hits={hits()}
          onOpen={(hit) => open(listedPlace(props.answer, hit))}
        />
      </section>
      <SidePanel answer={props.answer} onOpen={() => open(props.answer.symbol)} />
    </div>
  );
}

/** Screen 6, the usages of one symbol: graph, hit list and side panel (blueprint B.6, mock
 * `Usages.dc.html`). */
export function UsagesScreen() {
  useReading();
  return (
    <Switch>
      <Match when={activeProject() === null}>
        <Message
          title={t(I18N_KEYS.USAGES_NO_PROJECT_TITLE)}
          text={t(I18N_KEYS.USAGES_NO_PROJECT_TEXT)}
        />
      </Match>
      <Match when={focusSymbol() === null}>
        <Message
          title={t(I18N_KEYS.USAGES_NO_SYMBOL_TITLE)}
          text={t(I18N_KEYS.USAGES_NO_SYMBOL_TEXT)}
        />
      </Match>
      <Match when={usagesFailure()}>
        {(failure) => <Message text={failureText(failure())} alert />}
      </Match>
      <Match when={usagesAnswer()}>{(answer) => <Body answer={answer()} />}</Match>
      <Match when={usagesLoading()}>
        <Message text={t(I18N_KEYS.USAGES_LOADING)} />
      </Match>
    </Switch>
  );
}
