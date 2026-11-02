import { useTransition } from "react";
import { PlayIcon } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { commands } from "@/lib/tauri";
import type { Overlay } from "@/lib/types";

/** Starts this game's overlays right now, to check the setup without the game. */
export function TestOverlaysButton({ overlays }: { overlays: Overlay[] }) {
  const [pending, startTransition] = useTransition();
  const runnable = overlays.filter((o) => o.enabled && o.path.trim());

  const test = () =>
    startTransition(async () => {
      try {
        const started = await commands.testOverlays(runnable);
        if (started === runnable.length) {
          toast.success(started === 1 ? "Started 1 overlay" : `Started ${started} overlays`);
        } else {
          toast.error(`Started ${started} of ${runnable.length} overlays`, {
            description: "See Activity for what went wrong.",
          });
        }
      } catch (e) {
        toast.error(String(e));
      }
    });

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        {/* Wrapper keeps the tooltip working while the button is disabled. */}
        <span tabIndex={runnable.length === 0 ? 0 : -1}>
          <Button variant="outline" onClick={test} disabled={pending || runnable.length === 0}>
            {pending ? <Spinner data-icon="inline-start" /> : <PlayIcon data-icon="inline-start" />}
            Test overlays
          </Button>
        </span>
      </TooltipTrigger>
      <TooltipContent>
        {runnable.length === 0 ? "Add an overlay first" : "Start the overlays now, without the game"}
      </TooltipContent>
    </Tooltip>
  );
}
