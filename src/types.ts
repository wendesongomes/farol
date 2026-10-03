// Mirrors `PortProcess` in src-tauri/src/model.rs.

export type ProcessType = "front" | "back";
export type Origin = "claude-code" | "cursor" | "vscode" | "terminal" | "system" | "unknown";

export interface Worktree {
  root: string;
  branch: string | null;
  name: string;
}

export interface PortProcess {
  /** null when the system does not say who owns the port (e.g. another user on Linux). */
  pid: number | null;
  port: number;
  name: string;
  command: string;
  cwd: string | null;
  worktree: Worktree | null;
  type: ProcessType;
  typeSource: "detected" | "manual";
  origin: Origin;
  uptimeSeconds: number;
  pinned: boolean;
  canKill: boolean;
}
