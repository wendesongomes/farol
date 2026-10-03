/** Servers running longer than this are flagged as possibly forgotten. */
export const FORGOTTEN_AFTER_SECONDS = 24 * 60 * 60;

export const isForgotten = (uptimeSeconds: number) => uptimeSeconds > FORGOTTEN_AFTER_SECONDS;

/**
 * How long ago a process started, in the panel's language: "2 h ago",
 * "há 2 h". Uses the largest unit that fits, rounded down.
 */
export function formatUptime(seconds: number, language: string): string {
  const rtf = new Intl.RelativeTimeFormat(language, { numeric: "auto", style: "narrow" });
  const units: [Intl.RelativeTimeFormatUnit, number][] = [
    ["day", 86400],
    ["hour", 3600],
    ["minute", 60],
  ];
  for (const [unit, size] of units) {
    if (seconds >= size) return rtf.format(-Math.floor(seconds / size), unit);
  }
  return rtf.format(0, "second");
}
