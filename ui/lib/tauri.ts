// Every call into the Rust backend goes through this file.

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Config, LogEntry, Overlay, GameStatus, ProgramInfo, RunningProgram } from "@/lib/types";

export const commands = {
  getConfig: () => invoke<Config>("get_config"),
  saveConfig: (config: Config) => invoke<void>("save_config", { config }),
  getStatus: () => invoke<GameStatus[]>("get_status"),
  getLog: () => invoke<LogEntry[]>("get_log"),
  getPaused: () => invoke<boolean>("get_paused"),
  setPaused: (paused: boolean) => invoke<void>("set_paused", { paused }),
  listRunningPrograms: () => invoke<RunningProgram[]>("list_running_programs"),
  programInfo: (path: string) => invoke<ProgramInfo>("program_info", { path }),
  /** Starts the overlays now, without waiting for the game. Resolves to how many started. */
  testOverlays: (overlays: Overlay[]) => invoke<number>("test_overlays", { overlays }),
};

/** Events emitted by the backend, keyed by name. */
export interface EventPayloads {
  status: GameStatus[];
  log: LogEntry;
  paused: boolean;
  /** Sent when the config changed outside the window (e.g. from the tray menu). */
  config: Config;
}

/** Opens a file picker for a program. Resolves to null if cancelled. */
export async function pickExecutable(title: string): Promise<string | null> {
  const path = await open({
    title,
    multiple: false,
    directory: false,
    filters: [
      { name: "Programs", extensions: ["exe", "bat", "cmd"] },
      { name: "All files", extensions: ["*"] },
    ],
  });
  return typeof path === "string" ? path : null;
}
