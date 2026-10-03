/**
 * Calls `check` every `intervalMs` until it returns true or `timeoutMs`
 * passes. Resolves to whether it succeeded.
 */
export async function waitUntil(
  check: () => Promise<boolean>,
  timeoutMs: number,
  intervalMs: number,
  sleep: (ms: number) => Promise<void> = (ms) => new Promise((r) => setTimeout(r, ms)),
): Promise<boolean> {
  const attempts = Math.max(1, Math.ceil(timeoutMs / intervalMs));
  for (let i = 0; i < attempts; i++) {
    await sleep(intervalMs);
    if (await check()) return true;
  }
  return false;
}
