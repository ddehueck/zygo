import dayjs from "dayjs";
import { useEffect, useState } from "react";
import { formatDuration } from "../lib/dates";

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

  if (startedAt === null) return undefined;

  const startedAtDate = dayjs(startedAt);
  const completedAtDate = completedAt === null ? dayjs(now) : dayjs(completedAt);

  return formatDuration(Math.max(0, completedAtDate.diff(startedAtDate)));
}
