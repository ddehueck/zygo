import { Circle, type LucideProps } from "lucide-react";
import { cn } from "../utils";

export type StatusIconStatus = "completed" | "in-progress" | "errored";

export interface StatusIconProps extends LucideProps {
  status: StatusIconStatus;
}

const statusClasses: Record<StatusIconStatus, string> = {
  completed: "text-app-success",
  "in-progress": "text-app-accent motion-safe:animate-pulse",
  errored: "text-app-danger",
};

export function StatusIcon({ status, className, ...props }: StatusIconProps) {
  return (
    <Circle
      {...props}
      className={cn("fill-current stroke-none", statusClasses[status], className)}
    />
  );
}
