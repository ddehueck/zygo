import { ScrollArea } from "../ScrollArea";

export function MainContentLayout({ titleContent, children }: { titleContent: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="w-full h-full border border-color rounded-lg flex-1">
      <div>{titleContent}</div>
      <ScrollArea>
        {children}
      </ScrollArea>
    </div>
  );
}
