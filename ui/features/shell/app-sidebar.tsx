import { LayersIcon, PlusIcon, ScrollTextIcon, SettingsIcon } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarSeparator,
} from "@/components/ui/sidebar";
import { useConfig } from "@/features/config/config-provider";
import { GameStatusDot } from "@/features/games/game-status";
import { WatchingToggle } from "@/features/watching/watching-toggle";
import { gameTitle } from "@/lib/paths";
import { cn } from "@/lib/utils";
import { useNavigation } from "./navigation";

export function AppSidebar() {
  const { config, actions } = useConfig();
  const { page, navigate } = useNavigation();

  const addGame = () => navigate({ kind: "game", id: actions.addGame() });

  return (
    <Sidebar>
      <SidebarHeader>
        <div className="flex items-center gap-2 px-2 pt-1">
          <LayersIcon className="size-4 text-muted-foreground" aria-hidden />
          <span className="font-heading text-sm font-semibold">Overlay Manager</span>
        </div>
        <WatchingToggle />
      </SidebarHeader>
      <SidebarSeparator />

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Games</SidebarGroupLabel>
          <SidebarGroupAction title="Add game" onClick={addGame}>
            <PlusIcon />
            <span className="sr-only">Add game</span>
          </SidebarGroupAction>
          <SidebarGroupContent>
            <SidebarMenu>
              {config.games.map((game) => (
                <SidebarMenuItem key={game.id}>
                  <SidebarMenuButton
                    isActive={page?.kind === "game" && page.id === game.id}
                    onClick={() => navigate({ kind: "game", id: game.id })}
                  >
                    <GameStatusDot gameId={game.id} enabled={game.enabled} />
                    <span className={cn("truncate", !game.enabled && "text-muted-foreground")}>
                      {gameTitle(game)}
                    </span>
                  </SidebarMenuButton>
                  <SidebarMenuBadge aria-label={`${game.overlays.length} overlays`}>
                    {game.overlays.length}
                  </SidebarMenuBadge>
                </SidebarMenuItem>
              ))}
              <SidebarMenuItem>
                <SidebarMenuButton onClick={addGame} className="text-muted-foreground">
                  <PlusIcon />
                  <span>Add game</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton isActive={page?.kind === "activity"} onClick={() => navigate({ kind: "activity" })}>
              <ScrollTextIcon />
              <span>Activity</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton isActive={page?.kind === "settings"} onClick={() => navigate({ kind: "settings" })}>
              <SettingsIcon />
              <span>Settings</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
