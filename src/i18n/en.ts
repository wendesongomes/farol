// Every user-facing string in the panel lives here. To add a language, copy
// this file, translate the values and register it in `index.ts`.
export const en = {
  appName: "Farol",
  empty: "No open ports. When a server starts, it shows up here.",
  otherUser: "other user",
  pinnedGroup: "Pinned",
  outsideGroup: "Outside a worktree",
  portCount: (n: number) => (n === 1 ? "1 open port" : `${n} open ports`),

  // Stopping a process
  stopLabel: (port: number) => `Stop the process on port ${port}`,
  noPermission: "No permission",
  confirmStop: (name: string) => `Stop ${name}?`,
  cancel: "Cancel",
  force: "Force",
  stop: "Stop",
  didNotStop: "Didn't stop. Force?",
  portFreed: (port: number) => `Port ${port} freed`,
  stillRunning: (port: number) => `Port ${port} is still in use`,

  // Errors, by `kind` (see PlatformError in src-tauri/src/platform/mod.rs)
  errors: {
    "not-found": "The process no longer exists",
    "permission-denied": "No permission",
    "no-terminal": "No terminal app found",
    "not-implemented": "Not available on this system yet",
    other: "Something went wrong",
  } as Record<string, string>,
};

export type Messages = typeof en;
