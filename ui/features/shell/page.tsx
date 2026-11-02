import type { ReactNode } from "react";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { SaveIndicator } from "@/components/save-indicator";
import { ThemeToggle } from "@/components/theme-toggle";

/** Top bar of a page: title on the left, page actions, then save state and theme. */
export function PageHeader({ children }: { children: ReactNode }) {
  return (
    <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-3 border-b bg-background/80 px-4 backdrop-blur">
      <SidebarTrigger className="md:hidden" />
      {children}
      <div className="ml-auto flex shrink-0 items-center gap-3">
        <SaveIndicator />
        <ThemeToggle />
      </div>
    </header>
  );
}

export function PageTitle({ children }: { children: ReactNode }) {
  return <h1 className="flex min-w-0 items-center gap-2 font-heading text-base font-semibold">{children}</h1>;
}

/** Buttons for the current page, shown right after the title. */
export function PageActions({ children }: { children: ReactNode }) {
  return <div className="flex shrink-0 items-center gap-2">{children}</div>;
}

export function PageBody({ children }: { children: ReactNode }) {
  return <div className="mx-auto flex w-full max-w-3xl flex-col gap-6 p-6">{children}</div>;
}
