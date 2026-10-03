import { describe, expect, it } from "vitest";
import { formatUptime, isAgent, isForgotten, originName } from "./format";

describe("formatUptime", () => {
  it("uses the largest unit that fits", () => {
    expect(formatUptime(30, "en")).toBe("now");
    // Exact wording comes from the platform's CLDR data, so only the
    // number and unit are checked.
    expect(formatUptime(5 * 60, "en")).toMatch(/^5\s?m/);
    expect(formatUptime(2 * 3600 + 59 * 60, "en")).toMatch(/^2\s?h/);
    expect(formatUptime(3 * 86400, "en")).toMatch(/^3\s?d/);
  });

  it("speaks Portuguese", () => {
    expect(formatUptime(2 * 3600, "pt-BR")).toMatch(/^há 2\s?h/);
  });
});

describe("isForgotten", () => {
  it("flags servers running for more than 24 hours", () => {
    expect(isForgotten(24 * 3600)).toBe(false);
    expect(isForgotten(24 * 3600 + 1)).toBe(true);
  });
});

describe("origin", () => {
  it("highlights agents only", () => {
    expect(isAgent("claude-code")).toBe(true);
    expect(isAgent("windsurf")).toBe(true);
    expect(isAgent("terminal")).toBe(false);
    expect(isAgent("system")).toBe(false);
    expect(isAgent("unknown")).toBe(false);
  });

  it("names agents from the rules file by their label", () => {
    const names = { "claude-code": "Claude Code" };
    expect(originName("claude-code", null, names)).toBe("Claude Code");
    expect(originName("windsurf", "Windsurf", names)).toBe("Windsurf");
    expect(originName("zed", null, names)).toBe("zed");
  });
});
