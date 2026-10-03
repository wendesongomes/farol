import type { PortProcess } from "../types";
import { language, messages as t } from "../i18n";
import { formatUptime, isAgent, isForgotten, originName } from "../format";
import { displayName } from "../processKey";
import { ExternalLinkIcon, StarIcon, StopIcon, TerminalIcon } from "./icons";

export type RowState = "idle" | "confirm" | "stuck" | "busy";

interface Props {
  process: PortProcess;
  state: RowState;
  onAskStop: () => void;
  onCancel: () => void;
  onStop: (force: boolean) => void;
  onToggleType: () => void;
  onPrimary: () => void;
  onTogglePin: () => void;
}

export function ProcessRow(props: Props) {
  const { process, state, onAskStop, onCancel, onStop, onToggleType, onPrimary, onTogglePin } = props;
  const busy = state === "busy";
  const showConfirm = state === "confirm" || state === "stuck" || busy;
  const known = process.pid !== null;
  const typeLabel = process.type === "front" ? t.front : t.back;

  return (
    // Rows are reachable with the arrow keys (see useArrowNavigation).
    <li className="row" data-row tabIndex={-1} aria-label={`${process.port} ${process.command}`}>
      <div className="row-top">
        <div className="row-text">
          <div className="row-main">
            <span className="port" title={known ? `${process.name} · PID ${process.pid}` : undefined}>
              {process.port}
            </span>
            <span className="command" title={process.command}>
              {process.command || t.otherUser}
            </span>
          </div>
          {known && (
            <div className="meta">
              <button
                type="button"
                className="type-toggle"
                title={t.switchType(typeLabel)}
                aria-label={t.switchType(typeLabel)}
                onClick={onToggleType}
              >
                {typeLabel}
              </button>
              <span className={isAgent(process.origin) ? "origin agent" : "origin"}>
                {originName(process.origin, process.originLabel, t.origins)}
              </span>
              <Uptime seconds={process.uptimeSeconds} />
            </div>
          )}
        </div>
        <div className="actions">
          <PrimaryAction process={process} onClick={onPrimary} />
          <button
            type="button"
            className={process.pinned ? "icon-button pinned" : "icon-button"}
            aria-label={process.pinned ? t.unpin : t.pin}
            aria-pressed={process.pinned}
            title={process.pinned ? t.unpin : t.pin}
            onClick={onTogglePin}
          >
            <StarIcon filled={process.pinned} />
          </button>
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

/** Front-ends open in the browser; back-ends open a terminal in their folder. */
function PrimaryAction({ process, onClick }: { process: PortProcess; onClick: () => void }) {
  if (process.type === "front") {
    const label = t.openInBrowser(process.port);
    return (
      <button type="button" className="icon-button" aria-label={label} title={label} onClick={onClick}>
        <ExternalLinkIcon />
      </button>
    );
  }
  const label = process.cwd ? t.openTerminal : t.noFolder;
  return (
    <button
      type="button"
      className="icon-button"
      aria-label={t.openTerminal}
      title={label}
      disabled={!process.cwd}
      onClick={onClick}
    >
      <TerminalIcon />
    </button>
  );
}

function Uptime({ seconds }: { seconds: number }) {
  const text = formatUptime(seconds, language);
  if (!isForgotten(seconds)) return <span>{text}</span>;
  return (
    <span className="forgotten">
      {text} · {t.forgotten}
    </span>
  );
}
