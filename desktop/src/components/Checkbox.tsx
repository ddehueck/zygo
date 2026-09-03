"use client";
import { Check, Minus } from "lucide-react";
import React from "react";
import {
  CheckboxField,
  CheckboxButton,
  type CheckboxFieldProps,
  type ValidationResult,
} from "react-aria-components/Checkbox";
import { composeRenderProps } from "react-aria-components/composeRenderProps";
import { tv } from "tailwind-variants";
import { focusRing } from "./utils";
import { Description, FieldError } from "./Field";

const checkboxStyles = tv({
  base: "group relative flex items-center gap-2 font-sans text-sm transition [-webkit-tap-highlight-color:transparent]",
  variants: {
    isDisabled: {
      false: "text-app-foreground",
      true: "text-app-foreground-muted forced-colors:text-system-gray-text",
    },
  },
});

const boxStyles = tv({
  extend: focusRing,
  base: "box-border flex h-4.5 w-4.5 shrink-0 items-center justify-center rounded-sm border transition",
  variants: {
    isSelected: {
      false:
        "group-pressed:border-app-foreground-muted border-app-border bg-app-bg-surface forced-colors:border-system-button-border forced-colors:bg-system-canvas",
      true: "group-pressed:bg-app-accent/80 group-pressed:border-app-accent/80 border-app-accent bg-app-accent forced-colors:border-system-highlight! forced-colors:bg-system-highlight!",
    },
    isInvalid: {
      true: "group-pressed:border-app-danger/80 border-app-danger forced-colors:border-system-mark! forced-colors:bg-system-mark!",
    },
    isDisabled: {
      true: "border-app-border bg-app-border/50 forced-colors:border-system-gray-text! forced-colors:bg-system-canvas!",
    },
  },
  compoundVariants: [
    {
      isSelected: true,
      isInvalid: true,
      class:
        "group-pressed:bg-app-danger/80 group-pressed:border-app-danger/80 border-app-danger bg-app-danger",
    },
  ],
});

const iconStyles =
  "w-3.5 h-3.5 text-app-accent-foreground group-disabled:text-app-foreground-muted forced-colors:text-system-highlight-text pointer-events-none";

interface CheckboxProps extends CheckboxFieldProps {
  children?: React.ReactNode;
  description?: string;
  errorMessage?: string | ((validation: ValidationResult) => string);
}

export function Checkbox(props: CheckboxProps) {
  return (
    <CheckboxField {...props} className="group flex flex-col gap-1">
      <CheckboxButton
        className={composeRenderProps(props.className, (className, renderProps) =>
          checkboxStyles({ ...renderProps, className }),
        )}
      >
        {composeRenderProps(
          props.children,
          (children, { isSelected, isIndeterminate, ...renderProps }) => (
            <>
              <div
                className={boxStyles({ isSelected: isSelected || isIndeterminate, ...renderProps })}
              >
                {isIndeterminate ? (
                  <Minus aria-hidden className={iconStyles} />
                ) : isSelected ? (
                  <Check aria-hidden className={iconStyles} />
                ) : null}
              </div>
              {children}
            </>
          ),
        )}
      </CheckboxButton>
      {props.description && <Description className="ms-6.5">{props.description}</Description>}
      <FieldError className="ms-6.5">{props.errorMessage}</FieldError>
    </CheckboxField>
  );
}
