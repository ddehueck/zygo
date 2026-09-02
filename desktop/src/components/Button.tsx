import { composeRenderProps } from "react-aria-components/composeRenderProps";
import {
  Button as RACButton,
  type ButtonProps as RACButtonProps,
} from "react-aria-components/Button";
import { tv } from "tailwind-variants";
import { focusRing } from "./utils";

export interface ButtonProps extends RACButtonProps {
  /** @default "primary" */
  variant?: "primary" | "secondary" | "outline" | "ghost" | "destructive";
}

const button = tv({
  extend: focusRing,
  base: "relative inline-flex items-center justify-center gap-2 border border-transparent h-9 box-border px-3.5 py-0 font-sans text-sm text-center transition rounded-full cursor-default [-webkit-tap-highlight-color:transparent]",
  variants: {
    variant: {
      primary:
        "bg-app-accent text-app-accent-foreground hover:bg-app-accent/90 data-pressed:bg-app-accent/80",
      secondary:
        "border-app-border bg-app-background-tertiary text-app-foreground-secondary hover:bg-app-background-tertiary/80 data-pressed:bg-app-background-tertiary/70",
      outline:
        "border-app-border bg-transparent text-app-foreground hover:bg-app-background-secondary data-pressed:bg-app-background-tertiary",
      ghost:
        "border-transparent bg-transparent text-app-foreground-muted hover:bg-app-background-tertiary data-pressed:bg-app-background-tertiary/80",
      destructive:
        "bg-app-danger text-app-danger-foreground hover:bg-app-danger/90 data-pressed:bg-app-danger/80",
    },
    isDisabled: {
      true: "border-transparent bg-app-border/50 text-app-foreground-muted forced-colors:text-system-gray-text",
    },
    isPending: {
      true: "text-transparent",
    },
  },
  defaultVariants: {
    variant: "primary",
  },
  compoundVariants: [
    {
      variant: "ghost",
      isDisabled: true,
      class: "bg-transparent",
    },
  ],
});

export function Button({ variant, className, children, ...props }: ButtonProps) {
  const resolvedVariant = variant ?? "primary";

  return (
    <RACButton
      {...props}
      className={composeRenderProps(className, (className, renderProps) =>
        button({ ...renderProps, variant: resolvedVariant, className }),
      )}
    >
      {composeRenderProps(children, (children, { isPending }) => (
        <>
          {children}
          {isPending && (
            <span aria-hidden className="absolute inset-0 flex items-center justify-center">
              <svg
                className={`h-4 w-4 animate-spin ${
                  resolvedVariant === "primary"
                    ? "text-app-accent-foreground"
                    : resolvedVariant === "destructive"
                      ? "text-app-danger-foreground"
                      : resolvedVariant === "secondary"
                        ? "text-app-foreground-secondary"
                        : "text-app-foreground"
                }`}
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <circle cx="12" cy="12" r="10" strokeWidth="4" fill="none" className="opacity-25" />
                <circle
                  cx="12"
                  cy="12"
                  r="10"
                  strokeWidth="4"
                  strokeLinecap="round"
                  fill="none"
                  pathLength="100"
                  strokeDasharray="60 140"
                  strokeDashoffset="0"
                />
              </svg>
            </span>
          )}
        </>
      ))}
    </RACButton>
  );
}
