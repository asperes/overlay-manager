import { CheckIcon, TriangleAlertIcon } from "lucide-react";
import { Spinner } from "@/components/ui/spinner";
import { useConfig } from "@/features/config/config-provider";

/** Quiet confirmation that edits were autosaved. */
export function SaveIndicator() {
  const { saveState } = useConfig();

  return (
    <span aria-live="polite" className="flex items-center gap-1.5 text-xs text-muted-foreground">
      {saveState === "saving" ? (
        <>
          <Spinner className="size-3" />
          Saving…
        </>
      ) : saveState === "saved" ? (
        <>
          <CheckIcon className="size-3" />
          Saved
        </>
      ) : saveState === "error" ? (
        <span className="flex items-center gap-1.5 text-destructive">
          <TriangleAlertIcon className="size-3" />
          Not saved
        </span>
      ) : null}
    </span>
  );
}
