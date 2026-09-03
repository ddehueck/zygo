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
import { TokenFieldValue } from "react-aria-components/TokenField";
import { useMemo, useRef, useState } from "react";
import { ListBox, ListBoxItem } from "@/components/ListBox";

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
    new TokenFieldValue([
      { type: "text", text: "This example has autocomplete for " },
      { type: "token", text: "@usernames" },
      { type: "text", text: " and " },
      { type: "token", text: "/commands" },
    ]),
  );

  let [filterAnchor, filterValue] = useMemo(() => {
    let currentPosition = value.selectedRange.current;
    let filterAnchor = value.findText(
      currentPosition,
      TokenFieldValue.Direction.Backward,
      /(?<=^|\s)[@/]/,
    );
    if (filterAnchor != null) {
      let filterValue = value.slice(filterAnchor, currentPosition).toString();
      return [filterAnchor, filterValue];
    }
    return [null, null];
  }, [value]);

  let items: Item[] = [];
  if (filterValue != null && filterValue.startsWith("/")) {
    items = slashCommands.filter((item) => item.command.includes(filterValue.slice(1)));
  } else if (filterValue != null && filterValue.startsWith("@")) {
    items = usernames.filter((item) => item.username.includes(filterValue.slice(1)));
  }

  let insertItem = (item: Item) => {
    if (filterAnchor == null) {
      return;
    }

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
    <Autocomplete>
      <TokenField
        value={value}
        onChange={setValue}
        onKeyDown={(event) => {
          // Multiline token inputs otherwise insert a newline after selecting a suggestion.
          if (
            event.key === "Enter" &&
            inputRef.current?.getAttribute("aria-activedescendant") != null
          ) {
            event.preventDefault();
          }
        }}
        allowsNewlines
        inputRef={inputRef}
      >
        {(segment) => <Token>{segment.text}</Token>}
      </TokenField>
      <ListBox
        items={items}
        dependencies={[filterAnchor]}
        layout="stack"
        orientation="vertical"
        style={{ flexDirection: "row", overflowX: "auto" }}
        selectionMode="single"
        shouldFocusWrap
        selectedKeys={[]}
        onSelectionChange={(keys) => {
          if (keys === "all") {
            return;
          }

          let key = keys.values().next().value;
          let item = items.find((item) =>
            "username" in item ? item.username === key : item.command === key,
          );
          if (item) {
            insertItem(item);
          }
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
  );
}
