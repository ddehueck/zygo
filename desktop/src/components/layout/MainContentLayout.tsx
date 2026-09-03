import { ScrollArea } from "../ScrollArea";

export function MainContentLayout({
  header,
  children,
}: {
  header: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-0 w-full flex-1 flex-col overflow-hidden rounded-lg border border-app-border bg-app-bg-surface">
      <div className="shrink-0">{header}</div>
      <ScrollArea className="min-h-0 flex-1">{children}</ScrollArea>
    </div>
  );
}
