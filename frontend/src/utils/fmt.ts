/** Slices the first `n` characters of `str` and adds an ellipsis, defaulting to 50 characters. */
export const firstChars = (str: string, n = 50): string =>
  str.length > n ? str.slice(0, n) + '...' : str;

/**
 * Finds the amount of time that has elapsed between the provided time and now.
 *
 * @param ms - A date and time represented in milliseconds since the Unix epoch.
 * @returns A human readable representation of the amount of time elapsed.
 */
export const howLongAgo = (ms: number): string => {
  const now = new Date();
  const diffMs = now.getTime() - ms;

  const diffSeconds = Math.floor(diffMs / 1000);
  if (diffSeconds < 60) return `${diffSeconds.toString()} second${sUnless1(diffSeconds)}`;

  const diffMinutes = Math.floor(diffSeconds / 60);
  if (diffMinutes < 60) return `${diffMinutes.toString()} minute${sUnless1(diffMinutes)}`;

  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) return `${diffHours.toString()} hour${sUnless1(diffHours)}`;

  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays.toString()} day${sUnless1(diffDays)}`;
};

const sUnless1 = (x: number): string => (x === 1 ? '' : 's');
