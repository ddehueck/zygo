import { useEffect, useRef, useState, type ReactNode } from "react";

import { Icon, iconDefinitions } from "@/components/icons";

import { IconButton } from "./IconButton";
import { Text } from "./Text";

type CopyableTextProps = {
  value: string;
  children?: ReactNode;
  label?: string;
  className?: string;
  onTextPress?: () => void | Promise<void>;
};

export function CopyableText({
  value,
  children,
  label = "text",
  className,
  onTextPress,
}: CopyableTextProps) {
  const [copied, setCopied] = useState(false);
  const resetTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (resetTimeout.current) clearTimeout(resetTimeout.current);
    };
  }, []);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      return;
    }

    setCopied(true);

    if (resetTimeout.current) clearTimeout(resetTimeout.current);
    resetTimeout.current = setTimeout(() => setCopied(false), 1600);
  }

  const text = (
    <Text size="small" className={`min-w-0 flex-1 font-mono break-all ${className ?? ""}`}>
      {children ?? value}
    </Text>
  );

  return (
    <span className="flex min-w-0 items-start gap-1.5">
      {onTextPress ? (
        <button
          type="button"
          aria-label={`Reveal ${label} in directory`}
          className="min-w-0 flex-1 cursor-default text-left outline-offset-2 hover:underline focus-visible:outline focus-visible:outline-1 focus-visible:outline-app-accent"
          onClick={() => {
            void Promise.resolve(onTextPress()).catch(() => undefined);
          }}
        >
          {text}
        </button>
      ) : (
        text
      )}
      <IconButton
        aria-label={copied ? `${label} copied` : `Copy ${label}`}
        className="shrink-0"
        onPress={handleCopy}
      >
        <span className="relative flex size-4 items-center justify-center">
          <Icon
            aria-hidden
            className={`absolute size-4 transition-[opacity,transform] duration-150 ease-in-out motion-reduce:transition-none ${
              copied ? "scale-75 opacity-0" : "scale-100 opacity-100"
            }`}
            definition={iconDefinitions.copy}
          />
          <Icon
            aria-hidden
            className={`absolute size-4 text-app-accent transition-[opacity,transform] duration-150 ease-in-out motion-reduce:transition-none ${
              copied ? "scale-100 opacity-100" : "scale-75 opacity-0"
            }`}
            definition={iconDefinitions.check}
          />
        </span>
      </IconButton>
    </span>
  );
}
