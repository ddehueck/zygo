import { cn } from "@/lib/utils";

export type StatusVariant = "idle" | "pending" | "running" | "success" | "error";

type StatusIndicatorProps = {
  variant: StatusVariant;
  count?: number;
  className?: string;
  size?: "sm" | "md";
};

export function StatusIndicator({
  variant,
  count,
  className,
  size = "sm",
}: StatusIndicatorProps) {
  const sizeClasses = size === "sm" ? "size-4" : "size-5";
  const iconSize = size === "sm" ? "size-3" : "size-4";

  if (variant === "idle") {
    return null;
  }

  return (
    <div className={cn("flex items-center gap-1", className)}>
      <div
        className={cn(
          "flex items-center justify-center rounded-full",
          sizeClasses,
          {
            "bg-muted text-muted-foreground": variant === "pending",
            "bg-blue-500/20 text-blue-500": variant === "running",
            "bg-green-500/20 text-green-500": variant === "success",
            "bg-destructive/20 text-destructive": variant === "error",
          }
        )}
      >
        {variant === "pending" && <PendingIcon className={iconSize} />}
        {variant === "running" && <SpinnerIcon className={iconSize} />}
        {variant === "success" && <CheckIcon className={iconSize} />}
        {variant === "error" && <XIcon className={iconSize} />}
      </div>
      {count !== undefined && count > 1 && (
        <span className="text-xs font-medium text-muted-foreground">
          {count}
        </span>
      )}
    </div>
  );
}

function PendingIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 20 20"
      fill="currentColor"
      className={className}
    >
      <circle cx="10" cy="10" r="3" />
    </svg>
  );
}

function SpinnerIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className={cn("animate-spin", className)}
    >
      <path d="M10 3a7 7 0 1 0 7 7" strokeLinecap="round" />
    </svg>
  );
}

function CheckIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 20 20"
      fill="currentColor"
      className={className}
    >
      <path
        fillRule="evenodd"
        d="M16.704 4.153a.75.75 0 0 1 .143 1.052l-8 10.5a.75.75 0 0 1-1.127.075l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 0 1 1.05-.143Z"
        clipRule="evenodd"
      />
    </svg>
  );
}

function XIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 20 20"
      fill="currentColor"
      className={className}
    >
      <path d="M6.28 5.22a.75.75 0 0 0-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 1 0 1.06 1.06L10 11.06l3.72 3.72a.75.75 0 1 0 1.06-1.06L11.06 10l3.72-3.72a.75.75 0 0 0-1.06-1.06L10 8.94 6.28 5.22Z" />
    </svg>
  );
}

