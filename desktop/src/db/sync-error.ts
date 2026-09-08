export type SyncErrorCode = "stream_failed" | "stream_ended";

export class SyncError extends Error {
  readonly name = "SyncError";

  constructor(
    message: string,
    readonly code: SyncErrorCode,
  ) {
    super(message);
  }
}
