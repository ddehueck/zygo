import type { ComponentProps } from "react";
import { cn } from "./utils";

type ScrollAreaProps = ComponentProps<"div">;

export const scrollAreaClassName = cn(
  "overflow-auto supports-[overflow:overlay]:[overflow:overlay]",
  "[&::-webkit-scrollbar]:size-2.5 [&::-webkit-scrollbar]:bg-transparent",
  "[&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-track-piece]:bg-transparent",
  "[&::-webkit-scrollbar-corner]:bg-transparent [&::-webkit-scrollbar-button]:hidden",
  "[&::-webkit-scrollbar-thumb]:min-h-10 [&::-webkit-scrollbar-thumb]:min-w-10",
  "[&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:border-2",
  "[&::-webkit-scrollbar-thumb]:border-transparent [&::-webkit-scrollbar-thumb]:bg-app-border",
  "[&::-webkit-scrollbar-thumb]:bg-clip-content [&::-webkit-scrollbar-thumb:hover]:bg-app-border-secondary",
);

export function ScrollArea({ className = "", children, ...props }: ScrollAreaProps) {
  return (
    <div className={cn(scrollAreaClassName, className)} {...props}>
      {children}
    </div>
  );
}
