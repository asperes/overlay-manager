import { LayersIcon, PlusIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ItemGroup } from "@/components/ui/item";
import { useConfig } from "@/features/config/config-provider";
import type { Overlay } from "@/lib/types";
import { OverlayItem } from "./overlay-item";

interface OverlayListProps {
  gameId: string;
  overlays: Overlay[];
}

export function OverlayList({ gameId, overlays }: OverlayListProps) {
  const { actions } = useConfig();
  const add = () => actions.addOverlay(gameId);

  return (
    <section className="flex flex-col gap-3" aria-labelledby={`${gameId}-overlays`}>
      <div className="flex items-end justify-between gap-4">
        <div className="flex flex-col gap-1">
          <h2 id={`${gameId}-overlays`} className="font-heading text-sm font-semibold">
            Overlays
          </h2>
          <p className="text-sm text-muted-foreground">Programs that start with this game.</p>
        </div>
        {overlays.length > 0 ? (
          <Button variant="outline" size="sm" onClick={add}>
            <PlusIcon data-icon="inline-start" />
            Add overlay
          </Button>
        ) : null}
      </div>

      {overlays.length === 0 ? (
        <Empty className="border border-dashed">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <LayersIcon />
            </EmptyMedia>
            <EmptyTitle>No overlays yet</EmptyTitle>
            <EmptyDescription>Add the programs you want running while you play, like Discord or RTSS.</EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button onClick={add}>
              <PlusIcon data-icon="inline-start" />
              Add overlay
            </Button>
          </EmptyContent>
        </Empty>
      ) : (
        <ItemGroup>
          {overlays.map((overlay, index) => (
            <OverlayItem
              key={overlay.id}
              gameId={gameId}
              index={index}
              overlay={overlay}
            />
          ))}
        </ItemGroup>
      )}
    </section>
  );
}
