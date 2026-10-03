import { invoke } from "@tauri-apps/api/core";
import type { PortProcess } from "./types";

/** Errors from Rust commands arrive as `{ kind, message }`. */
export interface CommandError {
  kind: string;
  message: string;
}

export function isCommandError(value: unknown): value is CommandError {
  return typeof value === "object" && value !== null && "kind" in value;
}

export const api = {
  listProcesses: () => invoke<PortProcess[]>("list_processes"),
  killProcess: (pid: number, force: boolean) => invoke<void>("kill_process", { pid, force }),
};
