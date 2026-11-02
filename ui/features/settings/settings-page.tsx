import { Card, CardContent } from "@/components/ui/card";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel, FieldSeparator } from "@/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupText } from "@/components/ui/input-group";
import { Switch } from "@/components/ui/switch";
import { useConfig } from "@/features/config/config-provider";
import { PageBody, PageHeader, PageTitle } from "@/features/shell/page";

/** Matches MIN_POLL_MS in src-tauri/src/config.rs. */
const MIN_POLL_MS = 500;

export function SettingsPage() {
  const { config, actions } = useConfig();

  return (
    <>
      <PageHeader>
        <PageTitle>Settings</PageTitle>
      </PageHeader>
      <PageBody>
        <Card>
          <CardContent>
            <FieldGroup>
              <Field orientation="horizontal">
                <FieldContent>
                  <FieldLabel htmlFor="autostart">Start with Windows</FieldLabel>
                  <FieldDescription>Starts hidden in the tray when you log in.</FieldDescription>
                </FieldContent>
                <Switch
                  id="autostart"
                  checked={config.autostart}
                  onCheckedChange={(autostart) => actions.updateSettings({ autostart })}
                />
              </Field>
              <FieldSeparator />
              <Field orientation="horizontal">
                <FieldContent>
                  <FieldLabel htmlFor="poll">Check for games every</FieldLabel>
                  <FieldDescription>Lower reacts faster; higher uses a little less CPU.</FieldDescription>
                </FieldContent>
                <InputGroup className="w-32">
                  <InputGroupInput
                    id="poll"
                    type="number"
                    min={MIN_POLL_MS}
                    step={500}
                    value={config.pollIntervalMs}
                    onChange={(e) =>
                      actions.updateSettings({ pollIntervalMs: Math.round(Number(e.target.value) || 0) })
                    }
                    onBlur={() =>
                      actions.updateSettings({ pollIntervalMs: Math.max(MIN_POLL_MS, config.pollIntervalMs) })
                    }
                  />
                  <InputGroupAddon align="inline-end">
                    <InputGroupText>ms</InputGroupText>
                  </InputGroupAddon>
                </InputGroup>
              </Field>
            </FieldGroup>
          </CardContent>
        </Card>
        <p className="text-sm text-muted-foreground">
          Closing the window keeps Overlay Manager running in the tray. Use the tray icon to reopen it, pause
          watching, or quit.
        </p>
      </PageBody>
    </>
  );
}
