import { last } from "@/lib/arrays";
import { type Position, TokenFieldValue } from "react-aria-components/TokenField";
import {
  WorkflowRunFilter,
  WorkflowRunSearchSuggestion,
  WorkflowSearchTokenSegment,
} from "./types";
import { toFilter, toTokenSegment } from "./parse";
import { isErr } from "@/lib/result";
import { lastChar } from "@/lib/string";

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
export class WorkflowSearchTokenValue extends TokenFieldValue<WorkflowRunFilter> {
  tokenize(text: string): WorkflowSearchTokenSegment[] {
    const result = toFilter(text);

    if (isErr(result)) return [{ type: "text", text }];

    if (isSeparatorCharacter(lastChar(text))) return [toTokenSegment(result.data)];

    return [{ type: "text", text }];
  }

  /**
   * Returns the currently active text block and it's position
   */
  getActiveInputText(): ActiveWorkflowSearchFilter | null {
    let segment = last(this.segments);
    if (segment == null)
      return {
        anchor: this.caretPosition,
        value: "",
        mayBecomeToken: true,
      };

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

  getFilterValues(): WorkflowRunFilter[] {
    return this.segments.filter((s) => s.type === "token").map((t) => t.value!);
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
      suggestion.type === "token"
        ? [toTokenSegment(suggestion.value)]
        : [{ type: "text", text: suggestion.text }];

    return this.replaceRangeWithSegments(anchor, end, segments, false);
  }
}

function isSeparatorCharacter(character: string | undefined): boolean {
  return character === "," || character === "\u200B" || character?.trim().length === 0;
}
