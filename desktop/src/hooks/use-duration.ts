import dayjs from "dayjs";
import { useEffect, useState } from "react";
import { formatDurationBetween } from "../lib/dates";

type UseDurationProps = {
  startedAt: string | null;
  completedAt: string | null;
};

export function useDuration({ startedAt, completedAt }: UseDurationProps): string | undefined {
  const [now, setNow] = useState(() => dayjs().valueOf());

  useEffect(() => {
    if (startedAt === null || completedAt !== null) return;

    const interval = setInterval(() => setNow(dayjs().valueOf()), 1000);
    return () => clearInterval(interval);
  }, [completedAt, startedAt]);

  return formatDurationBetween(startedAt, completedAt ?? now);
}
