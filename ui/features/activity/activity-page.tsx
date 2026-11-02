import { ScrollTextIcon } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { PageBody, PageHeader, PageTitle } from "@/features/shell/page";
import { cn } from "@/lib/utils";
import { useActivityLog } from "./use-activity-log";

const timeFormat = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit", second: "2-digit" });

export function ActivityPage() {
  const log = useActivityLog();

  return (
    <>
      <PageHeader>
        <PageTitle>Activity</PageTitle>
      </PageHeader>
      <PageBody>
        {log.length === 0 ? (
          <Empty className="border border-dashed">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <ScrollTextIcon />
              </EmptyMedia>
              <EmptyTitle>Nothing yet</EmptyTitle>
              <EmptyDescription>
                Games starting and closing, and overlays launching, show up here since Overlay Manager started.
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <Card>
            <CardContent>
              <ol className="flex flex-col gap-1.5 font-mono text-xs">
                {log.map((entry, i) => (
                  <li key={`${entry.timeMs}-${i}`} className="flex gap-3">
                    <time className="shrink-0 text-muted-foreground tabular-nums">{timeFormat.format(entry.timeMs)}</time>
                    <span className={cn(entry.level === "error" && "text-destructive")}>{entry.message}</span>
                  </li>
                ))}
              </ol>
            </CardContent>
          </Card>
        )}
      </PageBody>
    </>
  );
}
