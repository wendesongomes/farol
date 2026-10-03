import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { ProcessRow } from "./ProcessRow";
import type { PortProcess } from "../types";

afterEach(cleanup);

const base: PortProcess = {
  key: "/code/web:3000:node",
  pid: 42,
  port: 3000,
  name: "node",
  command: "node server.js",
  cwd: "/code/web",
  worktree: null,
  type: "back",
  typeSource: "detected",
  origin: "unknown",
  uptimeSeconds: 120,
  pinned: false,
  canKill: true,
};

function renderRow(overrides: Partial<PortProcess> = {}) {
  const noop = vi.fn();
  return render(
    <ProcessRow
      process={{ ...base, ...overrides }}
      state="idle"
      onAskStop={noop}
      onCancel={noop}
      onStop={noop}
      onToggleType={noop}
      onPrimary={noop}
    />,
  );
}

describe("ProcessRow", () => {
  it("offers the browser for front-ends and a terminal for back-ends", () => {
    renderRow({ type: "front" });
    expect(screen.getByRole("button", { name: /open localhost:3000 in the browser/i })).toBeTruthy();
    cleanup();
    renderRow({ type: "back" });
    expect(screen.getByRole("button", { name: /open a terminal/i })).toBeTruthy();
  });

  it("switches the type when the label is clicked", () => {
    const onToggleType = vi.fn();
    render(
      <ProcessRow
        process={base}
        state="idle"
        onAskStop={vi.fn()}
        onCancel={vi.fn()}
        onStop={vi.fn()}
        onToggleType={onToggleType}
        onPrimary={vi.fn()}
      />,
    );
    screen.getByRole("button", { name: /detected as back-end/i }).click();
    expect(onToggleType).toHaveBeenCalledOnce();
  });

  it("flags servers running for more than a day", () => {
    renderRow({ uptimeSeconds: 2 * 86400 });
    expect(screen.getByText(/forgotten\?/)).toBeTruthy();
  });

  it("does not flag recent servers", () => {
    renderRow({ uptimeSeconds: 3600 });
    expect(screen.queryByText(/forgotten\?/)).toBeNull();
  });

  it("disables stopping without permission", () => {
    renderRow({ canKill: false });
    const button = screen.getByRole("button", { name: /stop the process on port 3000/i });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    expect(button.getAttribute("title")).toBe("No permission");
  });
});
