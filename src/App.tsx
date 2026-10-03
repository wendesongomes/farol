import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { messages as t } from "./i18n";
import { useProcesses } from "./hooks/useProcesses";
import { useFlash } from "./hooks/useFlash";
import { useStop } from "./hooks/useStop";
import { rowKey } from "./processKey";
import { ProcessRow } from "./components/ProcessRow";
import { GroupHeader } from "./components/GroupHeader";
import { groupProcesses } from "./group";

export function App() {
  const { processes, refresh } = useProcesses();
  const { message, flash } = useFlash();
  const stopper = useStop(refresh, flash);

  useEffect(() => {
    document.documentElement.lang = navigator.language;

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") void getCurrentWindow().hide();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const count = processes?.length ?? 0;

  return (
    <main className="panel">
      <header className="panel-header">
        <h1>{t.appName}</h1>
      </header>
      <section className="panel-body">
        {processes && processes.length === 0 && <p className="empty">{t.empty}</p>}
        {processes &&
          groupProcesses(processes).map((group) => (
            <section key={group.id} className="group" aria-label={group.name || undefined}>
              <GroupHeader group={group} />
              <ul className="list">
                {group.processes.map((p) => (
                  <ProcessRow
                    key={rowKey(p)}
                    process={p}
                    state={stopper.stateOf(p)}
                    onAskStop={() => stopper.ask(p)}
                    onCancel={stopper.cancel}
                    onStop={(force) => void stopper.stop(p, force)}
                  />
                ))}
              </ul>
            </section>
          ))}
      </section>
      <footer className="panel-footer" aria-live="polite">
        {message ?? t.portCount(count)}
      </footer>
    </main>
  );
}
