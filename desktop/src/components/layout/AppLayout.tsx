import { ArrowNavigatioMenu } from "../ArrowNavigatioMenu";
import { ThemeToggle } from "../ThemeToggle";
import { cn } from "../utils";

const MAC_STOP_LIGHT_PADDING_CLS = "pl-20";

export function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen min-h-0 w-full scrollbar-none flex-col items-start overflow-hidden bg-app-bg-base px-1.5">
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
        "flex h-10 w-full shrink-0 items-center justify-start bg-transparent text-xs font-medium text-app-foreground select-none",
        MAC_STOP_LIGHT_PADDING_CLS,
      )}
      data-tauri-drag-region
    >
      <ArrowNavigatioMenu />
    </div>
  );
}

function Bottombar() {
  return (
    <div className="flex h-7 w-full shrink-0 items-center justify-start bg-transparent px-0.5 text-xs font-medium text-app-foreground select-none">
      <ThemeToggle size={14} />
    </div>
  );
}
