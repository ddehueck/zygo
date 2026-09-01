import { DbClient, DbProvider } from "@tanstack/react-db";
import { QueryClient } from "@tanstack/query-core";
import { Outlet, createRootRoute } from "@tanstack/react-router";
import "../App.css";
// import { SyncWorkflowUpdates } from "../sync/SyncWorkflowUpdates";

export const Route = createRootRoute({
  component: RootComponent,
});

const queryClient = new QueryClient();
const dbClient = new DbClient({ queryClient });

function RootComponent() {
  return (
    <DbProvider client={dbClient}>
      {/*<SyncWorkflowUpdates />*/}
      <Outlet />
    </DbProvider>
  );
}
