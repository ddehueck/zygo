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
      className="inline-flex max-w-full items-center gap-1.5 rounded-md border border-app-border bg-app-bg-surface px-2.5 py-1.5 text-sm shadow-sm"
    >
      {includeIcon && (
        <Icons.Tag
          aria-hidden="true"
          className="size-3.5 shrink-0 text-app-accent"
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
