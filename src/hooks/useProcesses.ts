import { useCallback, useEffect, useState } from "react";
import { api } from "../api";
import type { PortProcess } from "../types";

const REFRESH_MS = 3000;

/** The list of listening processes, refreshed every few seconds. */
export function useProcesses() {
  const [processes, setProcesses] = useState<PortProcess[] | null>(null);

  /** Reloads the list now and returns it (null if it could not be read). */
  const refresh = useCallback(async (): Promise<PortProcess[] | null> => {
    try {
      const list = await api.listProcesses();
      setProcesses(list);
      return list;
    } catch (error) {
      console.error("list_processes failed", error);
      return null;
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- first load
    void refresh();
    const timer = window.setInterval(() => void refresh(), REFRESH_MS);
    return () => window.clearInterval(timer);
  }, [refresh]);

  return { processes, refresh };
}
