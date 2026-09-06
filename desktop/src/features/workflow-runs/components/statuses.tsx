import { StatusIcon as StatusGlyph, type StatusIconStatus } from "@/components/icons";

type RunStatusIconProps = {
  status: string;
  className?: string;
};

export function StatusIcon({ status, className = "size-3.5" }: RunStatusIconProps) {
  const iconStatus = statusIconStatus(status);

  if (iconStatus === null) {
    return (
      <span aria-hidden className="inline-block size-2 rounded-full bg-app-foreground-muted" />
    );
  }

  return <StatusGlyph status={iconStatus} aria-hidden className={className} />;
}

export function statusLabel(status: string): string {
  if (status === "succeeded") return "Completed";
  return status.replace(/_/g, " ").replace(/^./, (letter) => letter.toUpperCase());
}

function statusIconStatus(status: string): StatusIconStatus | null {
  switch (status) {
    case "succeeded":
    case "completed":
      return "completed";
    case "running":
      return "in-progress";
    case "failed":
    case "errored":
      return "errored";
    default:
      return null;
  }
}

export function RunStatus({ status }: { status: string }) {
  return (
    <span className="inline-flex shrink-0 items-center gap-1.5 text-sm font-medium text-app-foreground">
      <StatusIcon status={status} />
      {statusLabel(status)}
    </span>
  );
}
