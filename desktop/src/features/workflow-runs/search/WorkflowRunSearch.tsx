// WorkflowRunSearch consists of two elements
// 1. The text input that gets tokenized as the user types
// 2. The suggestion box that get's filtered as the user types
// This follows the autocomplete example: https://react-aria.adobe.com/TokenField#autocomplete
//
// We use a custom ListBox component to display the suggestions.
// Suggestions are presented horizontally, but keyboard navigation intentionally uses Up/Down so
// Left/Right remain available for moving the caret within the editable token field.

import { Autocomplete } from "react-aria-components/Autocomplete";
import { Token, TokenField } from "@/components/TokenField";
import { useRef, useState } from "react";
import { WorkflowSearchTokenValue } from "@/features/workflow-runs/search/workflow-run-search-token-value";
import { Icons } from "@/components/icons";
import { IconButton } from "@/components/IconButton";
import { SuggestionBar, SuggestionKeyboardHint } from "./SearchSuggestionBar";

export function WorkflowRunSearch() {
  let inputRef = useRef<HTMLDivElement>(null);

  let [value, setValue] = useState(new WorkflowSearchTokenValue([]));

  let activeFilter = value.getActiveFilter();
  if (activeFilter == null && value.segments.length === 0) {
    activeFilter = {
      anchor: value.caretPosition,
      value: "",
      mayBecomeToken: true,
    };
  }
  let hasValue = value.segments.some((segment) => segment.text.length > 0);

  console.log("value segments", value.segments);

  return (
    <Autocomplete>
      {/* Field dor searching */}
      <div className="flex items-start justify-between gap-2 border-b border-app-border px-3">
        <Icons.Search className="mt-4 shrink-0 text-app-foreground-muted" size={20} />
        <TokenField
          value={value}
          onChange={setValue}
          inputRef={inputRef}
          placeholder="Search runs"
          inputClassName="h-14 flex-1 overflow-y-hidden py-4 rounded-none border-0 outline-none ring-0 hover:border-0 hover:outline-none hover:ring-0 focus:border-0 focus:outline-none focus:ring-0 focus-visible:border-0 focus-visible:outline-none focus-visible:ring-0 active:border-0 active:outline-none active:ring-0"
          aria-label="Search workflow runs"
        >
          {(segment) => <Token>{segment.text}</Token>}
        </TokenField>
        {hasValue && (
          <IconButton
            size={12}
            className="mt-3"
            onClick={() => setValue(new WorkflowSearchTokenValue([]))}
          >
            <Icons.X />
          </IconButton>
        )}
      </div>
      <div className="flex h-9.5 w-full shrink-0 overflow-hidden border-b border-app-border px-2">
        {/* Bar underneath for suggestions/errors */}
        <SuggestionBar
          activeFilter={activeFilter}
          inputRef={inputRef}
          setValue={setValue}
          className="h-full min-w-0 flex-1 scrollbar-none rounded-none border-0 bg-transparent"
        />
        <SuggestionKeyboardHint />
      </div>
    </Autocomplete>
  );
}
