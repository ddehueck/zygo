import { TokenSegment } from "react-aria-components/TokenField";
import { WorkflowRunFilter } from "./types";
import { err, ok, Result } from "@/lib/result";
import { assertNever } from "@/utils";

const FILTER_SIGIL = "@";
const FILTER_DELIMITER = ":";

const FILTER_PREFIXS = {
  workflow: "workflow",
  tag: "tag",
};

const filterPrefix = (prefix: string) => `${FILTER_SIGIL}${prefix}${FILTER_DELIMITER}`;

export function toFilter(text: string): Result<WorkflowRunFilter, string> {
  // Check that the text starts with the filter sigil
  if (!text.startsWith(FILTER_SIGIL)) return err("Filter text must start with @ symbol");

  // Extract the prefix from the text after the filter sigil
  const prefixStrings = Object.values(FILTER_PREFIXS);
  const prefixRegex = new RegExp(`^(${prefixStrings.map(filterPrefix).join("|")})`);
  const match = text.match(prefixRegex);
  if (!match)
    return err(`Filter must include a valid prefix (${prefixStrings.join(", ").slice(0, 4)}...)`);

  // Extract the prefix from the text and build the filter
  const prefix = match[1].slice(FILTER_SIGIL.length, -FILTER_DELIMITER.length);
  const rest = text.slice(match[0].length).trim();
  if (!rest) return err("Filter must include a value");

  switch (prefix) {
    case FILTER_PREFIXS.workflow:
      return ok({ entity: "workflow", id: rest });
    case FILTER_PREFIXS.tag:
      const [name, value] = rest.split(":");
      return ok({ entity: "tag", name, value });
    default:
      return err("Unknown filter prefix");
  }
}

export function toTokenSegment(filter: WorkflowRunFilter): TokenSegment {
  let text: string;
  switch (filter.entity) {
    case "workflow":
      text = `${filterPrefix(filter.entity)}${filter.id}`;
      break;
    case "tag":
      text = filter.value
        ? `${filterPrefix(filter.entity)}${filter.name}:${filter.value}`
        : `${filterPrefix(filter.entity)}${filter.name}`;
      break;
    default:
      assertNever(filter);
  }
  return { type: "token", text, value: filter };
}
