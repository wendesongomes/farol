import { useCallback, useEffect, useRef, useState } from "react";

const FLASH_MS = 2500;

/** A short message shown in the footer after an action, then cleared. */
export function useFlash() {
  const [message, setMessage] = useState<string | null>(null);
  const timer = useRef<number | undefined>(undefined);

  const flash = useCallback((text: string) => {
    window.clearTimeout(timer.current);
    setMessage(text);
    timer.current = window.setTimeout(() => setMessage(null), FLASH_MS);
  }, []);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  return { message, flash };
}
