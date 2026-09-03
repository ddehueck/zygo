import { type TokenFieldSegment, TokenFieldValue } from "react-aria-components/TokenField";

/** The filters supported by the workflow-run list. */
export type WorkflowSearchToken =
  | { type: "workflow"; workflowId: string }
  | { type: "tag"; name: string; value?: string };

/**
 * A token must be a complete filter and must be separated from neighboring
 * tokens by whitespace. This prevents a valid prefix from hiding an invalid
 * suffix, e.g. treating `@tag:patient` as valid in `@tag:patient:`.
 */
export const WORKFLOW_SEARCH_TOKEN_REGEX =
  /(?<!\S)@(?:workflow:[^\s:]+|tag:[^\s:]+(?::[^\s:]+)?)(?=\s)/g;

const WORKFLOW_TOKEN_REGEX = /^@workflow:([^\s:]+)$/;
const TAG_TOKEN_REGEX = /^@tag:([^\s:]+)(?::([^\s:]+))?$/;

export function parseWorkflowSearchToken(text: string): WorkflowSearchToken | null {
  const workflowMatch = WORKFLOW_TOKEN_REGEX.exec(text);
  if (workflowMatch) {
    return { type: "workflow", workflowId: workflowMatch[1] };
  }

  const tagMatch = TAG_TOKEN_REGEX.exec(text);
  if (tagMatch) {
    return {
      type: "tag",
      name: tagMatch[1],
      ...(tagMatch[2] === undefined ? {} : { value: tagMatch[2] }),
    };
  }

  return null;
}

/** Returns null when the query contains anything other than valid tokens. */
export function parseWorkflowSearchQuery(text: string): WorkflowSearchToken[] | null {
  const trimmedText = text.trim();
  if (trimmedText.length === 0) {
    return [];
  }

  const tokens = trimmedText.split(/\s+/).map(parseWorkflowSearchToken);
  return tokens.every((token): token is WorkflowSearchToken => token !== null) ? tokens : null;
}

export class TokenizingFieldValue extends TokenFieldValue {
  tokenRegex: RegExp;

  constructor(tokens: TokenFieldSegment[], tokenRegex: RegExp) {
    super(tokens);
    this.tokenRegex = tokenRegex;
  }

  createFieldValue(segments: TokenFieldSegment[]): this {
    let Constructor = this.constructor as new (
      tokens: TokenFieldSegment[],
      tokenRegex: RegExp,
    ) => this;
    return new Constructor(segments, this.tokenRegex);
  }

  tokenize(text: string): TokenFieldSegment[] {
    if (text.length === 0) {
      return [{ type: "text", text }];
    }

    let tokenRegex = this.tokenRegex;
    tokenRegex.lastIndex = 0;

    let match: RegExpExecArray | null = null;
    let start = 0;
    let segments: TokenFieldSegment[] = [];
    while ((match = tokenRegex.exec(text))) {
      if (match.index > start) {
        segments.push({ type: "text", text: text.slice(start, match.index) });
      }
      segments.push({ type: "token", text: match[0] });
      start = match.index + match[0].length;
    }

    if (start < text.length) {
      segments.push({ type: "text", text: text.slice(start) });
    }

    return segments;
  }
}

/** Converts a complete query into atomic token segments. */
export function tokenizeWorkflowSearchValue(text: string): TokenizingFieldValue | null {
  if (parseWorkflowSearchQuery(text) === null) {
    return null;
  }

  const tokenTexts = text.trim().length === 0 ? [] : text.trim().split(/\s+/);
  const segments = tokenTexts.flatMap((tokenText, index) => [
    ...(index > 0 ? [{ type: "text" as const, text: " " }] : []),
    { type: "token" as const, text: tokenText },
  ]);

  const value = new TokenizingFieldValue(segments, WORKFLOW_SEARCH_TOKEN_REGEX);
  const lastIndex = Math.max(value.segments.length - 1, 0);
  const endPosition = {
    index: lastIndex,
    offset: value.segments[lastIndex]?.text.length ?? 0,
  };
  return value.withSelectedRange(new TokenFieldValue.SelectedRange(endPosition));
}
