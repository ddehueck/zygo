import dayjs from "dayjs";

const DATABASE_TIMESTAMP_PATTERN = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(?:\.\d+)?$/;

export function parseTimestamp(timestamp: string) {
  const normalizedTimestamp = DATABASE_TIMESTAMP_PATTERN.test(timestamp)
    ? `${timestamp.replace(" ", "T")}Z`
    : timestamp;

  return dayjs(normalizedTimestamp);
}

export function formatDate(timestamp: string | null): string {
  if (timestamp === null) return "—";

  const date = parseTimestamp(timestamp);
  return date.isValid() ? date.format("MMM D, YYYY h:mm A") : timestamp;
}

export function formatDurationBetween(
  startedAt: string | null,
  endedAt: string | number,
): string | undefined {
  if (startedAt === null) return undefined;

  const startedAtDate = parseTimestamp(startedAt);
  const endedAtDate = typeof endedAt === "number" ? dayjs(endedAt) : parseTimestamp(endedAt);
  if (!startedAtDate.isValid() || !endedAtDate.isValid()) return undefined;

  return formatDuration(Math.max(0, endedAtDate.diff(startedAtDate)));
}

export function formatDuration(durationMs: number): string {
  const totalSeconds = Math.floor(durationMs / 1000);
  if (totalSeconds < 60) {
    return `${totalSeconds}s`;
  }

  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (minutes < 60) {
    return `${minutes}m ${seconds}s`;
  }

  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}
