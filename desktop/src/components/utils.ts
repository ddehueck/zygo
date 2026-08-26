import { composeRenderProps } from "react-aria-components/composeRenderProps";
import { twMerge } from "tailwind-merge";
import { tv } from "tailwind-variants";

export const focusRing = tv({
  base: "outline outline-blue-600 dark:outline-blue-500 forced-colors:outline-[Highlight] outline-offset-2",
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
