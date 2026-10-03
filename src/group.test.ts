import { describe, expect, it } from "vitest";
import { groupProcesses } from "./group";
import type { PortProcess } from "./types";

function proc(port: number, root: string | null, pinned = false): PortProcess {
  return {
    key: `${root ?? "none"}:${port}:node`,
    pid: port,
    port,
    name: "node",
    command: "node",
    cwd: root,
    worktree: root ? { root, branch: "main", name: root.split("/").pop()! } : null,
    type: "back",
    typeSource: "detected",
    origin: "unknown",
    uptimeSeconds: 0,
    pinned,
    canKill: true,
  };
}

describe("groupProcesses", () => {
  it("orders pinned, worktrees by name, then outside", () => {
    const groups = groupProcesses([
      proc(1, null),
      proc(2, "/code/web"),
      proc(3, "/code/api"),
      proc(4, "/code/web"),
      proc(5, "/code/api", true),
    ]);
    expect(groups.map((g) => g.kind)).toEqual(["pinned", "worktree", "worktree", "outside"]);
    expect(groups.map((g) => g.name)).toEqual(["", "api", "web", ""]);
    expect(groups[0].processes.map((p) => p.port)).toEqual([5]);
    expect(groups[1].processes.map((p) => p.port)).toEqual([3]);
    expect(groups[2].processes.map((p) => p.port)).toEqual([2, 4]);
  });

  it("separates worktrees that share a folder name", () => {
    const groups = groupProcesses([proc(1, "/a/app"), proc(2, "/b/app")]);
    expect(groups).toHaveLength(2);
    expect(groups.map((g) => g.path)).toEqual(["/a/app", "/b/app"]);
  });

  it("has no pinned or outside group when empty", () => {
    expect(groupProcesses([proc(1, "/code/web")]).map((g) => g.kind)).toEqual(["worktree"]);
    expect(groupProcesses([])).toEqual([]);
  });
});
