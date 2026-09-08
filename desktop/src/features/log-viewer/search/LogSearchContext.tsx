import { createContext, useContext, useMemo, useState, type ReactNode } from "react";

type LogSearchContextValue = {
  query: string;
  setQuery: (query: string) => void;
  cleanedQuery: string;
};

const LogSearchContext = createContext<LogSearchContextValue | null>(null);

export function LogSearchProvider({ children }: { children: ReactNode }) {
  const [query, setQuery] = useState("");

  const contextValue = useMemo(
    () => ({
      query,
      setQuery,
      cleanedQuery: query.trim(),
    }),
    [query],
  );

  return <LogSearchContext.Provider value={contextValue}>{children}</LogSearchContext.Provider>;
}

export function useLogSearchContext() {
  const context = useContext(LogSearchContext);
  if (context == null) {
    throw new Error("useLogSearch must be used within LogSearchProvider");
  }
  return context;
}
