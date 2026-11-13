import { RATING } from "./consts";

/** The colour class for a rating. The caller passes its own CSS module classes. */
export function ratingClass(
  rating: string | null,
  classes: Readonly<Record<string, string | undefined>>,
): string | undefined {
  switch (rating) {
    case RATING.good:
      return classes.good;
    case RATING.medium:
      return classes.medium;
    case RATING.poor:
      return classes.poor;
    default:
      return undefined;
  }
}
