import { getToken, invalidateToken } from "../session/token";
import {
  BEARER_PREFIX,
  HEADER_AUTHORIZATION,
  HEADER_CONTENT_TYPE,
  HEADER_RETRY_AFTER,
  MEDIA_JSON,
  METHOD_GET,
  RETRY_AFTER_MAX_SECONDS,
  SECONDS_ONLY,
  STATUS_UNAUTHORIZED,
} from "./consts";
import { ERROR_CODES, type ErrorCode } from "./error-codes.generated";

/** A failed call of the local server: the registered code it answered with, if any. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: ErrorCode | null,
    /** Seconds the server asks to wait (`Retry-After`), `null` if it did not say or not in seconds. */
    readonly retryAfter: number | null = null,
  ) {
    super(code ?? String(status));
  }
}

/** Options of [`api`]. */
export interface ApiOptions {
  method?: string;
  /** JSON body. */
  body?: unknown;
  /** Send this token instead of the one of the session. */
  token?: string | null;
}

function knownCode(value: unknown): ErrorCode | null {
  return typeof value === "string" && value in ERROR_CODES ? (value as ErrorCode) : null;
}

/** The wait in seconds of a `Retry-After` header; a date or a text is not used. */
function retryAfterOf(response: Response): number | null {
  const value = response.headers.get(HEADER_RETRY_AFTER);
  if (value === null || !SECONDS_ONLY.test(value)) {
    return null;
  }
  const seconds = Number(value);
  return seconds > 0 && seconds <= RETRY_AFTER_MAX_SECONDS ? seconds : null;
}

async function codeOf(response: Response): Promise<ErrorCode | null> {
  try {
    const body: unknown = await response.json();
    return knownCode((body as { code?: unknown }).code);
  } catch {
    return null;
  }
}

/** Calls the local server and returns the JSON answer. The token of the session goes into the
 * `Authorization` header and nowhere else. A refused token ends the session (the UI then asks to
 * open CodeSkimr again).
 *
 * @throws ApiError for every answer that is not a success. A network failure is a `TypeError`.
 */
export async function api<T>(path: string, options: ApiOptions = {}): Promise<T> {
  const headers = new Headers();
  const token = options.token === undefined ? getToken() : options.token;
  if (token) {
    headers.set(HEADER_AUTHORIZATION, `${BEARER_PREFIX}${token}`);
  }
  if (options.body !== undefined) {
    headers.set(HEADER_CONTENT_TYPE, MEDIA_JSON);
  }
  const response = await fetch(path, {
    method: options.method ?? METHOD_GET,
    headers,
    ...(options.body === undefined ? {} : { body: JSON.stringify(options.body) }),
  });
  if (!response.ok) {
    if (response.status === STATUS_UNAUTHORIZED && token) {
      invalidateToken();
    }
    throw new ApiError(response.status, await codeOf(response), retryAfterOf(response));
  }
  return (await response.json()) as T;
}
