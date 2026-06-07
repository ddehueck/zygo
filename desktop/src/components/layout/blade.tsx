import * as React from "react";
import { cn } from "@/lib/utils";

type BladeProps = React.ComponentProps<"aside"> & {
  /** Which side of the layout the blade is positioned */
  position?: "left" | "right";
  /** Width of the blade (default: w-80) */
  width?: string;
};

function Blade({
  position = "right",
  width = "w-80",
  className,
  children,
  ...props
}: BladeProps) {
  return (
    <aside
      className={cn(
        "flex h-full shrink-0 flex-col overflow-hidden bg-sidebar",
        width,
        position === "left" ? "border-r" : "border-l",
        className
      )}
      {...props}
    >
      {children}
    </aside>
  );
}

type BladeHeaderProps = React.ComponentProps<"div">;

function BladeHeader({ className, children, ...props }: BladeHeaderProps) {
  return (
    <div
      className={cn("shrink-0 border-b px-4 py-3", className)}
      {...props}
    >
      {children}
    </div>
  );
}

type BladeContentProps = React.ComponentProps<"div">;

function BladeContent({ className, children, ...props }: BladeContentProps) {
  return (
    <div
      className={cn("flex-1 overflow-y-auto p-3", className)}
      {...props}
    >
      {children}
    </div>
  );
}

export { Blade, BladeHeader, BladeContent };

