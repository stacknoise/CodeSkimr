import type { ErrorCode } from "../api/error-codes.generated";
import { DETAILS_SEPARATOR } from "./consts";
import { CopyDetails } from "./CopyDetails";
import styles from "./ErrorDetails.module.css";

/** The code of an error in small monospace type, and the button "Copy details" that puts the code
 * and the message on the clipboard, so that a report can quote them (blueprint 11.5). */
export function ErrorDetails(props: { code: ErrorCode; message: string }) {
  return (
    <div class={styles.details}>
      <code class={styles.code}>{props.code}</code>
      <CopyDetails text={`${props.code}${DETAILS_SEPARATOR}${props.message}`} />
    </div>
  );
}
