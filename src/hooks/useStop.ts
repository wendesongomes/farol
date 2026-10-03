import { useCallback, useState } from "react";
import { api, isCommandError } from "../api";
import { messages as t } from "../i18n";
import { rowKey } from "../processKey";
import type { PortProcess } from "../types";
import { waitUntil } from "../wait";
import type { RowState } from "../components/ProcessRow";

/** How long a process gets to free its port before "Force?" is offered. */
const GRACE_MS = 3000;
const POLL_MS = 500;

/**
 * The stop flow: confirm inline, ask the process to stop, wait for the port
 * to be freed and, if it isn't, offer to force it. Only one row at a time.
 */
export function useStop(
  refresh: () => Promise<PortProcess[] | null>,
  flash: (message: string) => void,
) {
  const [active, setActive] = useState<{ key: string; state: RowState } | null>(null);

  const stateOf = useCallback(
    (p: PortProcess): RowState => (active?.key === rowKey(p) ? active.state : "idle"),
    [active],
  );

  const ask = useCallback((p: PortProcess) => setActive({ key: rowKey(p), state: "confirm" }), []);
  const cancel = useCallback(() => setActive(null), []);

  const stop = useCallback(
    async (p: PortProcess, force: boolean) => {
      if (p.pid === null) return;
      const key = rowKey(p);
      setActive({ key, state: "busy" });

      try {
        await api.killProcess(p.pid, force);
      } catch (error) {
        setActive(null);
        const kind = isCommandError(error) ? error.kind : "other";
        flash(t.errors[kind] ?? t.errors.other);
        void refresh();
        return;
      }

      const freed = await waitUntil(
        async () => {
          const list = await refresh();
          return list !== null && !list.some((other) => rowKey(other) === key);
        },
        GRACE_MS,
        POLL_MS,
      );

      if (freed) {
        setActive(null);
        flash(t.portFreed(p.port));
      } else if (!force) {
        setActive({ key, state: "stuck" });
      } else {
        setActive(null);
        flash(t.stillRunning(p.port));
      }
    },
    [refresh, flash],
  );

  return { stateOf, ask, cancel, stop };
}
