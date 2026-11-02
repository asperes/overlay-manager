import type { ReactNode } from "react";
import { FolderOpenIcon } from "lucide-react";
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from "@/components/ui/input-group";
import { pickExecutable } from "@/lib/tauri";

interface ExeInputProps {
  id: string;
  value: string;
  onChange: (path: string) => void;
  placeholder: string;
  dialogTitle: string;
  invalid?: boolean;
  /** Extra buttons shown before Browse, e.g. <RunningProgramsMenu />. */
  children?: ReactNode;
}

/** Path field for a program, with a Browse button that opens a file picker. */
export function ExeInput({ id, value, onChange, placeholder, dialogTitle, invalid, children }: ExeInputProps) {
  const browse = async () => {
    const path = await pickExecutable(dialogTitle);
    if (path) onChange(path);
  };

  return (
    <InputGroup>
      <InputGroupInput
        id={id}
        value={value}
        placeholder={placeholder}
        spellCheck={false}
        aria-invalid={invalid || undefined}
        onChange={(e) => onChange(e.target.value)}
      />
      <InputGroupAddon align="inline-end">
        {children}
        <InputGroupButton onClick={browse}>
          <FolderOpenIcon data-icon="inline-start" />
          Browse
        </InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  );
}
