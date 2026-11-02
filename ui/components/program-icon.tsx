import { AppWindowIcon } from "lucide-react";
import { useProgramInfo } from "@/hooks/use-program-info";

/** The program's own icon (Windows only), falling back to a generic app glyph. */
export function ProgramIcon({ path }: { path: string }) {
  const info = useProgramInfo(path);

  return info?.icon ? (
    <img src={info.icon} alt="" draggable={false} />
  ) : (
    <AppWindowIcon className="text-muted-foreground" aria-hidden />
  );
}
