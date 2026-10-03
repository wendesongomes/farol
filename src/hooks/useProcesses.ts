import { useCallback, useEffect, useState } from "react";
import { api } from "../api";
import type { PortProcess } from "../types";

const REFRESH_MS = 3000;

/** The list of listening processes, refreshed every few seconds. */
export function useProcesses() {
  const [processes, setProcesses] = useState<PortProcess[] | null>(null);

  const refresh = useCallback(async () => {
    try {
      setProcesses(await api.listProcesses());
    } catch (error) {
      console.error("list_processes failed", error);
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
