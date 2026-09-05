import { composeRenderProps } from "react-aria-components/composeRenderProps";
import { Button, type ButtonProps } from "./Button";
import { composeTailwindRenderProps } from "./utils";

export interface IconButtonProps extends Omit<ButtonProps, "variant"> {
  /** The icon size in pixels. */
  size?: number;
}

export function IconButton({ size = 16, className, style, ...props }: IconButtonProps) {
  return (
    <Button
      {...props}
      variant="ghost"
      className={composeTailwindRenderProps(
        className,
        "h-auto w-auto px-0 data-disabled:cursor-not-allowed data-disabled:opacity-40 data-disabled:hover:bg-transparent forced-colors:data-disabled:opacity-100",
      )}
      style={composeRenderProps(style, (style) => ({
        padding: size / 6,
        ...style,
      }))}
    />
  );
}
