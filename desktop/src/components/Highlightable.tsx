import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "./utils";

type HighlightableProps = {
  children?: ReactNode;
  className?: string;
} & Omit<HTMLAttributes<HTMLDivElement>, "className" | "children">;

export function Highlightable({ children, className, ...props }: HighlightableProps) {
  return (
    <div {...props} className={cn("highlightable", className)}>
      {children}
    </div>
  );
}
