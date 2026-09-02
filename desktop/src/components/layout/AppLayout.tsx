import { Breadcrumbs } from "../Breadcrumbs";


export function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="h-full min-h-screen flex flex-col items-start w-full p-2">
      <Titlebar />
      {children}
    </div>
  );
}


function Titlebar() {
  return (
    <div
      className="flex h-8 w-full select-none items-center justify-start bg-transparent px-4 pl-20 text-xs font-medium text-app-foreground"
      data-tauri-drag-region
    >
      <Breadcrumbs />
    </div>
  );
}
