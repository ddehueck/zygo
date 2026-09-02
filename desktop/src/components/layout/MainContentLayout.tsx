import { ScrollArea } from "../ScrollArea";

export function MainContentLayout({
  titleContent,
  children,
}: {
  titleContent: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-0 w-full flex-1 flex-col overflow-hidden rounded-lg border border-app-border">
      <div className="shrink-0">{titleContent}</div>
      <ScrollArea className="min-h-0 flex-1">{children}</ScrollArea>
    </div>
  );
}
