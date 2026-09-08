import { type RefObject } from "react";

import { SearchSuggestionBar } from "@/components/search";
import { WorkflowSearchTokenValue } from "@/features/workflow-runs/search/workflow-run-search-token-value";
import { type WorkflowRunSearchSuggestion } from "./types";
import { useWorkflowRunsSearchSuggestions } from "./use-workflow-runs-search-suggestions";
import { getFilterValue } from "./parse";

export function SuggestionBar({
  value,
  inputRef,
  setValue,
  className,
}: {
  value: WorkflowSearchTokenValue;
  inputRef: RefObject<HTMLDivElement | null>;
  setValue: (value: WorkflowSearchTokenValue) => void;
  className?: string;
}) {
  const activeFilter = value.getActiveInputText();
  const searchString = activeFilter?.value ?? "";
  const isInvalid = activeFilter?.mayBecomeToken === false;

  const { suggestions } = useWorkflowRunsSearchSuggestions({
    filterValue: getFilterValue(searchString),
    limit: 10,
  });

  const filteredSuggestions = suggestions.filter((item) =>
    item.text.toLocaleLowerCase().includes(searchString.toLocaleLowerCase()),
  );

  const insertItem = (item: WorkflowRunSearchSuggestion) => {
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
      items={filteredSuggestions}
      isInvalid={isInvalid}
      className={className}
      onSelect={insertItem}
    />
  );
}

export { SuggestionKeyboardHint } from "@/components/search";
