import { Circle, CircleDashed, type LucideProps } from "lucide-react";
import { cn } from "../utils";

export type StatusIconStatus = "completed" | "in-progress" | "errored";

export interface StatusIconProps extends LucideProps {
  status: StatusIconStatus;
}

const statusClasses: Record<StatusIconStatus, string> = {
  completed: "fill-current stroke-none text-app-success",
  "in-progress": "fill-none stroke-current text-app-accent motion-safe:animate-pulse",
  errored: "fill-current stroke-none text-app-danger",
};

export function StatusIcon({ status, className, ...props }: StatusIconProps) {
  const Icon = status === "in-progress" ? CircleDashed : Circle;

  return <Icon {...props} className={cn(statusClasses[status], className)} />;
}
