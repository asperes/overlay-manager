import { createContext, use, useState, type ReactNode } from "react";
import { useConfig } from "@/features/config/config-provider";

export type Page = { kind: "game"; id: string } | { kind: "activity" } | { kind: "settings" };

interface NavigationContextValue {
  page: Page | null;
  navigate: (page: Page) => void;
}

const NavigationContext = createContext<NavigationContextValue | null>(null);

export function useNavigation(): NavigationContextValue {
  const value = use(NavigationContext);
  if (!value) throw new Error("useNavigation must be used inside <NavigationProvider>");
  return value;
}

/**
 * Tracks which page is open. A game that no longer exists falls back to the
 * first game, or to nothing (the empty state) when there are none.
 */
export function NavigationProvider({ children }: { children: ReactNode }) {
  const { config } = useConfig();
  const [requested, navigate] = useState<Page | null>(null);

  const firstGame: Page | null = config.games[0] ? { kind: "game", id: config.games[0].id } : null;
  const isStale = requested?.kind === "game" && !config.games.some((p) => p.id === requested.id);
  const page = requested === null || isStale ? firstGame : requested;

  return <NavigationContext value={{ page, navigate }}>{children}</NavigationContext>;
}
