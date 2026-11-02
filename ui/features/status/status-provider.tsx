import { createContext, use, useEffect, useState, type ReactNode } from "react";
import { useTauriEvent } from "@/hooks/use-tauri-event";
import { commands } from "@/lib/tauri";
import type { GameStatus } from "@/lib/types";

type StatusMap = ReadonlyMap<string, GameStatus>;

const StatusContext = createContext<StatusMap>(new Map());

const toMap = (list: GameStatus[]): StatusMap => new Map(list.map((s) => [s.gameId, s]));

/** Shares one live subscription to "which games are running" with the whole app. */
export function StatusProvider({ children }: { children: ReactNode }) {
  const [statuses, setStatuses] = useState<StatusMap>(() => new Map());

  useTauriEvent("status", (list) => setStatuses(toMap(list)));
  useEffect(() => {
    commands.getStatus().then((list) => setStatuses(toMap(list)));
  }, []);

  return <StatusContext value={statuses}>{children}</StatusContext>;
}

/** Status of one game, or undefined when it isn't running. */
export function useGameStatus(gameId: string): GameStatus | undefined {
  return use(StatusContext).get(gameId);
}
