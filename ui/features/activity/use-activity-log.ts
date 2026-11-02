import { useEffect, useState } from "react";
import { useTauriEvent } from "@/hooks/use-tauri-event";
import { commands } from "@/lib/tauri";
import type { LogEntry } from "@/lib/types";

/** Matches LOG_CAPACITY in src-tauri/src/lib.rs. */
const LOG_LIMIT = 200;

/** Recent backend activity, newest first. */
export function useActivityLog() {
  const [log, setLog] = useState<LogEntry[]>([]);

  useTauriEvent("log", (entry) => setLog((prev) => [entry, ...prev].slice(0, LOG_LIMIT)));
  useEffect(() => {
    commands.getLog().then((entries) => setLog(entries.toReversed()));
  }, []);

  return log;
}
