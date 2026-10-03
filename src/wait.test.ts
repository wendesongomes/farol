import { describe, expect, it } from "vitest";
import { waitUntil } from "./wait";

const noSleep = async () => {};

describe("waitUntil", () => {
  it("stops as soon as the check passes", async () => {
    let calls = 0;
    const ok = await waitUntil(async () => ++calls === 2, 3000, 500, noSleep);
    expect(ok).toBe(true);
    expect(calls).toBe(2);
  });

  it("gives up after the timeout", async () => {
    let calls = 0;
    const ok = await waitUntil(async () => (calls++, false), 3000, 500, noSleep);
    expect(ok).toBe(false);
    expect(calls).toBe(6);
  });
});
