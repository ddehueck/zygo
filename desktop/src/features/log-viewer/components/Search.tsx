import { Icon, iconDefinitions } from "@/components/icons";
import { IconButton } from "@/components/IconButton";
import { Input } from "@/components/Field";
import { useLogSearchContext } from "../search/LogSearchContext";

export function Search() {
  const { query, setQuery } = useLogSearchContext();
  const hasValue = query.length > 0;

  return (
    <div className="flex shrink-0 items-center gap-2 border-b border-app-border/60 bg-app-bg-elevated px-3">
      <Icon
        definition={iconDefinitions.search}
        className="shrink-0 text-app-foreground-muted"
        size={16}
      />
      <Input
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search logs"
        aria-label="Search logs"
        className="h-10 rounded-none border-0 bg-transparent px-0 ring-0 outline-none hover:border-0 hover:ring-0 hover:outline-none focus:border-0 focus:ring-0 focus:outline-none focus-visible:border-0 focus-visible:ring-0 focus-visible:outline-none"
      />
      {hasValue && (
        <IconButton size={12} onClick={() => setQuery("")} aria-label="Clear log search">
          <Icon definition={iconDefinitions.close} />
        </IconButton>
      )}
    </div>
  );
}
