import { Clock, EyeOff, Tag } from "lucide-react";

import { IconButton } from "@/components/IconButton";
import { cn } from "@/components/utils";

export interface LogViewportHeaderProps {
  showDate: boolean;
  showJobId: boolean;
  onToggleDate: () => void;
  onToggleJobId: () => void;
}

const iconButtonClassName = "text-app-foreground-muted hover:opacity-100";
const columnLabelClassName =
  "group flex min-w-0 items-center gap-1 text-xs text-app-foreground-muted";
const columnHideButtonClassName =
  "ml-auto opacity-0 group-hover:opacity-70 focus-visible:opacity-100";

export function LogViewportHeader({
  showDate,
  showJobId,
  onToggleDate,
  onToggleJobId,
}: LogViewportHeaderProps) {
  return (
    <header
      aria-label="Log display options"
      className="sticky top-0 z-20 w-full shrink-0 border-b border-app-border bg-app-bg-elevated px-3 py-1"
    >
      <div
        className="log-viewer-grid grid items-center gap-3"
        style={{
          gridTemplateColumns: [showDate && "10.5rem", showJobId && "4rem", "minmax(0, 1fr)"]
            .filter(Boolean)
            .join(" "),
        }}
      >
        {showDate && (
          <div className={columnLabelClassName}>
            <Clock aria-hidden size={12} />
            <span className="whitespace-nowrap" style={{ fontSize: "10px" }}>
              Date
            </span>
            <IconButton
              size={14}
              type="button"
              onClick={onToggleDate}
              aria-label="Hide log dates"
              className={cn(iconButtonClassName, columnHideButtonClassName)}
            >
              <EyeOff aria-hidden size={10} />
            </IconButton>
          </div>
        )}
        {showJobId && (
          <div className={columnLabelClassName}>
            <Tag aria-hidden size={12} />
            <span className="whitespace-nowrap" style={{ fontSize: "10px" }}>
              Job ID
            </span>
            <IconButton
              size={14}
              type="button"
              onClick={onToggleJobId}
              aria-label="Hide job IDs"
              className={cn(iconButtonClassName, columnHideButtonClassName)}
            >
              <EyeOff aria-hidden size={10} />
            </IconButton>
          </div>
        )}
        <div className="flex w-full items-center justify-end gap-1">
          {!showDate && (
            <IconButton
              size={14}
              type="button"
              onClick={onToggleDate}
              aria-label="Show log dates"
              className={iconButtonClassName}
            >
              <Clock aria-hidden size={14} />
            </IconButton>
          )}
          {!showJobId && (
            <IconButton
              size={14}
              type="button"
              onClick={onToggleJobId}
              aria-label="Show job IDs"
              className={iconButtonClassName}
            >
              <Tag aria-hidden size={14} />
            </IconButton>
          )}
        </div>
      </div>
    </header>
  );
}
