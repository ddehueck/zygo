import { Icon, iconDefinitions } from "@/components/icons";
import { Text } from "@/components/Text";

interface TagBadgeProps {
  value: string;
  includeIcon?: boolean;
}

export function TagBadge({ value, includeIcon = false }: TagBadgeProps) {
  return (
    <span
      title={value}
      aria-label={value}
      className="inline-flex max-w-full shrink-0 items-center rounded-md border border-app-border bg-app-bg-surface px-1 py-1 font-mono text-xs leading-none whitespace-nowrap"
    >
      {includeIcon && (
        <Icon
          definition={iconDefinitions.tag}
          aria-hidden="true"
          className="size-3 shrink-0 fill-current pr-0.75 text-app-accent [&>circle]:fill-app-bg-surface [&>circle]:stroke-app-bg-surface"
          strokeWidth={2}
        />
      )}
      <Text size="small" className="truncate">
        {value}
      </Text>
    </span>
  );
}
