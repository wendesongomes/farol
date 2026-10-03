import { describe, expect, it } from "vitest";
import { filterProcesses } from "./search";
import type { PortProcess } from "./types";

const names = { "claude-code": "Claude Code", terminal: "terminal" };

function proc(port: number, extra: Partial<PortProcess> = {}): PortProcess {
  return {
    key: `none:${port}:node`,
    pid: port,
    port,
    name: "node",
    command: "node server.js",
    cwd: null,
    worktree: null,
    type: "back",
    typeSource: "detected",
    origin: "terminal",
    originLabel: null,
    uptimeSeconds: 0,
    pinned: false,
    canKill: true,
    ...extra,
  };
}

const list = [
  proc(3000, { command: "node vite", origin: "claude-code" }),
  proc(5432, { name: "postgres", command: "postgres -D /var/db", origin: "system" }),
  proc(8080, { worktree: { root: "/code/shop-checkout", branch: "feat/pay", name: "shop-checkout" } }),
];

const ports = (query: string) => filterProcesses(list, query, names).map((p) => p.port);

describe("filterProcesses", () => {
  it("returns everything for an empty query", () => {
    expect(ports("  ")).toEqual([3000, 5432, 8080]);
  });

  it("matches port, name and command", () => {
    expect(ports("543")).toEqual([5432]);
    expect(ports("POSTGRES")).toEqual([5432]);
    expect(ports("vite")).toEqual([3000]);
  });

  it("matches worktree name, path and branch", () => {
    expect(ports("checkout")).toEqual([8080]);
    expect(ports("feat/pay")).toEqual([8080]);
  });

  it("matches the origin by id and by display name", () => {
    expect(ports("claude code")).toEqual([3000]);
    expect(ports("claude-code")).toEqual([3000]);
  });

  it("requires every word", () => {
    expect(ports("node checkout")).toEqual([8080]);
    expect(ports("node postgres")).toEqual([]);
  });
});
