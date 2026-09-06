import { tv } from "tailwind-variants";

type HeadingProps = {
  text: string;
  size?: "large" | "medium";
  className?: string;
};

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

export function Heading({ text, size, className }: HeadingProps) {
  return <h1 className={heading({ size, className })}>{text}</h1>;
}
