import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { ActivityPage } from "@/features/activity/activity-page";
import { NoGamesPage } from "@/features/games/no-games-page";
import { GamePage } from "@/features/games/game-page";
import { SettingsPage } from "@/features/settings/settings-page";
import { AppSidebar } from "./app-sidebar";
import { useNavigation } from "./navigation";

/** Sidebar on the left, the selected page on the right. */
export function AppShell() {
  return (
    <SidebarProvider>
      <AppSidebar />
      {/* min-w-0: let the page shrink to the window instead of growing to fit long paths. */}
      <SidebarInset className="min-w-0">
        <CurrentPage />
      </SidebarInset>
    </SidebarProvider>
  );
}

function CurrentPage() {
  const { page } = useNavigation();

  switch (page?.kind) {
    case "game":
      return <GamePage key={page.id} gameId={page.id} />;
    case "activity":
      return <ActivityPage />;
    case "settings":
      return <SettingsPage />;
    default:
      return <NoGamesPage />;
  }
}
