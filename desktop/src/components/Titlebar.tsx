import { Breadcrumbs } from "./Breadcrumbs";

export function Titlebar() {
  return (
    <div
      className="flex h-8 w-full select-none items-center justify-start bg-transparent px-4 pl-20 text-xs font-medium text-app-foreground"
      data-tauri-drag-region
    >
      <Breadcrumbs />
    </div>
  );
}
