import {
  TokenField as AriaTokenField,
  TokenInput as AriaTokenInput,
  Token as AriaToken,
  type TokenFieldProps as AriaTokenFieldProps,
  type TokenInputProps,
  type TokenProps,
  type TokenFieldValue,
} from "react-aria-components/TokenField";
import { Label } from "react-aria-components/Label";
import { Text } from "react-aria-components/Text";
import type React from "react";

export interface TokenFieldProps<T extends TokenFieldValue = TokenFieldValue> extends Omit<
  AriaTokenFieldProps<T>,
  "children"
> {
  label?: string;
  description?: string;
  placeholder?: string;
  inputRef?: React.Ref<HTMLDivElement>;
  children: TokenInputProps["children"];
}

export function TokenField<T extends TokenFieldValue = TokenFieldValue>({
  label,
  description,
  placeholder,
  inputRef,
  children,
  ...props
}: TokenFieldProps<T>) {
  return (
    <AriaTokenField {...props}>
      {label && <Label>{label}</Label>}
      <AriaTokenInput
        ref={inputRef}
        data-placeholder={placeholder}
        className="react-aria-TokenInput inset"
      >
        {children}
      </AriaTokenInput>
      {description && <Text slot="description">{description}</Text>}
    </AriaTokenField>
  );
}

export function Token(props: TokenProps) {
  return <AriaToken {...props} />;
}
