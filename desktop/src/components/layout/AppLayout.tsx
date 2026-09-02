import { Breadcrumbs } from "../Breadcrumbs";
import { ThemeToggle } from "../ThemeToggle";
import { cn } from "../utils";

const MAC_STOP_LIGHT_PADDING_CLS = "pl-20";

export function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen min-h-0 w-full flex-col items-start overflow-hidden px-2">
      <Titlebar />
      {children}
      <Bottombar />
    </div>
  );
}

function Titlebar() {
  return (
    <div
      className={cn(
        "flex h-10 shrink-0 w-full select-none items-center justify-start bg-transparent text-xs font-medium text-app-foreground",
        MAC_STOP_LIGHT_PADDING_CLS,
      )}
      data-tauri-drag-region
    >
      <Breadcrumbs />
    </div>
  );
}

function Bottombar() {
  return (
    <div className="flex h-8 shrink-0 px-1 w-full select-none items-center justify-start bg-transparent text-xs font-medium text-app-foreground">
      <ThemeToggle size={14} />
    </div>
  );
}
