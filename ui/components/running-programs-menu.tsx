import { useState } from "react";
import { ListIcon } from "lucide-react";
import { toast } from "sonner";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { InputGroupButton } from "@/components/ui/input-group";
import { commands } from "@/lib/tauri";
import type { RunningProgram } from "@/lib/types";

/** Input-group button listing programs running right now. Loads the list when opened. */
export function RunningProgramsMenu({ onPick }: { onPick: (path: string) => void }) {
  const [apps, setApps] = useState<RunningProgram[] | null>(null);

  const load = (open: boolean) => {
    if (open) commands.listRunningPrograms().then(setApps, (e) => toast.error(String(e)));
  };

  return (
    <DropdownMenu onOpenChange={load}>
      <DropdownMenuTrigger asChild>
        <InputGroupButton>
          <ListIcon data-icon="inline-start" />
          Running
        </InputGroupButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="max-h-80 w-72">
        <DropdownMenuLabel>Pick a running program</DropdownMenuLabel>
        <DropdownMenuGroup>
          {apps === null ? (
            <DropdownMenuItem disabled>Loading…</DropdownMenuItem>
          ) : (
            apps.map((app) => (
              <DropdownMenuItem key={app.exe ?? app.name} onSelect={() => onPick(app.exe ?? app.name)}>
                <span className="truncate" title={app.exe ?? undefined}>
                  {app.name}
                </span>
              </DropdownMenuItem>
            ))
          )}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
