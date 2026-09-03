import { clsx, type ClassValue } from "clsx";
import { composeRenderProps } from "react-aria-components/composeRenderProps";
import { twMerge } from "tailwind-merge";
import { tv } from "tailwind-variants";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const focusRing = tv({
  base: "outline outline-app-accent forced-colors:outline-system-highlight outline-offset-2",
  variants: {
    isFocusVisible: {
      false: "outline-0",
      true: "outline-2",
    },
  },
});

export function composeTailwindRenderProps<T>(
  className: string | ((value: T) => string) | undefined,
  tailwindClassName: string,
): string | ((value: T) => string) {
  return composeRenderProps(className, (className) => twMerge(tailwindClassName, className));
}
