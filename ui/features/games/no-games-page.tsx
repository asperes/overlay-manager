import { Gamepad2Icon, PlusIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { useConfig } from "@/features/config/config-provider";
import { useNavigation } from "@/features/shell/navigation";
import { PageBody, PageHeader, PageTitle } from "@/features/shell/page";

/** First-run page, shown while no games are set up. */
export function NoGamesPage() {
  const { actions } = useConfig();
  const { navigate } = useNavigation();

  return (
    <>
      <PageHeader>
        <PageTitle>Games</PageTitle>
      </PageHeader>
      <PageBody>
        <Empty className="border border-dashed">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <Gamepad2Icon />
            </EmptyMedia>
            <EmptyTitle>Set up your first game</EmptyTitle>
            <EmptyDescription>
              Pick a game and the overlays that go with it. They'll open when the game starts and close when it exits.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button onClick={() => navigate({ kind: "game", id: actions.addGame() })}>
              <PlusIcon data-icon="inline-start" />
              Add game
            </Button>
          </EmptyContent>
        </Empty>
      </PageBody>
    </>
  );
}
