// TokenSearch consists of two elements
// 1. The text input that gets tokenized as the user types
// 2. The suggestion box that get's filtered as the user types
// This follows the autocomplete example: https://react-aria.adobe.com/TokenField#autocomplete
//
// We use a custom ListBox component to display the suggestions.
// Suggestions are presented horizontally, but keyboard navigation intentionally uses Up/Down so
// Left/Right remain available for moving the caret within the editable token field.
// TODO: Add a bottom-right indicator that Up/Down navigate suggestions.
import { Autocomplete } from "react-aria-components/Autocomplete";
import { Text } from "react-aria-components/Text";
import { Token, TokenField } from "./TokenField";
import { useRef, useState } from "react";
import { ListBox, ListBoxItem } from "@/components/ListBox";
import { WorkflowSearchTokenValue } from "./workflow-search-token-value";

type Item = { username: string } | { command: string; description: string };

const usernames = [
  { username: "alexmiller" },
  { username: "sarahjones" },
  { username: "davidkim" },
  { username: "emmawatson" },
  { username: "oliverliu" },
  { username: "ellagreen" },
  { username: "lucasbrown" },
  { username: "amandarivera" },
  { username: "masonlee" },
  { username: "nataliasmith" },
  { username: "benjamintaylor" },
  { username: "zoewilson" },
  { username: "henrywalker" },
  { username: "madelineyoung" },
  { username: "noahscott" },
  { username: "lucygonzalez" },
  { username: "jacobmartin" },
  { username: "averymoore" },
  { username: "loganmurphy" },
  { username: "miahernandez" },
  { username: "danieladair" },
  { username: "sofiacox" },
  { username: "jackharris" },
  { username: "chloebaker" },
  { username: "liamrodriguez" },
];

const slashCommands = [
  { command: "gif", description: "Insert a GIF" },
  { command: "todo", description: "Add a todo list item" },
  { command: "mention", description: "Mention a user with @username" },
  { command: "date", description: "Insert the current date" },
  { command: "quote", description: "Insert a quote block" },
];

export function TokenSearch() {
  let inputRef = useRef<HTMLDivElement>(null);

  let [value, setValue] = useState(
    new WorkflowSearchTokenValue([
      { type: "token", text: "@workflow:name", value: "workflow" },
      { type: "text", text: " " },
      { type: "token", text: "@tag:name", value: "tag" },
      { type: "text", text: " " },
    ]).withCaretPosition({ index: 3, offset: 1 }),
  );

  let activeFilter = value.getActiveFilter();
  let filterAnchor = activeFilter?.anchor ?? null;
  let filterValue = activeFilter?.value ?? null;

  // TODO: This should be a hook that accepts the token field value class instance
  // we'll include default suggestions and suggestions relative to the current value
  let suggestions: Item[] = [];
  if (filterValue != null && filterValue.startsWith("/")) {
    suggestions = slashCommands.filter((item) => item.command.includes(filterValue.slice(1)));
  } else if (filterValue != null && filterValue.startsWith("@")) {
    suggestions = usernames.filter((item) => item.username.includes(filterValue.slice(1)));
  }

  let insertItem = (item: Item) => {
    if (filterAnchor == null) return;

    setValue((value) =>
      value.replaceRangeWithSegments(
        filterAnchor,
        value.selectedRange.current,
        [
          {
            type: "token",
            text: "username" in item ? "@" + item.username : "/" + item.command,
          },
          { type: "text", text: " " },
        ],
        false,
      ),
    );
  };

  return (
    <div className="p-2">
      <Autocomplete>
        <TokenField
          value={value}
          onChange={setValue}
          inputRef={inputRef}
          aria-label="Search workflow runs"
          onKeyDown={(event) => {
            // Multiline token inputs otherwise insert a newline after selecting a suggestion.
            if (
              event.key === "Enter" &&
              inputRef.current?.getAttribute("aria-activedescendant") != null
            ) {
              event.preventDefault();
            }
          }}
        >
          {(segment) => <Token>{segment.text}</Token>}
        </TokenField>
        <ListBox
          items={suggestions}
          dependencies={[filterAnchor]}
          layout="stack"
          orientation="vertical"
          style={{ flexDirection: "row", overflowX: "auto" }}
          className={"w-full scrollbar-none"}
          selectionMode="single"
          selectedKeys={[]}
          onSelectionChange={(keys) => {
            if (keys === "all") return;

            let key = keys.values().next().value;
            let item = suggestions.find((item) =>
              "username" in item ? item.username === key : item.command === key,
            );
            if (item) insertItem(item);
          }}
        >
          {(item) => (
            <ListBoxItem
              id={"username" in item ? item.username : item.command}
              textValue={"username" in item ? item.username : item.command}
            >
              <Text slot="label">{"username" in item ? item.username : item.command}</Text>
              {"description" in item ? <Text slot="description">{item.description}</Text> : null}
            </ListBoxItem>
          )}
        </ListBox>
      </Autocomplete>
      <p> {activeFilter?.mayBecomeToken === false && "invalid?"} </p>
    </div>
  );
}
