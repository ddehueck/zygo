import { last } from "@/lib/arrays";
import {
  type Position,
  type TokenFieldSegment,
  TokenFieldValue,
} from "react-aria-components/TokenField";
import { WorkflowRunFilter, WorkflowRunSearchSuggestion, WorkflowSearchValue } from "./types";
import { toFilter, toTokenSegment } from "./parse";
import { isErr } from "@/lib/result";
import { lastChar } from "@/lib/string";

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
  getInputValue(): string {
    let segment = this.segments[this.caretPosition.index];
    return segment?.type === "text" ? segment.text : "";
  }

  getFilters(): WorkflowRunFilter[] {
    const filters = [];
    for (let segment of this.segments) {
      if (segment.type == "text") continue;

      const result = toFilter(segment.text);
      if (isErr(result)) {
        console.warn("Could not parse token to filter");
        continue;
      }

      filters.push(result.data);
    }
    return filters;
  }

  addFilter(filter: WorkflowRunFilter): WorkflowSearchTokenValue {
    let allFilters = [...this.getFilters(), filter];
    return new WorkflowSearchTokenValue(allFilters.map(toTokenSegment));
  }

  tokenize(text: string): WorkflowSearchTokenSegment[] {
    const result = toFilter(text);

    if (isErr(result)) return [{ type: "text", text }];

    if (isSeparatorCharacter(lastChar(text))) return [toTokenSegment(result.data)];

    return [{ type: "text", text }];
  }
  // }
  //     segments.push({ type: "text", text: part });
  //     continue;
  //   }
  //   let tokenValue = index < text.length ? getTokenValue(part) : null;
  //   if (tokenValue == null) {
  //     segments.push({ type: "text", text: text.slice(partStart) });
  //     break;
  //   }
  //   segments.push({ type: "token", text: part, value: tokenValue });
  // }
  // return segments;
  // }

  /**
   * Returns the final text segment, which is the only segment that can still be
   * pending or invalid. All preceding non-separator segments are tokens.
   */
  getActiveFilter(): ActiveWorkflowSearchFilter | null {
    let segment = last(this.segments);
    if (segment == null) return null;

    // React Aria represents the caret immediately after a token without a
    // trailing text segment. Treat that position as an empty filter so a
    // second suggestion can be inserted at the caret.
    if (segment.type === "token") {
      if (this.caretPosition.index < this.segments.length) return null;
      return {
        anchor: { index: this.segments.length, offset: 0 },
        value: "",
        mayBecomeToken: true,
      };
    }

    let value = segment.text.trim();
    return {
      anchor: { index: this.segments.length - 1, offset: 0 },
      value,
      mayBecomeToken: true, // TODO: Move validation somewhere else
    };
  }

  acceptSuggestion({
    suggestion,
    anchor,
    end,
  }: {
    suggestion: WorkflowRunSearchSuggestion;
    anchor: Position;
    end: Position;
  }): WorkflowSearchTokenValue {
    let segments: WorkflowSearchTokenSegment[] =
      suggestion.type === "token" ? [suggestion] : [{ type: "text", text: suggestion.text }];

    return this.replaceRangeWithSegments(anchor, end, segments, false);
  }
}

function isSeparatorCharacter(character: string | undefined): boolean {
  return character === "," || character === "\u200B" || character?.trim().length === 0;
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
