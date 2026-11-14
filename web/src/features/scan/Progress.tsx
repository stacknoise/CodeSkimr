import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, seconds } from "../projects/format";
import { statusInfo } from "../status/status";
import { PERCENT_SIGN, PROGRESS_MAX, PROGRESS_MIN, STATE } from "./consts";
import { lastEventAt } from "./events";
import { files, filesPerSecond, percent, secondsLeft } from "./run";
import type { Run } from "./types";
import styles from "./Progress.module.css";

function Metric(props: { value: string; label: string }) {
  return (
    <div>
      <div class={styles.value}>{props.value}</div>
      <div class={styles.label}>{props.label}</div>
    </div>
  );
}

function rate(value: number | null): string {
  return value === null
    ? t(I18N_KEYS.SCAN_UNAVAILABLE)
    : count(Math.round(value), currentLanguage());
}

/** The caption under the percentage: files done and the time left, as far as measured. */
function caption(run: Run): string {
  const known = files(run);
  if (known === null) {
    return t(I18N_KEYS.SCAN_PENDING);
  }
  const language = currentLanguage();
  const counts = { done: count(known.done, language), total: count(known.total, language) };
  const left = secondsLeft(run, lastEventAt());
  return left === null
    ? t(I18N_KEYS.SCAN_FILES_DONE, counts)
    : t(I18N_KEYS.SCAN_PROGRESS, { ...counts, remaining: seconds(left, language) });
}

function Memory() {
  return (
    <Metric
      value={
        statusInfo()?.memory_mb == null
          ? t(I18N_KEYS.SCAN_UNAVAILABLE)
          : t(I18N_KEYS.SCAN_MEMORY_VALUE, { mb: statusInfo()?.memory_mb ?? 0 })
      }
      label={t(I18N_KEYS.SCAN_METRIC_MEMORY)}
    />
  );
}

/** The threads of the analyze stage while a scan runs; nothing is active otherwise. */
function Cores(props: { running: boolean }) {
  return (
    <Metric
      value={
        props.running && statusInfo()?.workers != null
          ? count(statusInfo()?.workers ?? 0, currentLanguage())
          : t(I18N_KEYS.SCAN_UNAVAILABLE)
      }
      label={t(I18N_KEYS.SCAN_METRIC_CORES)}
    />
  );
}

/** Percentage, progress bar and the measured rates of the scan (mock 5, top card). The lines per
 * second are known only once a full scan is over. */
export function Progress(props: { run: Run; linesPerSecond: number | null }) {
  const now = lastEventAt;
  return (
    <div class={styles.card}>
      <div class={styles.head}>
        <span class={styles.percent}>
          {percent(props.run)} {PERCENT_SIGN}
        </span>
        <span class={styles.caption}>{caption(props.run)}</span>
      </div>
      <div
        class={styles.track}
        role="progressbar"
        aria-valuemin={PROGRESS_MIN}
        aria-valuemax={PROGRESS_MAX}
        aria-valuenow={percent(props.run)}
      >
        <div class={styles.fill} style={{ width: percent(props.run) + PERCENT_SIGN }} />
      </div>
      <div class={styles.metrics}>
        <Metric
          value={rate(filesPerSecond(props.run, now()))}
          label={t(I18N_KEYS.SCAN_METRIC_FILES_PER_SECOND)}
        />
        <Metric
          value={rate(props.linesPerSecond)}
          label={t(I18N_KEYS.SCAN_METRIC_LINES_PER_SECOND)}
        />
        <Cores running={props.run.state === STATE.running} />
        <Memory />
      </div>
    </div>
  );
}
