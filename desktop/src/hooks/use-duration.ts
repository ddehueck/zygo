import { useEffect, useState } from "react";
import { formatDuration } from "../lib/dates";

type UseDurationProps = {
  startedAt: number | null;
  completedAt: number | null;
};

export function useDuration({ startedAt, completedAt }: UseDurationProps): string | undefined {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (startedAt === null || completedAt !== null) {
      return;
    }

    const interval = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(interval);
  }, [completedAt, startedAt]);

  if (startedAt === null) {
    return undefined;
  }

  return formatDuration(Math.max(0, (completedAt ?? now) - startedAt));
}
