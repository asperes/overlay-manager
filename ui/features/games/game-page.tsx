import { useConfig } from "@/features/config/config-provider";
import { PageActions, PageBody, PageHeader, PageTitle } from "@/features/shell/page";
import { gameTitle } from "@/lib/paths";
import { GameSettingsCard } from "./game-settings-card";
import { OverlayList } from "./overlay-list";
import { GameActionsMenu } from "./game-actions-menu";
import { GameStatusBadge } from "./game-status";
import { TestOverlaysButton } from "./test-overlays-button";

/** The page for one game: its program, its overlays, and actions. */
export function GamePage({ gameId }: { gameId: string }) {
  const { config } = useConfig();
  const game = config.games.find((p) => p.id === gameId);
  if (!game) return null;

  return (
    <>
      <PageHeader>
        <PageTitle>
          <span className="truncate">{gameTitle(game)}</span>
          <GameStatusBadge gameId={game.id} enabled={game.enabled} />
        </PageTitle>
        <PageActions>
          <TestOverlaysButton overlays={game.overlays} />
          <GameActionsMenu game={game} />
        </PageActions>
      </PageHeader>
      <PageBody>
        <GameSettingsCard game={game} />
        <OverlayList gameId={game.id} overlays={game.overlays} />
      </PageBody>
    </>
  );
}
