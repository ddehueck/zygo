import { Search, X } from "lucide-react";
import { useState } from "react";
import { Autocomplete } from "react-aria-components/Autocomplete";
import { ListBox, ListBoxItem } from "react-aria-components/ListBox";
import type { ValidationResult } from "react-aria-components/SearchField";
import {
  Token as AriaToken,
  TokenField as AriaTokenField,
  TokenInput as AriaTokenInput,
  TokenFieldValue,
  type Position,
  type TokenFieldProps as AriaTokenFieldProps,
  type TokenFieldRenderProps,
  type TokenInputRenderProps,
} from "react-aria-components/TokenField";

import { Description, FieldError, FieldGroup } from "@/components/Field";
import { IconButton } from "@/components/IconButton";
import { composeTailwindRenderProps } from "@/components/utils";
import {
  parseWorkflowSearchQuery,
  tokenizeWorkflowSearchValue,
  WORKFLOW_SEARCH_TOKEN_REGEX,
  type WorkflowSearchToken,
  TokenizingFieldValue,
} from "@/features/workflow-runs/components/search/search-token-field-value";

type SearchBarClassNameRenderProps = TokenFieldRenderProps & {
  defaultClassName: string | undefined;
};

type SearchBarInputClassNameRenderProps = TokenInputRenderProps & {
  defaultClassName: string | undefined;
};

const INVALID_SEARCH_MESSAGE =
  "Use @workflow:<id>, @tag:<name>, or @tag:<name>:<value>. Separate filters with spaces.";

type SearchBarSuggestion = "workflow" | "tag";

const SEARCH_BAR_SUGGESTIONS: readonly {
  type: SearchBarSuggestion;
  tokenPrefix: string;
}[] = [
  { type: "workflow", tokenPrefix: "@workflow:" },
  { type: "tag", tokenPrefix: "@tag:" },
];

export interface SearchBarProps extends Omit<
  AriaTokenFieldProps<TokenizingFieldValue>,
  "children" | "defaultValue" | "onChange" | "value"
> {
  /** Accessible name for the search field. */
  label?: string;
  /** Placeholder text shown when the search field is empty. */
  placeholder?: string;
  /** Additional help text displayed below the search field. */
  description?: string;
  /** Validation message displayed below the search field. */
  errorMessage?: string | ((validation: ValidationResult) => string);
  /** Called whenever the field contains only valid search tokens. */
  onSearchChange?: (tokens: WorkflowSearchToken[]) => void;
}

