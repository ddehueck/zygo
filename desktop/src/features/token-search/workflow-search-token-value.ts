import { last } from "@/lib/arrays";
import {
  type Position,
  type TokenFieldSegment,
  TokenFieldValue,
} from "react-aria-components/TokenField";

type WorkflowSearchValue = "workflow" | "tag";
type WorkflowSearchTokenSegment = TokenFieldSegment<WorkflowSearchValue>;

export interface ActiveWorkflowSearchFilter {
  anchor: Position;
  value: string;
  mayBecomeToken: boolean;
}

/**
 * A workflow can be searched with two token prefixes:
 * - @workflow:
 * - @tag:
 *
 * First principles of this token field:
 * - Using a contenteditable div to allow for rich text editing
 * - Using contenteditable=false for atomic token UI elements
 *
 * ```html
 * <div contenteditable="true">
 *     Hello <span contenteditable="false">world</span>!
 * </div>
 *
 * <div contenteditable="true">
   <span data-react-aria-token>
     \u200B                <!--with 0-width spaces so cursor can be rendered by the browser-->
     <span contenteditable="false">Architecture</span>
     \u200B
   </span>
   <span data-react-aria-token>
     \u200B
     <span contenteditable="false">Design</span>
     \u200B
   </span>
 </div>

 * ```
 */
export class WorkflowSearchTokenValue extends TokenFieldValue<WorkflowSearchValue> {
  tokenize(text: string): WorkflowSearchTokenSegment[] {
    // Keep separators in the model so segment positions continue to map one-to-one
    // to the contenteditable DOM. A complete filter becomes a token only after a
    // separator is entered, and ordinary text prevents later filters from becoming
    // tokens.
    if (text.length === 0) {
      return [{ type: "text", text }];
    }

    let segments: WorkflowSearchTokenSegment[] = [];
    let index = 0;
    while (index < text.length) {
      let partStart = index;
      let separator = isSeparatorCharacter(text[index]);
      while (index < text.length && isSeparatorCharacter(text[index]) === separator) {
        index++;
      }

      let part = text.slice(partStart, index);
      if (separator) {
        segments.push({ type: "text", text: part });
        continue;
      }

      let tokenValue = index < text.length ? getTokenValue(part) : null;
      if (tokenValue == null) {
        segments.push({ type: "text", text: text.slice(partStart) });
        break;
      }

      segments.push({ type: "token", text: part, value: tokenValue });
    }

    return segments;
  }

  /**
   * Returns the final text segment, which is the only segment that can still be
   * pending or invalid. All preceding non-separator segments are tokens.
   */
  getActiveFilter(): ActiveWorkflowSearchFilter | null {
    let segment = last(this.segments);
    if (segment == null || segment.type !== "text") return null;

    let start = 0;
    let end = segment.text.length;
    while (start < end && isSeparatorCharacter(segment.text[start])) start++;
    while (end > start && isSeparatorCharacter(segment.text[end - 1])) end--;

    let value = segment.text.slice(start, end);
    return {
      anchor: { index: this.segments.length - 1, offset: start },
      value,
      mayBecomeToken: mayBecomeToken(value),
    };
  }
}

function isSeparatorCharacter(character: string | undefined): boolean {
  return character === "," || character === "\u200B" || character?.trim().length === 0;
}

function mayBecomeToken(part: string): boolean {
  if (getTokenValue(part) !== null) return true;
  let prefixes = ["@workflow:", "@tag:"];
  for (let prefix of prefixes) if (prefix.startsWith(part)) return true;
  return false;
}

// If a part is prefixed with "@workflow" or "@tag" and contains a colon
// with a non-empty value after it, return the corresponding token value.
function getTokenValue(part: string): WorkflowSearchValue | null {
  let prefix: [string, WorkflowSearchValue] | undefined;
  if (part.startsWith("@workflow:")) {
    prefix = ["@workflow:", "workflow"];
  } else if (part.startsWith("@tag:")) {
    prefix = ["@tag:", "tag"];
  }

  if (prefix == null) {
    return null;
  }

  let value = part.slice(prefix[0].length);
  return value.length > 0 && !value.includes(":") ? prefix[1] : null;
}
