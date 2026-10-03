import { useEffect, type RefObject } from "react";

/**
 * Arrow keys move focus between rows (`[data-row]`) inside `container`.
 * From the search field, ArrowDown jumps to the first row; ArrowUp on the
 * first row goes back to the search field.
 */
export function useArrowNavigation(container: RefObject<HTMLElement | null>, search: RefObject<HTMLInputElement | null>) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
      const rows = Array.from(container.current?.querySelectorAll<HTMLElement>("[data-row]") ?? []);
      if (rows.length === 0) return;

      const active = document.activeElement as HTMLElement | null;
      const current = active ? rows.findIndex((row) => row.contains(active)) : -1;
      const down = event.key === "ArrowDown";

      if (current === -1) {
        if (!down) return;
        rows[0].focus();
      } else if (!down && current === 0) {
        search.current?.focus();
      } else {
        rows[Math.min(rows.length - 1, Math.max(0, current + (down ? 1 : -1)))].focus();
      }
      event.preventDefault();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [container, search]);
}
