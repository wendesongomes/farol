import type { PortProcess } from "./types";

/**
 * Keeps the processes matching every word of the query, searching the port,
 * name, command, worktree (name, path, branch) and origin. Case-insensitive.
 */
export function filterProcesses(
  processes: PortProcess[],
  query: string,
  originNames: Record<string, string>,
): PortProcess[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return processes;
  return processes.filter((p) => {
    const haystack = [
      String(p.port),
      p.name,
      p.command,
      p.worktree?.name,
      p.worktree?.root,
      p.worktree?.branch,
      p.origin,
      originNames[p.origin],
      p.originLabel,
    ]
      .filter(Boolean)
      .join("\n")
      .toLowerCase();
    return words.every((word) => haystack.includes(word));
  });
}
