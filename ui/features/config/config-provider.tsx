import { createContext, use, useEffect, useRef, useState, type ReactNode } from "react";
import { toast } from "sonner";
import { useTauriEvent } from "@/hooks/use-tauri-event";
import { commands } from "@/lib/tauri";
import type { Config, Overlay, Game } from "@/lib/types";

const SAVE_DEBOUNCE_MS = 400;

type Settings = Pick<Config, "autostart" | "pollIntervalMs">;
export type SaveState = "idle" | "saving" | "saved" | "error";

interface ConfigContextValue {
  config: Config;
  saveState: SaveState;
  actions: {
    /** Adds an empty game and returns its id. */
    addGame: () => string;
    updateGame: (id: string, patch: Partial<Game>) => void;
    removeGame: (id: string) => void;
    /** Puts a removed game back at its old position (for undo). */
    restoreGame: (game: Game, index: number) => void;
    addOverlay: (gameId: string) => void;
    updateOverlay: (gameId: string, overlayId: string, patch: Partial<Overlay>) => void;
    removeOverlay: (gameId: string, overlayId: string) => void;
    /** Puts a removed overlay back at its old position (for undo). */
    restoreOverlay: (gameId: string, overlay: Overlay, index: number) => void;
    updateSettings: (patch: Partial<Settings>) => void;
  };
}

const ConfigContext = createContext<ConfigContextValue | null>(null);

export function useConfig(): ConfigContextValue {
  const value = use(ConfigContext);
  if (!value) throw new Error("useConfig must be used inside <ConfigProvider>");
  return value;
}

function newOverlay(): Overlay {
  return { id: crypto.randomUUID(), enabled: true, path: "", delayMs: 0, closeOnExit: true, skipIfRunning: true };
}

/** Gives overlays saved before ids existed one, so they can be keyed in lists. */
function withOverlayIds(config: Config): Config {
  const needsIds = config.games.some((p) => p.overlays.some((o) => !o.id));
  if (!needsIds) return config;
  return {
    ...config,
    games: config.games.map((p) => ({
      ...p,
      overlays: p.overlays.map((o) => (o.id ? o : { ...o, id: crypto.randomUUID() })),
    })),
  };
}

function newGame(): Game {
  return { id: crypto.randomUUID(), name: "", exe: "", enabled: true, overlays: [] };
}

/**
 * Owns the config: loads it once, exposes edit actions, and saves (debounced)
 * after every edit. Children render only once the config has loaded.
 */
export function ConfigProvider({ children }: { children: ReactNode }) {
  const [config, setConfig] = useState<Config | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const edited = useRef(false);

  useEffect(() => {
    commands.getConfig().then((c) => setConfig(withOverlayIds(c)), (e) => toast.error(`Couldn't load settings: ${e}`));
  }, []);

  // Changed from the tray: adopt it as-is, it's already saved.
  useTauriEvent("config", (next) => {
    edited.current = false;
    setConfig(withOverlayIds(next));
  });

  useEffect(() => {
    if (!config || !edited.current) return;
    const timer = setTimeout(() => {
      commands.saveConfig(config).then(
        () => setSaveState("saved"),
        (e) => {
          setSaveState("error");
          toast.error(String(e));
        },
      );
    }, SAVE_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [config]);

  if (!config) return null;

  const edit = (fn: (c: Config) => Config) => {
    edited.current = true;
    setSaveState("saving");
    setConfig((prev) => (prev ? fn(prev) : prev));
  };
  const editGame = (id: string, fn: (g: Game) => Game) =>
    edit((c) => ({ ...c, games: c.games.map((p) => (p.id === id ? fn(p) : p)) }));

  const actions: ConfigContextValue["actions"] = {
    addGame: () => {
      const game = newGame();
      edit((c) => ({ ...c, games: [...c.games, game] }));
      return game.id;
    },
    updateGame: (id, patch) => editGame(id, (p) => ({ ...p, ...patch })),
    removeGame: (id) => edit((c) => ({ ...c, games: c.games.filter((p) => p.id !== id) })),
    restoreGame: (game, index) =>
      edit((c) => ({ ...c, games: c.games.toSpliced(index, 0, game) })),
    addOverlay: (gameId) => editGame(gameId, (p) => ({ ...p, overlays: [...p.overlays, newOverlay()] })),
    updateOverlay: (gameId, overlayId, patch) =>
      editGame(gameId, (p) => ({
        ...p,
        overlays: p.overlays.map((o) => (o.id === overlayId ? { ...o, ...patch } : o)),
      })),
    removeOverlay: (gameId, overlayId) =>
      editGame(gameId, (p) => ({ ...p, overlays: p.overlays.filter((o) => o.id !== overlayId) })),
    restoreOverlay: (gameId, overlay, index) =>
      editGame(gameId, (p) => ({ ...p, overlays: p.overlays.toSpliced(index, 0, overlay) })),
    updateSettings: (patch) => edit((c) => ({ ...c, ...patch })),
  };

  return <ConfigContext value={{ config, saveState, actions }}>{children}</ConfigContext>;
}
