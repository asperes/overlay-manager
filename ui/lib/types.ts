// Mirrors the Rust types in src-tauri/src (serialized as camelCase).

export interface Overlay {
  id: string;
  enabled: boolean;
  path: string;
  delayMs: number;
  closeOnExit: boolean;
  skipIfRunning: boolean;
}

export interface Game {
  id: string;
  name: string;
  exe: string;
  enabled: boolean;
  overlays: Overlay[];
}

export interface Config {
  games: Game[];
  pollIntervalMs: number;
  autostart: boolean;
}

/** Present only for games that are running right now. */
export interface GameStatus {
  gameId: string;
  /** Overlays started for this session that are still tracked. */
  overlays: number;
}

export interface LogEntry {
  timeMs: number;
  level: "info" | "error";
  message: string;
}

export interface RunningProgram {
  name: string;
  exe: string | null;
}

export interface ProgramInfo {
  /** The exe's file description (e.g. "Discord"), or its file name. */
  name: string;
  /** PNG data URL; only available on Windows. */
  icon: string | null;
  exists: boolean;
}
