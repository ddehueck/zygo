import {
  TokenField as AriaTokenField,
  TokenInput as AriaTokenInput,
  Token as AriaToken,
  type TokenFieldProps as AriaTokenFieldProps,
  type TokenFieldRenderProps,
  type TokenInputProps,
  type TokenInputRenderProps,
  type TokenProps,
  type TokenRenderProps,
  type TokenFieldValue,
} from "react-aria-components/TokenField";
import { composeRenderProps } from "react-aria-components/composeRenderProps";
import type React from "react";
import { tv } from "tailwind-variants";
import { Description, Label } from "@/components/Field";
import { focusRing } from "@/components/utils";

type TokenFieldClassNameRenderProps = TokenFieldRenderProps & {
  defaultClassName: string | undefined;
};

type TokenInputClassNameRenderProps = TokenInputRenderProps & {
  defaultClassName: string | undefined;
};

type TokenClassNameRenderProps = TokenRenderProps & {
  defaultClassName: string | undefined;
};

const tokenFieldStyles = tv({
  base: "group flex w-full flex-col gap-1 font-sans text-sm text-app-foreground",
  variants: {
    isDisabled: {
      true: "text-app-foreground-muted forced-colors:text-system-gray-text",
    },
  },
});

// NB: Never use flex layout for the input. They will break the token field's layout.
const tokenInputStyles = tv({
  extend: focusRing,
  base: "min-h-9 w-full overflow-x-auto rounded-lg border bg-app-bg-surface px-2 py-1 font-sans text-sm leading-6 text-app-foreground outline-0 transition empty:before:pointer-events-none empty:before:text-app-foreground-muted/80 empty:before:content-[attr(data-placeholder)] forced-colors:bg-system-field",
  variants: {
    isFocused: {
      false:
        "border-app-border hover:border-app-foreground-muted forced-colors:border-system-button-border",
      true: "border-app-accent forced-colors:border-system-highlight",
    },
    isDisabled: {
      true: "border-app-border/50 text-app-foreground-muted forced-colors:border-system-gray-text forced-colors:text-system-gray-text",
    },
    isReadOnly: {
      false: "cursor-text",
      true: "cursor-default",
    },
  },
});

const tokenStyles = tv({
  base: "mx-0.5 inline-flex h-6 cursor-default items-center rounded-full px-2 align-top font-mono text-xs transition select-none",
  variants: {
    isSelected: {
      false: "bg-app-accent/15 text-app-accent hover:bg-app-accent/20",
      true: "bg-app-accent text-app-accent-foreground forced-colors:bg-system-highlight forced-colors:text-system-highlight-text",
    },
    isDisabled: {
      true: "bg-app-border/50 text-app-foreground-muted forced-colors:bg-system-canvas forced-colors:text-system-gray-text",
    },
  },
});

export interface TokenFieldProps<T extends TokenFieldValue = TokenFieldValue> extends Omit<
  AriaTokenFieldProps<T>,
  "children"
> {
  label?: string;
  description?: string;
  placeholder?: string;
  inputRef?: React.Ref<HTMLDivElement>;
  inputClassName?: TokenInputProps["className"];
  children: TokenInputProps["children"];
}

export function TokenField<T extends TokenFieldValue = TokenFieldValue>({
  label,
  description,
  placeholder,
  inputRef,
  inputClassName,
  children,
  ...props
}: TokenFieldProps<T>) {
  const { className, ...ariaProps } = props;

  return (
    <AriaTokenField
      {...ariaProps}
      className={composeRenderProps<string | undefined, TokenFieldClassNameRenderProps, string>(
        className,
        (className, renderProps) => tokenFieldStyles({ ...renderProps, className }),
      )}
    >
      {label && <Label>{label}</Label>}
      <AriaTokenInput
        ref={inputRef}
        data-placeholder={placeholder}
        className={composeRenderProps<string | undefined, TokenInputClassNameRenderProps, string>(
          inputClassName,
          (className, renderProps) => tokenInputStyles({ ...renderProps, className }),
        )}
      >
        {children}
      </AriaTokenInput>
      {description && <Description>{description}</Description>}
    </AriaTokenField>
  );
}

export function Token(props: TokenProps) {
  return (
    <AriaToken
      {...props}
      className={composeRenderProps<string | undefined, TokenClassNameRenderProps, string>(
        props.className,
        (className, renderProps) => tokenStyles({ ...renderProps, className }),
      )}
    />
  );
}
