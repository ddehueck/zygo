import { Icons } from "../../../components/icons";
import type { Maybe } from "../../../utils";

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
      className="inline-flex max-w-full shrink-0 items-center rounded-md border border-app-border bg-app-bg-surface px-2 py-1 font-mono text-xs leading-none whitespace-nowrap"
    >
      {includeIcon && (
        <Icons.Tag
          aria-hidden="true"
          className="size-3 shrink-0 pr-0.5 text-app-accent"
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
