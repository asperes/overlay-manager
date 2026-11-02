import { ThemeProvider } from "next-themes";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ConfigProvider } from "@/features/config/config-provider";
import { AppShell } from "@/features/shell/app-shell";
import { NavigationProvider } from "@/features/shell/navigation";
import { StatusProvider } from "@/features/status/status-provider";

export function App() {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <TooltipProvider>
        <ConfigProvider>
          <StatusProvider>
            <NavigationProvider>
              <AppShell />
            </NavigationProvider>
          </StatusProvider>
        </ConfigProvider>
        <Toaster />
      </TooltipProvider>
    </ThemeProvider>
  );
}
