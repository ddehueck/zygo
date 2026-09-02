import type { ComponentProps } from "react";
import { cn } from "./utils";

type ScrollAreaProps = ComponentProps<"div">;

export function ScrollArea({
  className = "",
  children,
  ...props
}: ScrollAreaProps) {
  return (
    <div
      className={cn(
        "overflow-auto",
        "scrollbar-thin",
        "scrollbar-track-transparent",
        "scrollbar-thumb-app-border",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}
