interface JobCountsBadgeProps {
  activeJobCount: number;
  succeededJobCount: number;
  erroredJobCount: number;
}

interface JobCountItem {
  label: string;
  value: number;
  dotClassName: string;
  valueClassName?: string;
}

export function JobCountsBadge({
  activeJobCount,
  succeededJobCount,
  erroredJobCount,
}: JobCountsBadgeProps) {
  const items: JobCountItem[] = [
    { label: "Active jobs", value: activeJobCount, dotClassName: "bg-app-accent" },
    { label: "Succeeded jobs", value: succeededJobCount, dotClassName: "bg-app-success" },
    {
      label: "Errored jobs",
      value: erroredJobCount,
      dotClassName: "bg-app-danger",
      valueClassName: "text-app-danger",
    },
  ];

  return (
    <div
      role="group"
      aria-label="Job counts"
      className="inline-flex max-w-full items-center gap-3 rounded-full bg-transparent px-4 py-2 font-sans"
    >
      {items.map(({ label, value, dotClassName, valueClassName }) => (
        <span
          key={label}
          aria-label={`${label}: ${formatJobCount(value)}`}
          className="inline-flex items-center gap-2 font-semibold leading-none tracking-tight text-app-foreground"
        >
          <span aria-hidden="true" className={`size-2 shrink-0 rounded-full ${dotClassName}`} />
          <span className={valueClassName}>{formatJobCount(value)}</span>
        </span>
      ))}
    </div>
  );
}

function formatJobCount(value: number): string {
  return new Intl.NumberFormat("en-US", {
    notation: "compact",
    maximumFractionDigits: 1,
  })
    .format(value)
    .toLowerCase();
}
