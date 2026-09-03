import {
  ListBox as AriaListBox,
  ListBoxItem as AriaListBoxItem,
  Header,
  type ListBoxProps as AriaListBoxProps,
  type ListBoxItemProps,
} from "react-aria-components/ListBox";
import { composeRenderProps } from "react-aria-components/composeRenderProps";
import { tv } from "tailwind-variants";
import { composeTailwindRenderProps, focusRing } from "./utils";

export type ListBoxProps<T> = AriaListBoxProps<T>;

export function ListBox<T>({ children, ...props }: ListBoxProps<T>) {
  return (
    <AriaListBox
      {...props}
      className={composeTailwindRenderProps(
        props.className,
        "w-50 rounded-lg border border-app-border bg-app-bg-surface p-1 font-sans outline-0 data-[layout=grid]:grid data-[layout=stack]:flex data-[layout=stack]:flex-col [&[data-layout=stack][data-orientation=horizontal]]:flex-row data-[orientation=horizontal]:overflow-x-auto",
      )}
    >
      {children}
    </AriaListBox>
  );
}

export const itemStyles = tv({
  extend: focusRing,
  base: "group relative flex shrink-0 cursor-default items-center gap-8 rounded-md px-2.5 py-1.5 text-sm will-change-transform forced-color-adjust-none select-none",
  variants: {
    isSelected: {
      false:
        "pressed:bg-app-interaction-pressed text-app-foreground-secondary -outline-offset-2 hover:bg-app-interaction-hover",
      true: "bg-app-accent text-app-accent-foreground -outline-offset-4 outline-app-accent-foreground forced-colors:bg-system-highlight forced-colors:text-system-highlight-text forced-colors:outline-system-highlight-text [&+[data-selected]]:rounded-t-none [&:has(+[data-selected])]:rounded-b-none",
    },
    isDisabled: {
      true: "text-app-foreground-muted forced-colors:text-system-gray-text",
    },
    isFocused: {
      true: "bg-app-accent text-app-accent-foreground forced-colors:bg-system-highlight forced-colors:text-system-highlight-text",
    },
  },
});

export function ListBoxItem<T = object>(props: ListBoxItemProps<T>) {
  let textValue =
    props.textValue || (typeof props.children === "string" ? props.children : undefined);
  return (
    <AriaListBoxItem {...props} textValue={textValue} className={itemStyles}>
      {composeRenderProps(props.children, (children) => (
        <>
          {children}
          <div className="absolute right-4 bottom-0 left-4 hidden h-px bg-app-accent-foreground/20 forced-colors:bg-system-highlight-text [.group[data-selected]:has(+[data-selected])_&]:block" />
        </>
      ))}
    </AriaListBoxItem>
  );
}
