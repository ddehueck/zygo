import {
  ListBox as AriaListBox,
  ListBoxItem as AriaListBoxItem,
} from "react-aria-components/ListBox";
import { cn, focusRing } from "@/components/utils";
import { tv } from "tailwind-variants";

export type SearchSuggestionItem = {
  id: string;
  text: string;
};

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

/**
 * Horizontal suggestion chips with vertical keyboard navigation so Left/Right
 * stay available for the TokenField caret.
 */
export function SearchSuggestionBar<T extends SearchSuggestionItem>({
  items,
  isInvalid = false,
  className,
  onSelect,
}: {
  items: T[];
  isInvalid?: boolean;
  className?: string;
  onSelect: (item: T) => void;
}) {
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

  if (items.length === 0) {
    return (
      <p className={cn("flex items-center p-1 text-xs text-app-foreground-muted", className)}>
        No suggestions available
      </p>
    );
  }

  return (
    <AriaListBox
      id="suggestion-bar"
      items={items}
      layout="stack"
      orientation="vertical"
      className={cn("flex min-w-0 scroll-p-1 items-center gap-1 overflow-x-auto p-1", className)}
      selectionMode="single"
      selectedKeys={[]}
      onSelectionChange={(keys) => {
        if (keys === "all") return;
        const key = keys.values().next().value;
        const item = items.find((suggestion) => suggestion.id === key);
        if (item) onSelect(item);
      }}
    >
      {(item) => <SuggestionItem id={item.id} text={item.text} />}
    </AriaListBox>
  );
}

function SuggestionItem({ id, text }: { id: string; text: string }) {
  return (
    <AriaListBoxItem
      id={id}
      textValue={text}
      className={suggestionItemStyles}
      // Keep the token field focused so its updated caret position is applied to the DOM.
      onMouseDown={(event) => event.preventDefault()}
    >
      {text}
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
