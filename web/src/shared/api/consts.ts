/** Route that exchanges the launch code for the token (ADR 0028). */
export const EXCHANGE_PATH = "/api/v1/session/exchange";
/** Route that answers whether the token is valid (and whether a launch code waits). */
export const SESSION_PATH = "/api/v1/session/launch-code";
/** Route of the settings the UI needs at start. */
export const UI_CONFIG_PATH = "/api/v1/ui-config";

/** HTTP methods. */
export const METHOD_GET = "GET";
export const METHOD_POST = "POST";

/** Header names and values. */
export const HEADER_AUTHORIZATION = "Authorization";
export const HEADER_CONTENT_TYPE = "Content-Type";
export const BEARER_PREFIX = "Bearer ";
export const MEDIA_JSON = "application/json";

/** Status of an answer that refuses the token. */
export const STATUS_UNAUTHORIZED = 401;
