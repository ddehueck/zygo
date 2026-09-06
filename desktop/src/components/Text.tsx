import type { HTMLAttributes, ReactNode } from "react";
import { tv } from "tailwind-variants";

const text = tv({
  base: "min-w-0",
  variants: {
    size: {
      small: "text-sm",
      medium: "text-base",
      large: "text-xl",
    },
    variant: {
      default: "text-app-foreground",
      muted: "text-app-foreground-muted",
      secondary: "text-app-foreground-secondary",
      danger: "text-app-danger",
    },
  },
  defaultVariants: {
    size: "medium",
    variant: "default",
  },
});

type TextProps = {
  children?: ReactNode;
  size?: "small" | "medium" | "large";
  variant?: "default" | "muted" | "secondary" | "danger";
  className?: string;
} & Omit<HTMLAttributes<HTMLSpanElement>, "className" | "children">;

export function Text({ children, size, variant, className, ...props }: TextProps) {
  return (
    <span {...props} className={text({ size, variant, className })}>
      {children}
    </span>
  );
}

type DescriptionProps = {
  children?: ReactNode;
  className?: string;
} & Omit<HTMLAttributes<HTMLParagraphElement>, "className" | "children">;

export function Description({ children, className, ...props }: DescriptionProps) {
  return (
    <p {...props} className={text({ size: "small", variant: "muted", className })}>
      {children}
    </p>
  );
}

type HeadingProps = {
  text?: string;
  children?: ReactNode;
  size?: "large" | "medium";
  className?: string;
} & Omit<HTMLAttributes<HTMLHeadingElement>, "className" | "children">;

const heading = tv({
  base: "min-w-0 font-semibold tracking-tight break-all text-app-foreground",
  variants: {
    size: {
      large: "text-3xl",
      medium: "text-xl",
    },
  },
  defaultVariants: {
    size: "large",
  },
});

export function Heading({ text: headingText, children, size, className, ...props }: HeadingProps) {
  return (
    <h1 {...props} className={heading({ size, className })}>
      {children ?? headingText}
    </h1>
  );
}
