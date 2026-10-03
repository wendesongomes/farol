import type { PortProcess } from "./types";

/** Identifies a row while it is on screen. */
export const rowKey = (p: Pick<PortProcess, "pid" | "port">) => `${p.pid ?? "?"}:${p.port}`;

/** What to call a process in messages ("Stop node?"). */
export const displayName = (p: Pick<PortProcess, "name" | "port">) => p.name || `:${p.port}`;
