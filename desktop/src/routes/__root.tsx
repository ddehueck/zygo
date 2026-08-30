import type { ReactNode } from "react";
import { DbClient, DbProvider } from "@tanstack/react-db";
import { QueryClient } from "@tanstack/query-core";
import { HeadContent, Outlet, Scripts, createRootRoute } from "@tanstack/react-router";
import "../App.css";
import { SyncWorkflowUpdates } from "../sync/SyncWorkflowUpdates";

export const Route = createRootRoute({
  ssr: false,
  shellComponent: RootShell,
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      { title: "Zygo" },
    ],
  }),
  component: RootComponent,
});

const queryClient = new QueryClient();
const dbClient = new DbClient({ queryClient });

function RootShell({ children }: Readonly<{ children: ReactNode }>) {
  return <RootDocument>{children}</RootDocument>;
}

function RootComponent() {
  return (
    <DbProvider client={dbClient}>
      <SyncWorkflowUpdates />
      <Outlet />
    </DbProvider>
  );
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html>
      <head>
        <HeadContent />
      </head>
      <body>
        {children}
        <Scripts />
      </body>
    </html>
  );
}
