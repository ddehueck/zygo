// TokenSearch consists of two elements
// 1. The text input that gets tokenized as the user types
// 2. The suggestion box that get's filtered as the user types
// This follows the autocomplete example: https://react-aria.adobe.com/TokenField#autocomplete
//
// We use a custom ListBox component to display the suggestions.
// Suggestions are presented horizontally, but keyboard navigation intentionally uses Up/Down so
// Left/Right remain available for moving the caret within the editable token field.

import { Autocomplete } from "react-aria-components/Autocomplete";
import {
  ListBox as AriaListBox,
  ListBoxItem as AriaListBoxItem,
} from "react-aria-components/ListBox";
import { Token, TokenField } from "./TokenField";
import { type Dispatch, type SetStateAction, useRef, useState } from "react";
import { WorkflowSearchTokenValue } from "./workflow-search-token-value";
import { cn, focusRing } from "@/components/utils";
import { tv } from "tailwind-variants";
import { Icons } from "@/components/icons";
import { IconButton } from "@/components/IconButton";

type Suggestion = { tokenPrefix: string }; // todo: based on actual data

const tokenPrefixes = [{ tokenPrefix: "@workflow" }, { tokenPrefix: "@tag" }];

export function TokenSearch() {
  let inputRef = useRef<HTMLDivElement>(null);

  let [value, setValue] = useState(
    new WorkflowSearchTokenValue([
      { type: "token", text: "@workflow:name", value: "workflow" },
      { type: "token", text: "@tag:name", value: "tag" },
    ]).withCaretPosition({ index: 2, offset: 1 }),
  );

  let activeFilter = value.getActiveFilter();
  let hasValue = value.segments.some((segment) => segment.text.length > 0);

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
      <div className="relative h-9.5 w-full shrink-0 border-b border-app-border px-2">
        {/* Bar underneath for suggestions/errors */}
        <SuggestionBar
          activeFilter={activeFilter}
          setValue={setValue}
          className="h-full w-full shrink-0 scrollbar-none rounded-none border-0 bg-transparent pr-24"
        />
        <SuggestionKeyboardHint />
      </div>
    </Autocomplete>
  );
}

function SuggestionBar({
  activeFilter,
  setValue,
  className,
}: {
  activeFilter: ReturnType<WorkflowSearchTokenValue["getActiveFilter"]>;
  setValue: Dispatch<SetStateAction<WorkflowSearchTokenValue>>;
  className?: string;
}) {
  let isInvalid = activeFilter?.mayBecomeToken === false;

  let filterValue = activeFilter?.value;
  let filterAnchor = activeFilter?.anchor;

  let suggestions: Suggestion[] = [];
  if (filterValue)
    suggestions = tokenPrefixes.filter((item) => item.tokenPrefix.includes(filterValue.slice(1)));

  let insertItem = (item: Suggestion) => {
    if (filterAnchor == null) return;

    setValue((value) =>
      value.replaceRangeWithSegments(
        filterAnchor,
        value.selectedRange.current,
        [
          {
            type: "text",
            text: item.tokenPrefix,
          },
        ],
        false,
      ),
    );
  };

  if (isInvalid) {
    return (
      <p
        className={cn(
          "flex items-center p-1 text-xs text-app-danger/70 forced-colors:text-system-mark",
          className,
        )}
        role="alert"
      >
        Invalid search token
      </p>
    );
  }

  return (
    <AriaListBox
      id="suggestion-bar"
      items={suggestions}
      dependencies={[filterAnchor]}
      layout="stack"
      orientation="horizontal"
      className={cn("flex scroll-p-1 items-center gap-1 overflow-x-auto p-1", className)}
      selectionMode="single"
      selectedKeys={[]}
      onSelectionChange={(keys) => {
        if (keys === "all") return;

        let key = keys.values().next().value;
        let item = suggestions.find((item) => item.tokenPrefix === key);
        if (item) insertItem(item);
      }}
    >
      {(item) => <SuggestionItem id={item.tokenPrefix} item={item} />}
    </AriaListBox>
  );
}

export const suggestionItemStyles = tv({
  extend: focusRing,
  base: "group relative flex h-6 shrink-0 cursor-default items-center rounded-full border border-app-accent bg-transparent px-2 align-middle font-mono text-xs transition will-change-transform forced-color-adjust-none select-none",
  variants: {
    isSelected: {
      false:
        "pressed:bg-app-accent pressed:text-app-accent-foreground text-app-accent hover:bg-app-accent hover:text-app-accent-foreground",
      true: "border-app-accent bg-app-accent text-app-accent-foreground -outline-offset-4 outline-app-accent-foreground forced-colors:bg-system-highlight forced-colors:text-system-highlight-text forced-colors:outline-system-highlight-text",
    },
    isDisabled: {
      true: "border-app-border/50 bg-transparent text-app-foreground-muted forced-colors:bg-system-canvas forced-colors:text-system-gray-text",
    },
    isFocused: {
      true: "bg-app-accent text-app-accent-foreground forced-colors:bg-system-highlight forced-colors:text-system-highlight-text",
    },
  },
});

function SuggestionItem({ item, id }: { item: Suggestion; id: string }) {
  return (
    <AriaListBoxItem id={id} textValue={item.tokenPrefix} className={suggestionItemStyles}>
      {item.tokenPrefix}
    </AriaListBoxItem>
  );
}

function SuggestionKeyboardHint() {
  return (
    <div className="pointer-events-none absolute top-1/2 right-3 flex -translate-y-1/2 items-center gap-0.5 text-xs text-app-foreground-muted">
      <span className="sr-only">Use the up and down arrow keys to select a suggestion</span>
      <span aria-hidden="true" className="flex items-center gap-0.5">
        <kbd className="inline-flex size-3.5 items-center justify-center rounded-sm border border-app-border bg-app-bg-surface font-sans text-xs leading-none">
          ↑
        </kbd>
        <kbd className="inline-flex size-3.5 items-center justify-center rounded-sm border border-app-border bg-app-bg-surface font-sans text-xs leading-none">
          ↓
        </kbd>
        <span className="ml-1">Select</span>
      </span>
    </div>
  );
}
