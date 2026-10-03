import type { PortProcess } from "../types";
import { messages as t } from "../i18n";
import { displayName } from "../processKey";
import { StopIcon } from "./icons";

export type RowState = "idle" | "confirm" | "stuck" | "busy";

interface Props {
  process: PortProcess;
  state: RowState;
  onAskStop: () => void;
  onCancel: () => void;
  onStop: (force: boolean) => void;
}

export function ProcessRow({ process, state, onAskStop, onCancel, onStop }: Props) {
  const busy = state === "busy";
  const showConfirm = state === "confirm" || state === "stuck" || busy;

  return (
    <li className="row">
      <div className="row-top">
        <div className="row-text">
          <div className="row-main">
            <span className="port">{process.port}</span>
            <span className="command" title={process.command}>
              {process.command || t.otherUser}
            </span>
          </div>
          {process.pid !== null && (
            <div className="meta">
              {process.name} · {process.pid}
            </div>
          )}
        </div>
        <div className="actions">
          <button
            type="button"
            className="icon-button danger"
            aria-label={t.stopLabel(process.port)}
            title={process.canKill ? t.stopLabel(process.port) : t.noPermission}
            disabled={!process.canKill || showConfirm}
            onClick={onAskStop}
          >
            <StopIcon />
          </button>
        </div>
      </div>

      {showConfirm && (
        <div className="confirm" role="group" aria-label={t.confirmStop(displayName(process))}>
          <span className="confirm-text">
            {state === "stuck" ? t.didNotStop : t.confirmStop(displayName(process))}
          </span>
          <button type="button" className="button" onClick={onCancel} disabled={busy}>
            {t.cancel}
          </button>
          <button type="button" className="button danger-outline" onClick={() => onStop(true)} disabled={busy}>
            {t.force}
          </button>
          {state !== "stuck" && (
            <button
              type="button"
              className="button danger-solid"
              onClick={() => onStop(false)}
              disabled={busy}
              autoFocus
            >
              {t.stop}
            </button>
          )}
        </div>
      )}
    </li>
  );
}
