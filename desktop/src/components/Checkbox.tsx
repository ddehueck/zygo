'use client';
import {Check, Minus} from 'lucide-react';
import React from 'react';
import {
  CheckboxField,
  CheckboxButton,
  type CheckboxFieldProps,
  type ValidationResult
} from 'react-aria-components/Checkbox';
import {composeRenderProps} from 'react-aria-components/composeRenderProps';
import {tv} from 'tailwind-variants';
import {focusRing} from './utils';
import {Description, FieldError} from './Field';

const checkboxStyles = tv({
  base: 'flex gap-2 items-center group font-sans text-sm transition relative [-webkit-tap-highlight-color:transparent]',
  variants: {
    isDisabled: {
      false: 'text-app-foreground',
      true: 'text-app-foreground-muted forced-colors:text-system-gray-text'
    }
  }
});

const boxStyles = tv({
  extend: focusRing,
  base: 'w-4.5 h-4.5 box-border shrink-0 rounded-sm flex items-center justify-center border transition',
  variants: {
    isSelected: {
      false:
        'bg-app-surface border-app-border group-pressed:border-app-foreground-muted forced-colors:bg-system-canvas forced-colors:border-system-button-border',
      true: 'bg-app-accent border-app-accent group-pressed:bg-app-accent/80 group-pressed:border-app-accent/80 forced-colors:bg-system-highlight! forced-colors:border-system-highlight!'
    },
    isInvalid: {
      true: 'border-app-danger group-pressed:border-app-danger/80 forced-colors:bg-system-mark! forced-colors:border-system-mark!'
    },
    isDisabled: {
      true: 'bg-app-border/50 border-app-border forced-colors:bg-system-canvas! forced-colors:border-system-gray-text!'
    }
  },
  compoundVariants: [
    {
      isSelected: true,
      isInvalid: true,
      class: 'bg-app-danger border-app-danger group-pressed:bg-app-danger/80 group-pressed:border-app-danger/80'
    }
  ]
});

const iconStyles =
  'w-3.5 h-3.5 text-app-accent-foreground group-disabled:text-app-foreground-muted forced-colors:text-system-highlight-text pointer-events-none';

interface CheckboxProps extends CheckboxFieldProps {
  children?: React.ReactNode;
  description?: string;
  errorMessage?: string | ((validation: ValidationResult) => string);
}

export function Checkbox(props: CheckboxProps) {
  return (
    <CheckboxField {...props} className="flex flex-col gap-1 group">
      <CheckboxButton
        className={composeRenderProps(props.className, (className, renderProps) =>
          checkboxStyles({...renderProps, className})
        )}>
        {composeRenderProps(
          props.children,
          (children, {isSelected, isIndeterminate, ...renderProps}) => (
            <>
              <div
                className={boxStyles({isSelected: isSelected || isIndeterminate, ...renderProps})}>
                {isIndeterminate ? (
                  <Minus aria-hidden className={iconStyles} />
                ) : isSelected ? (
                  <Check aria-hidden className={iconStyles} />
                ) : null}
              </div>
              {children}
            </>
          )
        )}
      </CheckboxButton>
      {props.description && <Description className="ms-6.5">{props.description}</Description>}
      <FieldError className="ms-6.5">{props.errorMessage}</FieldError>
    </CheckboxField>
  );
}
