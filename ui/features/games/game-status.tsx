import { Badge } from "@/components/ui/badge";
import { useGameStatus } from "@/features/status/status-provider";
import { cn } from "@/lib/utils";

interface GameStatusProps {
  gameId: string;
  enabled: boolean;
}

function useStatusLabel({ gameId, enabled }: GameStatusProps) {
  const status = useGameStatus(gameId);
  if (status) return { state: "running", label: "Running" } as const;
  if (!enabled) return { state: "off", label: "Off" } as const;
  return { state: "waiting", label: "Waiting for game" } as const;
}

/** Small colored dot for lists; the label is kept for screen readers. */
export function GameStatusDot(props: GameStatusProps) {
  const { state, label } = useStatusLabel(props);

  return (
    <span
      className={cn(
        "size-2 shrink-0 rounded-full",
        state === "running" && "bg-success",
        state === "waiting" && "bg-muted-foreground/50",
        state === "off" && "border border-muted-foreground/50",
      )}
    >
      <span className="sr-only">{label}</span>
    </span>
  );
}

export function GameStatusBadge(props: GameStatusProps) {
  const { state, label } = useStatusLabel(props);

  return (
    <Badge variant={state === "running" ? "default" : "secondary"} className="gap-1.5">
      <GameStatusDot {...props} />
      {label}
    </Badge>
  );
}
