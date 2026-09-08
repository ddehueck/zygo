import { last } from "@/lib/arrays";
import { type Position, TokenFieldValue } from "react-aria-components/TokenField";
import { toTokenSegment } from "./parse";
import type { LogSearchFilter, LogSearchSuggestion, LogSearchTokenSegment } from "./types";

export interface ActiveLogSearchFilter {
  anchor: Position;
  value: string;
  mayBecomeToken: boolean;
}

/**
 * Log search tokens:
 * - @job_run:<short-public-id> (label) → value.job_run_id (numeric query id)
 *
 * Job-run tokens are created from suggestions or row clicks (they need a known
 * database id). Free-typed @job_run:… text stays as text until accepted.
 *
 * Remaining text segments are free-text content search.
 */
export class LogSearchTokenValue extends TokenFieldValue<LogSearchFilter> {
  tokenize(text: string): LogSearchTokenSegment[] {
    // Job-run filters require a known database id from a suggestion / click.
    return [{ type: "text", text }];
  }

  getActiveInputText(): ActiveLogSearchFilter | null {
    const segment = last(this.segments);
    if (segment == null) {
      return {
        anchor: this.caretPosition,
        value: "",
        mayBecomeToken: true,
      };
    }

    if (segment.type === "token") {
      if (this.caretPosition.index < this.segments.length) return null;
      return {
        anchor: { index: this.segments.length, offset: 0 },
        value: "",
        mayBecomeToken: true,
      };
    }

    const value = segment.text.trim();
    return {
      anchor: { index: this.segments.length - 1, offset: 0 },
      value,
      mayBecomeToken: true,
    };
  }

  getFilterValues(): LogSearchFilter[] {
    return this.segments.filter((s) => s.type === "token").map((t) => t.value!);
  }

  getContentQuery(): string {
    return this.segments
      .filter((segment) => segment.type === "text")
      .map((segment) => segment.text)
      .join(" ")
      .trim();
  }

  acceptSuggestion({
    suggestion,
    anchor,
    end,
  }: {
    suggestion: LogSearchSuggestion;
    anchor: Position;
    end: Position;
  }): LogSearchTokenValue {
    const segments: LogSearchTokenSegment[] =
      suggestion.type === "token"
        ? [toTokenSegment(suggestion.value)]
        : [{ type: "text", text: suggestion.text }];

    const next = this.replaceRangeWithSegments(anchor, end, segments, false);
    // Job run filters are mutually exclusive — keep only the newly inserted token.
    return next.withSingleJobFilter();
  }

  /** Keep at most one job_run filter token (the last / newest). */
  withSingleJobFilter(): LogSearchTokenValue {
    let lastJobIndex = -1;
    for (let i = 0; i < this.segments.length; i++) {
      const segment = this.segments[i];
      if (segment.type === "token" && segment.value?.entity === "job_run") {
        lastJobIndex = i;
      }
    }
    if (lastJobIndex < 0) return this;

    const nextSegments = this.segments.filter((segment, index) => {
      if (segment.type === "token" && segment.value?.entity === "job_run") {
        return index === lastJobIndex;
      }
      return true;
    });

    if (nextSegments.length === this.segments.length) return this;
    return new LogSearchTokenValue(nextSegments);
  }

  withJobRunFilter(filter: LogSearchFilter): LogSearchTokenValue {
    const withoutJob = this.segments.filter(
      (segment) => !(segment.type === "token" && segment.value?.entity === "job_run"),
    );
    const nextSegments: LogSearchTokenSegment[] = [...withoutJob, toTokenSegment(filter)];
    return new LogSearchTokenValue(nextSegments);
  }
}
