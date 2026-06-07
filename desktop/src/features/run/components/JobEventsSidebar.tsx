import { cn } from "@/lib/utils";
import {
  useJobExecutions,
  useJobStateFromStore,
} from "../hooks/useRunEventsStore";
import { JobExecutionRow } from "./JobExecutionRow";
import { Blade, BladeHeader, BladeContent } from "@/components/layout/blade";

type JobEventsSidebarProps = {
  runId: string;
  jobId: string;
  jobName: string;
  className?: string;
};

export function JobEventsSidebar({
  runId,
  jobId,
  jobName,
  className,
}: JobEventsSidebarProps) {
  const { data: jobState, isLoading } = useJobStateFromStore({ runId, jobId });
  const { data: executions } = useJobExecutions({ runId, jobId });

  return (
    <Blade position="right" className={cn(className)}>
      <BladeHeader>
        <div className="flex items-center gap-2">
          <div className="flex size-6 items-center justify-center rounded bg-primary/10 text-primary">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              className="size-3.5"
            >
              <path
                fillRule="evenodd"
                d="M7.84 1.804A1 1 0 0 1 8.82 1h2.36a1 1 0 0 1 .98.804l.331 1.652a6.993 6.993 0 0 1 1.929 1.115l1.598-.54a1 1 0 0 1 1.186.447l1.18 2.044a1 1 0 0 1-.205 1.251l-1.267 1.113a7.047 7.047 0 0 1 0 2.228l1.267 1.113a1 1 0 0 1 .206 1.25l-1.18 2.045a1 1 0 0 1-1.187.447l-1.598-.54a6.993 6.993 0 0 1-1.929 1.115l-.33 1.652a1 1 0 0 1-.98.804H8.82a1 1 0 0 1-.98-.804l-.331-1.652a6.993 6.993 0 0 1-1.929-1.115l-1.598.54a1 1 0 0 1-1.186-.447l-1.18-2.044a1 1 0 0 1 .205-1.251l1.267-1.114a7.05 7.05 0 0 1 0-2.227L1.821 7.773a1 1 0 0 1-.206-1.25l1.18-2.045a1 1 0 0 1 1.187-.447l1.598.54A6.993 6.993 0 0 1 7.51 3.456l.33-1.652ZM10 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
                clipRule="evenodd"
              />
            </svg>
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="truncate text-sm font-medium text-sidebar-foreground">
              {jobName}
            </h3>
            <p className="text-xs text-muted-foreground">Job Executions</p>
          </div>
        </div>
      </BladeHeader>

      {/* Stats Bar */}
      {jobState && (
        <div className="shrink-0 border-b px-4 py-2">
          <div className="flex items-center gap-4 text-xs">
            {jobState.runningCount > 0 && (
              <span className="flex items-center gap-1 text-blue-500">
                <span className="size-2 animate-pulse rounded-full bg-blue-500" />
                {jobState.runningCount} running
              </span>
            )}
            {jobState.pendingCount > 0 && (
              <span className="text-muted-foreground">
                {jobState.pendingCount} pending
              </span>
            )}
            {jobState.completedCount > 0 && (
              <span className="text-green-500">
                {jobState.completedCount} done
              </span>
            )}
            {jobState.erroredCount > 0 && (
              <span className="text-destructive">
                {jobState.erroredCount} failed
              </span>
            )}
          </div>
        </div>
      )}

      <BladeContent>
        {isLoading ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            Loading...
          </div>
        ) : !executions || executions.length === 0 ? (
          <div className="py-8 text-center">
            <p className="text-sm text-muted-foreground">No executions yet</p>
            <p className="mt-1 text-xs text-muted-foreground/70">
              Executions will appear here when the job runs
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {executions.map((execution) => (
              <JobExecutionRow
                key={execution.idempotencyKey}
                execution={execution}
              />
            ))}
          </div>
        )}
      </BladeContent>
    </Blade>
  );
}
