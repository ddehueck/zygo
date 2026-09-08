import { type RefObject } from "react";

import { SearchSuggestionBar } from "@/components/search";
import { LogSearchTokenValue } from "./log-search-token-value";
import type { LogSearchSuggestion } from "./types";
import { useLogSearchSuggestions } from "./use-log-search-suggestions";

export function LogSearchSuggestionBar({
  workflowRunId,
  value,
  inputRef,
  setValue,
  className,
}: {
  workflowRunId: number;
  value: LogSearchTokenValue;
  inputRef: RefObject<HTMLDivElement | null>;
  setValue: (value: LogSearchTokenValue) => void;
  className?: string;
}) {
  const activeFilter = value.getActiveInputText();
  const searchString = activeFilter?.value ?? "";
  const isInvalid = activeFilter?.mayBecomeToken === false;
  const hasJobFilter = value
    .getFilterValues()
    .some((filter) => filter.entity === "job_run");

  const { suggestions } = useLogSearchSuggestions({
    workflowRunId,
    searchString,
    limit: 10,
    hasJobFilter,
  });

  const insertItem = (item: LogSearchSuggestion) => {
    const filterAnchor = activeFilter?.anchor;
    if (filterAnchor == null) return;

    const next = value.acceptSuggestion({
      suggestion: item,
      anchor: filterAnchor,
      end: value.selectedRange.current,
    });
    setValue(next);
    inputRef.current?.focus();
  };

  return (
    <SearchSuggestionBar
      items={suggestions}
      isInvalid={isInvalid}
      className={className}
      onSelect={insertItem}
    />
  );
}
