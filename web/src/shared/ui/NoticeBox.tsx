import { Show } from "solid-js";
import type { ErrorCode } from "../api/error-codes.generated";
import { ALERT_ROLE } from "./consts";
import { ErrorDetails } from "./ErrorDetails";

/** A boxed note with an optional title; an alert is announced by screen readers at once. A note
 * about a failure names its error `code`, with the button that copies the details. */
export function NoticeBox(props: {
  class?: string | undefined;
  title?: string;
  text: string;
  alert?: boolean;
  code?: ErrorCode | null | undefined;
}) {
  return (
    <div class={props.class} role={props.alert ? ALERT_ROLE : undefined}>
      <Show when={props.title}>{(title) => <h2>{title()}</h2>}</Show>
      <p>{props.text}</p>
      <Show when={props.code}>{(code) => <ErrorDetails code={code()} message={props.text} />}</Show>
    </div>
  );
}
