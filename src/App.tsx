import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { messages as t } from "./i18n";

export function App() {
  useEffect(() => {
    document.documentElement.lang = navigator.language;

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") void getCurrentWindow().hide();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return (
    <main className="panel">
      <header className="panel-header">
        <h1>{t.appName}</h1>
      </header>
      <section className="panel-body">
        <p className="empty">{t.empty}</p>
      </section>
      <footer className="panel-footer">{t.portCount(0)}</footer>
    </main>
  );
}
