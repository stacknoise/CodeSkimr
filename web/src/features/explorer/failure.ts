import { ApiError } from "../../shared/api/client";
import type { ErrorCode } from "../../shared/api/error-codes.generated";

/** Why a read failed: a registered code, or `null` if the server could not be reached. */
export type Failure = { code: ErrorCode | null };

/** The failure behind a caught error. */
export function failed(error: unknown): Failure {
  return { code: error instanceof ApiError ? error.code : null };
}
