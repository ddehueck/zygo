import {type FieldErrorProps, FieldError as RACFieldError} from 'react-aria-components/FieldError';
import {Group, type GroupProps} from 'react-aria-components/Group';
import {type InputProps, Input as RACInput} from 'react-aria-components/Input';
import {type LabelProps, Label as RACLabel} from 'react-aria-components/Label';
import {Text, type TextProps} from 'react-aria-components/Text';
import {composeRenderProps} from 'react-aria-components/composeRenderProps';
import {twMerge} from 'tailwind-merge';
import {tv} from 'tailwind-variants';
import {composeTailwindRenderProps, focusRing} from './utils';

export function Label(props: LabelProps) {
  return (
    <RACLabel
      {...props}
      className={twMerge(
        'font-sans text-sm text-app-foreground-muted font-medium cursor-default w-fit',
        props.className
      )}
    />
  );
}

export function Description(props: TextProps) {
  return (
    <Text
      {...props}
      slot="description"
      className={twMerge(
        'text-xs text-app-foreground-muted group-disabled:text-app-foreground-muted/50 contain-inline-size',
        props.className
      )}
    />
  );
}

export function FieldError(props: FieldErrorProps) {
  return (
    <RACFieldError
      {...props}
      className={composeTailwindRenderProps(
        props.className,
        'text-xs text-app-danger contain-inline-size forced-colors:text-system-mark'
      )}
    />
  );
}

export const fieldBorderStyles = tv({
  base: 'transition',
  variants: {
    isFocusWithin: {
      false:
        'border-app-border hover:border-app-foreground-muted forced-colors:border-system-button-border',
      true: 'border-app-accent forced-colors:border-system-highlight'
    },
    isInvalid: {
      true: 'border-app-danger forced-colors:border-system-mark'
    },
    isDisabled: {
      true: 'border-app-border/50 forced-colors:border-system-gray-text'
    }
  }
});

export const fieldGroupStyles = tv({
  extend: focusRing,
  base: 'group flex items-center h-9 box-border bg-app-surface forced-colors:bg-system-field border rounded-lg overflow-hidden transition',
  variants: fieldBorderStyles.variants
});

export function FieldGroup(props: GroupProps) {
  return (
    <Group
      {...props}
      className={composeRenderProps(props.className, (className, renderProps) =>
        fieldGroupStyles({...renderProps, className})
      )}
    />
  );
}

export function Input(props: InputProps) {
  return (
    <RACInput
      {...props}
      className={composeTailwindRenderProps(
        props.className,
        'px-3 py-0 min-h-9 flex-1 min-w-0 border-0 outline outline-0 bg-app-surface font-sans text-sm text-app-foreground placeholder:text-app-foreground-muted disabled:text-app-foreground-muted/50 disabled:placeholder:text-app-foreground-muted/50 [-webkit-tap-highlight-color:transparent]'
      )}
    />
  );
}
