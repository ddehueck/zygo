import { TokenFieldSegment } from "react-aria-components/TokenField";

export type LogSearchSuggestion =
  | {
      id: string;
      type: "prefix";
      text: string;
    }
  | {
      id: string;
      type: "token";
      text: string;
      value: LogSearchFilter;
    };

/**
 * Job-run filter value is the database id used for query_logs.
 * The token segment `text` is the chip label (short public id).
 * `public_id` is kept for local log matching (Log.job_run_id is the public id).
 */
export type LogSearchFilter = {
  entity: "job_run";
  /** Database job_runs.id — the query value. */
  job_run_id: number;
  /** Full public id for local matching and chip label source. */
  public_id: string;
};

export type LogSearchTokenSegment = TokenFieldSegment<LogSearchFilter>;
