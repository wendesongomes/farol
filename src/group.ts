import type { PortProcess } from "./types";

export type GroupKind = "pinned" | "worktree" | "outside";

export interface Group {
  id: string;
  kind: GroupKind;
  /** Worktree name; empty for "pinned" and "outside", which the UI labels. */
  name: string;
  path: string | null;
  branch: string | null;
  processes: PortProcess[];
}

/**
 * Pinned processes first (if any), then one group per worktree sorted by
 * name, then everything outside a worktree. A pinned process only appears in
 * the pinned group.
 */
export function groupProcesses(processes: PortProcess[]): Group[] {
  const pinned = processes.filter((p) => p.pinned);
  const worktrees = new Map<string, Group>();
  const outside: PortProcess[] = [];

  for (const p of processes) {
    if (p.pinned) continue;
    if (!p.worktree) {
      outside.push(p);
      continue;
    }
    const { root, name, branch } = p.worktree;
    let group = worktrees.get(root);
    if (!group) {
      group = { id: `wt:${root}`, kind: "worktree", name, path: root, branch, processes: [] };
      worktrees.set(root, group);
    }
    group.processes.push(p);
  }

  const groups: Group[] = [];
  if (pinned.length > 0) {
    groups.push({ id: "pinned", kind: "pinned", name: "", path: null, branch: null, processes: pinned });
  }
  groups.push(
    ...[...worktrees.values()].sort(
      (a, b) => a.name.localeCompare(b.name) || (a.path ?? "").localeCompare(b.path ?? ""),
    ),
  );
  if (outside.length > 0) {
    groups.push({ id: "outside", kind: "outside", name: "", path: null, branch: null, processes: outside });
  }
  return groups;
}
