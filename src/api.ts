import { invoke } from "@tauri-apps/api/core";
import type { PortProcess } from "./types";

export const api = {
  listProcesses: () => invoke<PortProcess[]>("list_processes"),
};
