import { useEffect, useState } from "react";
import { commands } from "@/lib/tauri";
import type { ProgramInfo } from "@/lib/types";

// One lookup per path for the app's lifetime; icons and names don't change.
const cache = new Map<string, Promise<ProgramInfo>>();

function lookup(path: string): Promise<ProgramInfo> {
  let info = cache.get(path);
  if (!info) {
    info = commands.programInfo(path);
    cache.set(path, info);
  }
  return info;
}

/** Name, icon and existence of the program at `path`; null while loading or empty. */
export function useProgramInfo(path: string): ProgramInfo | null {
  const trimmed = path.trim();
  const [result, setResult] = useState<{ path: string; info: ProgramInfo } | null>(null);

  useEffect(() => {
    if (!trimmed) return;
    let current = true;
    lookup(trimmed).then((info) => {
      if (current) setResult({ path: trimmed, info });
    });
    return () => {
      current = false;
    };
  }, [trimmed]);

  // Ignore a result that belongs to a previous path.
  return trimmed && result?.path === trimmed ? result.info : null;
}
