import { Card, CardContent } from "@/components/ui/card";
import { Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { ExeInput } from "@/components/exe-input";
import { RunningProgramsMenu } from "@/components/running-programs-menu";
import { useConfig } from "@/features/config/config-provider";
import { useProgramInfo } from "@/hooks/use-program-info";
import { isFullPath } from "@/lib/paths";
import type { Game } from "@/lib/types";

export function GameSettingsCard({ game }: { game: Game }) {
  const { actions } = useConfig();
  const { id } = game;
  const info = useProgramInfo(game.exe);
  const missing = info !== null && !info.exists && isFullPath(game.exe);
  const setExe = (exe: string) => actions.updateGame(id, { exe });

  return (
    <Card>
      <CardContent>
        <FieldGroup>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel htmlFor={`${id}-enabled`}>Launch overlays for this game</FieldLabel>
              <FieldDescription>Turn off to keep the setup without using it.</FieldDescription>
            </FieldContent>
            <Switch
              id={`${id}-enabled`}
              checked={game.enabled}
              onCheckedChange={(enabled) => actions.updateGame(id, { enabled })}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor={`${id}-name`}>Name</FieldLabel>
            <Input
              id={`${id}-name`}
              value={game.name}
              placeholder={info?.name ?? "e.g. Elden Ring"}
              onChange={(e) => actions.updateGame(id, { name: e.target.value })}
            />
          </Field>
          <Field data-invalid={missing || undefined}>
            <FieldLabel htmlFor={`${id}-game`}>Game program</FieldLabel>
            <ExeInput
              id={`${id}-game`}
              value={game.exe}
              onChange={setExe}
              placeholder="C:\Games\MyGame\game.exe"
              dialogTitle="Choose the game's program"
              invalid={missing}
            >
              <RunningProgramsMenu onPick={setExe} />
            </ExeInput>
            {missing ? (
              <FieldError>No program at this path. Check it, or pick the game while it's running.</FieldError>
            ) : (
              <FieldDescription>
                When this program starts, its overlays start too. Already playing? Pick it from Running.
              </FieldDescription>
            )}
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}
