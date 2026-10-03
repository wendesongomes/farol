import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { messages as t } from "./i18n";
import { useProcesses } from "./hooks/useProcesses";
import { useFlash } from "./hooks/useFlash";
import { useStop } from "./hooks/useStop";
import { useActions } from "./hooks/useActions";
import { useArrowNavigation } from "./hooks/useArrowNavigation";
import { rowKey } from "./processKey";
import { groupProcesses } from "./group";
import { filterProcesses } from "./search";
import { ProcessRow } from "./components/ProcessRow";
import { GroupHeader } from "./components/GroupHeader";

export function App() {
  const { processes, refresh } = useProcesses();
  const { message, flash } = useFlash();
  const stopper = useStop(refresh, flash);
  const actions = useActions(refresh, flash);
  const [query, setQuery] = useState("");
  const searchRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLElement>(null);
  useArrowNavigation(listRef, searchRef);

  useEffect(() => {
    document.documentElement.lang = navigator.language;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") void getCurrentWindow().hide();
    };
    // Ready to type as soon as the panel opens.
    const onFocus = () => searchRef.current?.focus();
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("focus", onFocus);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("focus", onFocus);
    };
  }, []);

  const groups = useMemo(
    () => groupProcesses(filterProcesses(processes ?? [], query, t.origins)),
    [processes, query],
  );
  const count = processes?.length ?? 0;

  return (
    <main className="panel">
      <header className="panel-header">
        <input
          ref={searchRef}
          className="search"
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t.search}
          aria-label={t.search}
          spellCheck={false}
          autoFocus
        />
      </header>
      <section className="panel-body" ref={listRef}>
        {processes && count === 0 && <p className="empty">{t.empty}</p>}
        {processes && count > 0 && groups.length === 0 && <p className="empty">{t.noResults(query.trim())}</p>}
        {groups.map((group) => (
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
                  onToggleType={() => void actions.toggleType(p)}
                  onPrimary={() => void actions.primary(p)}
                  onTogglePin={() => void actions.togglePin(p)}
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
