import { Autocomplete } from "react-aria-components/Autocomplete";
import { useRef } from "react";

import { Icon, iconDefinitions } from "@/components/icons";
import { IconButton } from "@/components/IconButton";
import { SuggestionKeyboardHint } from "@/components/search";
import { Token, TokenField } from "@/components/TokenField";
import { useLogSearchContext } from "../search/LogSearchContext";
import { LogSearchSuggestionBar } from "../search/LogSearchSuggestionBar";

export function Search({ workflowRunId }: { workflowRunId: number }) {
  const inputRef = useRef<HTMLDivElement>(null);
  const { value, setValue, clear } = useLogSearchContext();
  const hasValue = value.segments.some((segment) => segment.text.length > 0);

  return (
    <Autocomplete>
      <div className="flex shrink-0 items-start gap-2 border-b border-app-border bg-app-bg-elevated px-3">
        <Icon
          definition={iconDefinitions.search}
          className="mt-3 shrink-0 text-app-foreground-muted"
          size={16}
        />
        <TokenField
          value={value}
          onChange={setValue}
          inputRef={inputRef}
          placeholder="Search logs"
          inputClassName="min-h-10 flex-1 overflow-y-hidden rounded-none border-0 bg-transparent px-0 py-2 outline-none ring-0 hover:border-0 hover:outline-none hover:ring-0 focus:border-0 focus:outline-none focus:ring-0 focus-visible:border-0 focus-visible:outline-none focus-visible:ring-0 active:border-0 active:outline-none active:ring-0"
          aria-label="Search logs"
        >
          {(segment) => <Token>{segment.text}</Token>}
        </TokenField>
        {hasValue && (
          <IconButton size={12} className="mt-3" onClick={clear} aria-label="Clear log search">
            <Icon definition={iconDefinitions.close} size={12} aria-hidden />
          </IconButton>
        )}
      </div>
      <div className="flex h-9 w-full shrink-0 overflow-hidden border-b border-app-border bg-app-bg-elevated px-2">
        <LogSearchSuggestionBar
          workflowRunId={workflowRunId}
          value={value}
          inputRef={inputRef}
          setValue={setValue}
          className="h-full min-w-0 flex-1 scrollbar-none rounded-none border-0 bg-transparent"
        />
        <SuggestionKeyboardHint />
      </div>
    </Autocomplete>
  );
}
