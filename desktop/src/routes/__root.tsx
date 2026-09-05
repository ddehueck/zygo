import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import "../styles/theme.css";
import "../App.css";

import type { RouterContext } from "../router-context";
import { AppLayout } from "../components/layout/AppLayout";

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
});

function RootComponent() {
  return (
    <AppLayout>
      <Outlet />
    </AppLayout>
  );
}
