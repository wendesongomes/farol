import type { PortProcess } from "../types";
import { messages as t } from "../i18n";

export function ProcessRow({ process }: { process: PortProcess }) {
  return (
    <li className="row">
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
    </li>
  );
}
