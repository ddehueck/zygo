import type { RefAttributes } from "react";
import {
  GridList as AriaGridList,
  GridListItem as AriaGridListItem,
  type GridListItemProps as AriaGridListItemProps,
  type GridListProps as AriaGridListProps,
  type GridListRenderProps,
} from "react-aria-components/GridList";
import { composeRenderProps } from "react-aria-components/composeRenderProps";
import { tv } from "tailwind-variants";
import { composeTailwindRenderProps, focusRing } from "./utils";

export type GridListProps<T> = AriaGridListProps<T> & RefAttributes<HTMLDivElement>;

export function GridList<T>(props: GridListProps<T>) {
  return (
    <AriaGridList
      {...props}
      className={composeTailwindRenderProps<
        GridListRenderProps & { defaultClassName: string | undefined }
      >(props.className, "relative box-border w-full font-sans")}
    />
  );
}

const itemStyles = tv({
  extend: focusRing,
  base: "group/item pressed:bg-app-interaction-pressed selected:bg-app-accent/10 selected:hover:bg-app-accent/20 selected:pressed:bg-app-accent/20 relative cursor-pointer border-b border-app-border px-2 py-2 text-sm text-app-foreground -outline-offset-2 select-none last:border-b-0 hover:bg-app-interaction-hover focus-visible:bg-app-interaction-hover disabled:text-app-foreground-muted",
});

export function GridListItem<T = object>(props: AriaGridListItemProps<T>) {
  return (
    <AriaGridListItem
      {...props}
      className={composeRenderProps(props.className, (className, renderProps) =>
        itemStyles({ ...renderProps, className }),
      )}
    />
  );
}
