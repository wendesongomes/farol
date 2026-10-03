import { invoke } from "@tauri-apps/api/core";
import type { PortProcess, ProcessType } from "./types";

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
  openInBrowser: (port: number) => invoke<void>("open_in_browser", { port }),
  openTerminal: (cwd: string) => invoke<void>("open_terminal", { cwd }),
  setTypeOverride: (key: string, kind: ProcessType) => invoke<void>("set_type_override", { key, kind }),
  togglePin: (key: string) => invoke<boolean>("toggle_pin", { key }),
};
