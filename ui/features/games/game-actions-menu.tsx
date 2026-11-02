import { EllipsisIcon, Trash2Icon } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useConfig } from "@/features/config/config-provider";
import { useNavigation } from "@/features/shell/navigation";
import { gameTitle } from "@/lib/paths";
import type { Game } from "@/lib/types";

/** "More" menu for a game. Deleting is instant, with Undo in the toast. */
export function GameActionsMenu({ game }: { game: Game }) {
  const { config, actions } = useConfig();
  const { navigate } = useNavigation();

  const remove = () => {
    const index = config.games.findIndex((p) => p.id === game.id);
    actions.removeGame(game.id);
    toast(`Deleted ${gameTitle(game)}`, {
      action: {
        label: "Undo",
        onClick: () => {
          actions.restoreGame(game, index);
          navigate({ kind: "game", id: game.id });
        },
      },
    });
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="More actions">
          <EllipsisIcon />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuGroup>
          <DropdownMenuItem variant="destructive" onSelect={remove}>
            <Trash2Icon />
            Delete game
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
