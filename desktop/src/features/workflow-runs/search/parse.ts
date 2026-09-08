import { TokenSegment } from "react-aria-components/TokenField";
import { filterPrefix, FILTER_DELIMITER, FILTER_SIGIL } from "@/components/search";
import { err, ok, Result } from "@/lib/result";
import { assertNever } from "@/utils";
import { WorkflowRunFilter } from "./types";

const FILTER_PREFIXS = {
  workflow: "workflow",
  tag: "tag",
};

export function toFilter(text: string): Result<WorkflowRunFilter, string> {
  if (!text.startsWith(FILTER_SIGIL)) return err("Filter text must start with @ symbol");

  const prefixStrings = Object.values(FILTER_PREFIXS);
  const prefixRegex = new RegExp(`^(${prefixStrings.map(filterPrefix).join("|")})`);
  const match = text.match(prefixRegex);
  if (!match)
    return err(`Filter must include a valid prefix (${prefixStrings.join(", ").slice(0, 4)}...)`);

  const prefix = match[1].slice(FILTER_SIGIL.length, -FILTER_DELIMITER.length);
  const rest = text.slice(match[0].length).trim();
  if (!rest) return err("Filter must include a value");

  switch (prefix) {
    case FILTER_PREFIXS.workflow:
      return ok({ entity: "workflow", id: rest });
    case FILTER_PREFIXS.tag:
      return ok({ entity: "tag", value: rest });
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
      text = `${filterPrefix(filter.entity)}${filter.value}`;
      break;
    default:
      assertNever(filter);
  }
  return { type: "token", text, value: filter };
}

export { getFilterValue } from "@/components/search";
