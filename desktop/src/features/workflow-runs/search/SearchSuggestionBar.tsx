import {
  ListBox as AriaListBox,
  ListBoxItem as AriaListBoxItem,
} from "react-aria-components/ListBox";
import { type RefObject } from "react";
import { cn, focusRing } from "@/components/utils";
import { tv } from "tailwind-variants";
import { WorkflowSearchTokenValue } from "@/features/workflow-runs/search/workflow-run-search-token-value";
import { type WorkflowRunSearchSuggestion } from "./types";
import { useWorkflowRunsSearchSuggestions } from "./use-workflow-runs-search-suggestions";
import { getFilterValue } from "./parse";

const suggestionItemStyles = tv({
  extend: focusRing,
  base: "group relative flex h-6 shrink-0 cursor-default items-center rounded-full bg-transparent px-2 align-middle font-mono text-xs transition will-change-transform forced-color-adjust-none select-none",
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
  let activeFilter = value.getActiveInputText();
  let searchString = activeFilter?.value ?? "";
  let isInvalid = activeFilter?.mayBecomeToken === false;

  let { suggestions } = useWorkflowRunsSearchSuggestions({
    filterValue: getFilterValue(searchString),
    limit: 10,
  });

  let filteredSuggestions = suggestions.filter((item) =>
    item.text.toLocaleLowerCase().includes(searchString),
  );

  let insertItem = (item: WorkflowRunSearchSuggestion) => {
    let filterAnchor = activeFilter?.anchor;
    console.log("has filterAnchor", !!filterAnchor);
    if (filterAnchor == null) return;

    const newSuggestion = value.acceptSuggestion({
      suggestion: item,
      anchor: filterAnchor,
      end: value.selectedRange.current,
    });
    setValue(newSuggestion);
    // Restore focus to the input so the updated caret position is applied to the DOM.
    inputRef.current?.focus();
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
      items={filteredSuggestions}
      layout="stack"
      orientation="vertical" // We use vertical so only up and down arrow keys move through suggestions
      className={cn("flex min-w-0 scroll-p-1 items-center gap-1 overflow-x-auto p-1", className)}
      selectionMode="single"
      selectedKeys={[]}
      onSelectionChange={(keys) => {
        console.log("selection change", keys, filteredSuggestions);
        if (keys === "all") return;
        let key = keys.values().next().value;
        let item = filteredSuggestions.find((item) => item.id === key);
        console.log("selected item", item);
        if (item) insertItem(item);
      }}
    >
      {(item) => <SuggestionItem id={item.id} item={item} />}
    </AriaListBox>
  );
}

function SuggestionItem({ item, id }: { item: WorkflowRunSearchSuggestion; id: string }) {
  return (
    <AriaListBoxItem
      id={id}
      textValue={item.text}
      className={suggestionItemStyles}
      // Keep the token field focused so its updated caret position is applied to the DOM.
      onMouseDown={(event) => event.preventDefault()}
    >
      {item.text}
    </AriaListBoxItem>
  );
}

export function SuggestionKeyboardHint() {
  return (
    <div className="bg-app-bg flex h-full w-22 shrink-0 items-center justify-end gap-0.5 px-2 text-xs text-app-foreground-muted">
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
