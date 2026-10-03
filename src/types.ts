// Mirrors `PortProcess` in src-tauri/src/model.rs.

export type ProcessType = "front" | "back";
/**
 * Who started the process. Agent ids come from rules/detection.toml, so any
 * other string is an agent added there.
 */
export type Origin = "claude-code" | "cursor" | "vscode" | "terminal" | "system" | "unknown" | (string & {});

export interface Worktree {
  root: string;
  branch: string | null;
  name: string;
}

export interface PortProcess {
  /** Identifies the process across restarts (worktree, port and name). */
  key: string;
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
  /** Display name for agents that only exist in the rules file. */
  originLabel: string | null;
  uptimeSeconds: number;
  pinned: boolean;
  canKill: boolean;
}
