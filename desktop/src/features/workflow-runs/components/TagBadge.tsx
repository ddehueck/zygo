import { Icon, iconDefinitions } from "@/components/icons";
import type { Maybe } from "@/utils";

interface TagBadgeProps {
  name: string;
  value: Maybe<string>;
  includeIcon?: boolean;
}

export function TagBadge({ name, value, includeIcon = false }: TagBadgeProps) {
  const displayValue = value ?? "—";

  return (
    <span
      title={`${name}: ${displayValue}`}
      aria-label={`${name}: ${displayValue}`}
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
      <span className="truncate text-app-foreground-muted">{name}</span>
      <span aria-hidden="true" className="text-app-foreground-muted">
        :
      </span>
      <span className={value == null ? "text-app-foreground-muted" : "text-app-foreground"}>
        {displayValue}
      </span>
    </span>
  );
}
