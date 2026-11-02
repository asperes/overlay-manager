import { useState } from "react";
import { ChevronDownIcon, Trash2Icon } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupText } from "@/components/ui/input-group";
import { Item, ItemActions, ItemContent, ItemDescription, ItemMedia, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { ExeInput } from "@/components/exe-input";
import { ProgramIcon } from "@/components/program-icon";
import { useConfig } from "@/features/config/config-provider";
import { useProgramInfo } from "@/hooks/use-program-info";
import { fileName, isFullPath, shortPath } from "@/lib/paths";
import { cn } from "@/lib/utils";
import type { Overlay } from "@/lib/types";

interface OverlayItemProps {
  gameId: string;
  /** Position in the list, so Undo can put a removed overlay back in place. */
  index: number;
  overlay: Overlay;
}

/**
 * One overlay as a compact row (icon, name, on/off). The less common options
 * live in an expandable editor, which starts open for a brand-new overlay.
 */
export function OverlayItem({ gameId, index, overlay }: OverlayItemProps) {
  const { id } = overlay;
  const { actions } = useConfig();
  const [open, setOpen] = useState(() => overlay.path === "");
  const info = useProgramInfo(overlay.path);
  const missing = info !== null && !info.exists && isFullPath(overlay.path);
  const name = info?.name || (overlay.path ? fileName(overlay.path) : "New overlay");
  const update = (patch: Partial<Overlay>) => actions.updateOverlay(gameId, id, patch);

  const remove = () => {
    actions.removeOverlay(gameId, id);
    toast(`Removed ${name}`, {
      action: { label: "Undo", onClick: () => actions.restoreOverlay(gameId, overlay, index) },
    });
  };

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <Item variant="outline">
        <ItemMedia variant="image" className={cn("bg-muted", !overlay.enabled && "opacity-50")}>
          <ProgramIcon path={overlay.path} />
        </ItemMedia>
        <ItemContent className={cn("min-w-0", !overlay.enabled && "opacity-60")}>
          <ItemTitle>{name}</ItemTitle>
          <ItemDescription className={cn("truncate", missing && "text-destructive")} title={overlay.path}>
            {missing ? "Program not found" : overlay.path ? shortPath(overlay.path) : "No program chosen yet"}
          </ItemDescription>
        </ItemContent>
        <ItemActions className="shrink-0">
          <Switch
            checked={overlay.enabled}
            onCheckedChange={(enabled) => update({ enabled })}
            aria-label={`Start ${name} with the game`}
          />
          <CollapsibleTrigger asChild>
            <Button variant="ghost" size="icon" aria-label={open ? `Close ${name} options` : `Edit ${name}`}>
              <ChevronDownIcon className={cn("transition-transform", open && "rotate-180")} />
            </Button>
          </CollapsibleTrigger>
        </ItemActions>

        <CollapsibleContent className="basis-full">
          <FieldGroup className="gap-4 border-t pt-4">
            <Field data-invalid={missing || undefined}>
              <FieldLabel htmlFor={`${id}-path`}>Program</FieldLabel>
              <ExeInput
                id={`${id}-path`}
                value={overlay.path}
                onChange={(path) => update({ path })}
                placeholder="C:\Path\to\overlay.exe"
                dialogTitle="Choose an overlay program"
                invalid={missing}
              />
              {missing ? <FieldError>No program at this path.</FieldError> : null}
            </Field>

            <Field className="max-w-48">
              <FieldLabel htmlFor={`${id}-delay`}>Start delay</FieldLabel>
              <InputGroup>
                <InputGroupInput
                  id={`${id}-delay`}
                  type="number"
                  min={0}
                  step={500}
                  value={overlay.delayMs}
                  onChange={(e) => update({ delayMs: Math.max(0, Math.round(Number(e.target.value) || 0)) })}
                />
                <InputGroupAddon align="inline-end">
                  <InputGroupText>ms</InputGroupText>
                </InputGroupAddon>
              </InputGroup>
              <FieldDescription>Wait this long after the game starts.</FieldDescription>
            </Field>

            <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
              <CheckboxField
                id={`${id}-close`}
                label="Close when the game closes"
                checked={overlay.closeOnExit}
                onCheckedChange={(closeOnExit) => update({ closeOnExit })}
              />
              <CheckboxField
                id={`${id}-skip`}
                label="Don't start if already running"
                checked={overlay.skipIfRunning}
                onCheckedChange={(skipIfRunning) => update({ skipIfRunning })}
              />
              <Button variant="destructive" size="sm" className="ml-auto" onClick={remove}>
                <Trash2Icon data-icon="inline-start" />
                Remove
              </Button>
            </div>
          </FieldGroup>
        </CollapsibleContent>
      </Item>
    </Collapsible>
  );
}

interface CheckboxFieldProps {
  id: string;
  label: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}

function CheckboxField({ id, label, checked, onCheckedChange }: CheckboxFieldProps) {
  return (
    <Field orientation="horizontal" className="w-auto">
      <Checkbox id={id} checked={checked} onCheckedChange={(v) => onCheckedChange(v === true)} />
      <FieldLabel htmlFor={id} className="font-normal">
        {label}
      </FieldLabel>
    </Field>
  );
}
