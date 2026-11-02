import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { useTauriEvent } from "@/hooks/use-tauri-event";
import { commands } from "@/lib/tauri";

/** Pauses/resumes game detection. Stays in sync with the tray menu's toggle. */
export function WatchingToggle() {
  const [paused, setPaused] = useState(false);

  useTauriEvent("paused", setPaused);
  useEffect(() => {
    commands.getPaused().then(setPaused);
  }, []);

  return (
    <div className="flex items-center justify-between gap-2 rounded-md px-2 py-1.5">
      <Label htmlFor="watching" className="flex flex-col items-start gap-0.5">
        <span className="text-sm font-medium">{paused ? "Paused" : "Watching"}</span>
        <span className="text-xs font-normal text-muted-foreground">
          {paused ? "Overlays won't start" : "Overlays start with games"}
        </span>
      </Label>
      <Switch id="watching" checked={!paused} onCheckedChange={(on) => commands.setPaused(!on)} />
    </div>
  );
}