export function SearchBar({
  label,
  placeholder = "@workflow:id or @tag:name",
  description,
  errorMessage,
  onSearchChange,
  className,
  "aria-label": ariaLabel,
  "aria-labelledby": ariaLabelledby,
  onBlur,
  onKeyDown,
  onSubmit,
  ...props
}: SearchBarProps) {
  const [value, setValue] = useState(
    () => new TokenizingFieldValue([], WORKFLOW_SEARCH_TOKEN_REGEX),
  );
  const [hasValidationError, setHasValidationError] = useState(false);

  const accessibleLabel =
    ariaLabel ?? (ariaLabelledby ? undefined : (label ?? "Search workflow runs"));

  const hasValue = value.toString().trim().length > 0;

  const activeSuggestion = getActiveSuggestion(value);
  const suggestionOptions = getSearchBarSuggestions(activeSuggestion?.query ?? null);

  const handleChange = (nextValue: TokenizingFieldValue) => {
    setValue(nextValue);
    setHasValidationError(false);

    const tokens = parseWorkflowSearchQuery(nextValue.toString());
    if (tokens) {
      const hasUncommittedText = nextValue.segments.some(
        (segment) => segment.type === "text" && segment.text.trim().length > 0,
      );
      if (!hasUncommittedText) {
        onSearchChange?.(tokens);
      }
    }
  };

  const selectSuggestion = (suggestion: SearchBarSuggestion) => {
    if (!activeSuggestion) {
      return;
    }

    const nextValue = value.replaceRange(
      activeSuggestion.start,
      value.selectedRange.current,
      `@${suggestion}:`,
      false,
    );

    handleChange(nextValue);
  };

  const clearValue = () => {
    const emptyValue = new TokenizingFieldValue([], WORKFLOW_SEARCH_TOKEN_REGEX);
    setValue(emptyValue);
    setHasValidationError(false);
    onSearchChange?.([]);
  };

  const rejectInvalidValue = () => {
    const parsedTokens = parseWorkflowSearchQuery(value.toString());
    if (parsedTokens) {
      const committedValue = tokenizeWorkflowSearchValue(value.toString());
      if (committedValue) {
        setValue(committedValue);
        setHasValidationError(false);
        onSearchChange?.(parsedTokens);
      }
      return true;
    }

    // An active autocomplete query is still valid input in progress. React Aria
    // intentionally lets Tab leave the autocomplete without selecting an item.
    if (activeSuggestion && suggestionOptions.length > 0) {
      return false;
    }

    // Keep incomplete input intact so blur does not discard what the user is
    // still editing.
    setHasValidationError(true);
    return false;
  };

  return (
    <Autocomplete filter={() => true}>
      <AriaTokenField
        {...props}
        value={value}
        onChange={handleChange}
        onKeyDown={onKeyDown}
        onBlur={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
            rejectInvalidValue();
          }
          onBlur?.(event);
        }}
        onSubmit={() => {
          if (rejectInvalidValue()) {
            onSubmit?.();
          }
        }}
        aria-label={accessibleLabel}
        aria-labelledby={ariaLabelledby}
        aria-invalid={hasValidationError || undefined}
        role={props.role ?? "searchbox"}
        className={composeTailwindRenderProps<SearchBarClassNameRenderProps>(
          className,
          "flex w-full flex-col gap-1 font-sans text-sm text-app-foreground",
        )}
      >
        <div className="w-full overflow-hidden rounded-lg bg-app-bg-surface forced-colors:bg-system-field">
          <FieldGroup className="w-full rounded-none border-0 bg-transparent shadow-none outline-0 focus-within:border-0">
            <Search
              aria-hidden
              className="ml-2.5 size-4 shrink-0 text-app-foreground-muted group-data-disabled:text-app-foreground-muted/50 forced-colors:text-system-gray-text"
            />
            <AriaTokenInput
              data-placeholder={placeholder}
              className={composeTailwindRenderProps<SearchBarInputClassNameRenderProps>(
                undefined,
                "min-h-9 min-w-0 flex-1 overflow-x-auto whitespace-nowrap px-1.5 py-1 outline-0 placeholder:text-app-foreground-muted/80 empty:before:text-app-foreground-muted/80 empty:before:content-[attr(data-placeholder)]",
              )}
            >
              {(segment) => (
                <AriaToken
                  aria-label={`Search filter ${segment.text}`}
                  className="mx-0.5 inline-flex h-6 items-center rounded-full bg-app-accent/15 px-2 font-mono text-xs text-app-accent data-selected:bg-app-accent data-selected:text-app-accent-foreground"
                >
                  {segment.text}
                </AriaToken>
              )}
            </AriaTokenInput>
            {hasValue && (
              <IconButton
                size={16}
                aria-label="Clear search"
                onPress={clearValue}
                className="mr-1 size-7 shrink-0 rounded-full stroke-0 text-app-foreground-muted hover:text-app-foreground"
              >
                <X aria-hidden className="size-4" />
              </IconButton>
            )}
          </FieldGroup>
          <div className="flex min-h-9 w-full items-center gap-1.5 px-1.5 py-0.5 text-xs">
            {suggestionOptions.length > 0 && (
              <>
                <span className="mr-1 shrink-0 text-app-foreground-muted">Filter by</span>
                <ListBox
                  aria-label="Search suggestions"
                  className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5 outline-0"
                  selectionBehavior="replace"
                  selectionMode="single"
                >
                  {suggestionOptions.map(({ type, tokenPrefix }) => (
                    <ListBoxItem
                      key={type}
                      id={type}
                      aria-label={`Insert ${tokenPrefix} filter`}
                      textValue={tokenPrefix}
                      onAction={() => selectSuggestion(type)}
                      className="cursor-default rounded-md px-2 py-1 font-mono text-xs text-app-accent outline-0 data-focused:bg-app-accent/10 data-hovered:bg-app-accent/10 data-pressed:bg-app-accent/15"
                    >
                      {tokenPrefix}
                    </ListBoxItem>
                  ))}
                </ListBox>
              </>
            )}
          </div>
        </div>
        {description && <Description>{description}</Description>}
        {(hasValidationError || errorMessage) && (
          <FieldError>
            {hasValidationError
              ? INVALID_SEARCH_MESSAGE
              : typeof errorMessage === "string"
                ? errorMessage
                : null}
          </FieldError>
        )}
      </AriaTokenField>
    </Autocomplete>
  );
}

function getSearchBarSuggestions(query: string | null) {
  if (!query) {
    return [];
  }

  const normalizedQuery = query.toLowerCase();
  return SEARCH_BAR_SUGGESTIONS.filter(({ tokenPrefix }) =>
    tokenPrefix.startsWith(normalizedQuery),
  );
}

function getActiveSuggestion(value: TokenizingFieldValue): {
  start: Position;
  query: string;
} | null {
  const currentPosition = value.selectedRange.current;
  const start = value.findText(currentPosition, TokenFieldValue.Direction.Backward, /(?<=^|\s)@/);
  if (start === null) {
    return null;
  }

  const query = value.slice(start, currentPosition).toString();
  if (query.includes(":")) {
    return null;
  }

  return { start, query };
}
