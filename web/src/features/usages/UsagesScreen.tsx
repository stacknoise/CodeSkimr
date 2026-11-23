import { createEffect, Match, on, Switch } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import type { Failure } from "../explorer/failure";
import { GraphView } from "./GraphView";
import type { Card } from "./graph";
import { ORIENTATION, SIDE } from "../../shared/layout/consts";
import { Splitter } from "../../shared/layout/Splitter";
import { HitList } from "./HitList";
import { filterHits } from "./hitText";
import { hitsHeight, panelWidth } from "./layout";
import { listedHits, listedPlace } from "./hits";
import { nextHit, useUsagesKeys } from "./keys";
import { moveUsages, originPlace, originSymbol } from "../../shared/navigation/navigation";
import { SidePanel } from "./SidePanel";
import { Toolbar } from "./Toolbar";
import type { Hit, UsagesAnswer } from "./types";
import {
  clearPreview,
  clearUsages,
  direction,
  focusSymbol,
  focusProject,
  hitCode,
  hitFilter,
  loadPreview,
  loadUsages,
  openInExplorer,
  openOriginInExplorer,
  openPlaceInExplorer,
  previewAnswer,
  selectedHit,
  setSelectedHit,
  testsShown,
  usagesAnswer,
  usagesFailure,
  usagesLoading,
  walkDepth,
} from "./usages";
import { NoticeBox } from "../../shared/ui/NoticeBox";
import styles from "./UsagesScreen.module.css";

function Message(props: {
  title?: string;
  text: string;
  alert?: boolean;
  code?: ErrorCode | null;
}) {
  return (
    <div class={styles.centered}>
      <NoticeBox class={styles.message} {...props} />
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
    if (symbol !== null && focusProject() !== project) {
      // The symbol was chosen in another project while this screen was closed.
      clearUsages();
    } else if (project !== null && symbol !== null) {
      void loadUsages(project, symbol);
    }
  });
}

/** Opens a symbol in the explorer. */
function useOpener(): (place: Hit["from"]) => void {
  return (place) => {
    const id = activeProject();
    if (id !== null) {
      void openInExplorer(id, place);
    }
  };
}

/** A click on a card: the side panel shows its symbol; a click on the focus shows the focus. */
function usePicker(): (card: Card) => void {
  return (card) => {
    const id = activeProject();
    if (card.focus || id === null) {
      clearPreview();
    } else {
      void loadPreview(id, card.place.id);
    }
  };
}

/** The way back: to the symbol the user came from, or to the focus if that is not known. */
function useBack(open: (place: Hit["from"]) => void, focus: Hit["from"]): () => void {
  return () => {
    const project = activeProject();
    const origin = originSymbol();
    const place = originPlace();
    if (place !== null) {
      openPlaceInExplorer(place);
    } else if (project !== null && origin !== null) {
      void openOriginInExplorer(project, origin);
    } else {
      open(focus);
    }
  };
}

/** The hits the list shows: the direct ones, narrowed to the symbol picked and to the filter. */
function useHits(answer: () => UsagesAnswer): () => Hit[] {
  return () =>
    filterHits(
      answer(),
      listedHits(answer(), previewAnswer()?.symbol.id ?? null),
      hitFilter(),
      hitCode(),
    );
}

function Body(props: { answer: UsagesAnswer }) {
  const hits = useHits(() => props.answer);
  const selectedPlace = () => {
    const hit = hits()[selectedHit()];
    return hit ? listedPlace(props.answer, hit) : undefined;
  };
  const open = useOpener();
  const pick = usePicker();
  const back = useBack(open, props.answer.symbol);
  useUsagesKeys({
    step: (step) => setSelectedHit(nextHit(selectedHit(), step, hits().length)),
    open: () => open(selectedPlace() ?? props.answer.symbol),
  });
  return (
    <div class={styles.screen}>
      <section class={styles.main}>
        <Toolbar onBack={back} />
        <GraphView
          answer={props.answer}
          picked={previewAnswer()?.symbol.id ?? null}
          marked={selectedPlace()?.id ?? null}
          onPick={pick}
          onOpen={(card) => !card.focus && moveUsages(card.place.id)}
        />
        <Splitter pane={hitsHeight} orientation={ORIENTATION.horizontal} side={SIDE.after} />
        <HitList
          answer={props.answer}
          hits={hits()}
          onOpen={(hit) => open(listedPlace(props.answer, hit))}
        />
      </section>
      <Splitter pane={panelWidth} orientation={ORIENTATION.vertical} side={SIDE.after} />
      <SidePanel
        answer={previewAnswer() ?? props.answer}
        onOpen={() => open((previewAnswer() ?? props.answer).symbol)}
      />
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
        {(failure) => <Message text={failureText(failure())} code={failure().code} alert />}
      </Match>
      <Match when={usagesAnswer()}>{(answer) => <Body answer={answer()} />}</Match>
      <Match when={usagesLoading()}>
        <Message text={t(I18N_KEYS.USAGES_LOADING)} />
      </Match>
    </Switch>
  );
}
