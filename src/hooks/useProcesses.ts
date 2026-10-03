import { useCallback, useEffect, useState } from "react";
import { api } from "../api";
import type { PortProcess } from "../types";

const REFRESH_MS = 3000;

/**
 * The list of listening processes. Refreshed every 3 seconds while the panel
 * is open, and right away when it opens; nothing runs while it is closed
 * (the tray counter is updated by the Rust side on its own).
 */
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
    const timer = window.setInterval(() => {
      if (document.hasFocus()) void refresh();
    }, REFRESH_MS);
    const onFocus = () => void refresh();
    window.addEventListener("focus", onFocus);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  }, [refresh]);

  return { processes, refresh };
}
