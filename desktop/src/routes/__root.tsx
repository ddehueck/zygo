import { DbProvider as TanstackDbProvider } from "@tanstack/react-db";
import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import "../styles/theme.css";
import "../App.css";
import { tdb } from "../db/shared";
import type { RouterContext } from "../router-context";
import { AppLayout } from "../components/layout/AppLayout";

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
});

function RootComponent() {
  return (
    <TanstackDbProvider client={tdb}>
      <AppLayout>
        <Outlet />
      </AppLayout>
    </TanstackDbProvider>
  );
}
