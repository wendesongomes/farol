import { useCallback } from "react";
import { api, isCommandError } from "../api";
import { messages as t } from "../i18n";
import type { PortProcess } from "../types";

/** Opening, retyping and pinning: quick actions that report in the footer. */
export function useActions(refresh: () => Promise<unknown>, flash: (message: string) => void) {
  const report = useCallback(
    (error: unknown) => {
      const kind = isCommandError(error) ? error.kind : "other";
      flash(t.errors[kind] ?? t.errors.other);
    },
    [flash],
  );

  const primary = useCallback(
    async (p: PortProcess) => {
      try {
        if (p.type === "front") {
          flash(t.opening(p.port));
          await api.openInBrowser(p.port);
        } else if (p.cwd) {
          flash(t.openingTerminal);
          await api.openTerminal(p.cwd);
        }
      } catch (error) {
        report(error);
      }
    },
    [flash, report],
  );

  const toggleType = useCallback(
    async (p: PortProcess) => {
      try {
        await api.setTypeOverride(p.key, p.type === "front" ? "back" : "front");
        await refresh();
      } catch (error) {
        report(error);
      }
    },
    [refresh, report],
  );

  return { primary, toggleType };
}
