import { Show } from "solid-js";
import { ALERT_ROLE } from "./consts";

/** A boxed note with an optional title; an alert is announced by screen readers at once. */
export function NoticeBox(props: {
  class?: string | undefined;
  title?: string;
  text: string;
  alert?: boolean;
}) {
  return (
    <div class={props.class} role={props.alert ? ALERT_ROLE : undefined}>
      <Show when={props.title}>{(title) => <h2>{title()}</h2>}</Show>
      <p>{props.text}</p>
    </div>
  );
}
