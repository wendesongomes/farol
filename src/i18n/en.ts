// Every user-facing string in the panel lives here. To add a language, copy
// this file, translate the values and register it in `index.ts`.
export const en = {
  appName: "Farol",
  empty: "No open ports. When a server starts, it shows up here.",
  otherUser: "other user",
  portCount: (n: number) => (n === 1 ? "1 open port" : `${n} open ports`),
};

export type Messages = typeof en;
