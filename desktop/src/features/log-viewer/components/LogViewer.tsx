import { LogSearchProvider } from "../search/LogSearchContext";
import { LogViewport } from "./LogViewport";
import { Search } from "./Search";

export function LogViewer({ workflowRunId }: { workflowRunId: number }) {
  return (
    <LogSearchProvider>
      <div className="flex min-h-0 w-full flex-1 flex-col overflow-hidden">
        <Search />
        <LogViewport key={workflowRunId} workflowRunId={workflowRunId} />
      </div>
    </LogSearchProvider>
  );
}
