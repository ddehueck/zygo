import { OverflowList } from "react-responsive-overflow-list";
import type { Tag } from "../../../bindings";
import { TagBadge } from "./TagBadge";

type TagOverflowListProps = { tags: Tag[] };

export function TagOverflowList({ tags }: TagOverflowListProps) {
  return (
    <OverflowList
      items={tags}
      maxRows={1}
      className="w-full min-w-0 justify-end gap-1"
      renderItem={(tag) => <TagBadge name={tag.key} value={tag.value} includeIcon />}
      renderOverflow={(hiddenTags) => (
        <span
          className="inline-flex shrink-0 items-center rounded-md border border-app-border bg-app-bg-surface px-2 py-1 font-mono text-xs leading-none whitespace-nowrap text-app-foreground-muted"
          title={hiddenTags.map((tag) => `${tag.key}: ${tag.value}`).join(", ")}
          aria-label={`${hiddenTags.length} more tag${hiddenTags.length === 1 ? "" : "s"}: ${hiddenTags
            .map((tag) => `${tag.key}: ${tag.value}`)
            .join(", ")}`}
        >
          +{hiddenTags.length} more
        </span>
      )}
    />
  );
}
