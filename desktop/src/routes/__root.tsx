import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import "../styles/theme.css";
import "../App.css";

import type { RouterContext } from "../router-context";
import { AppLayout } from "../components/layout/AppLayout";
import { ReactError } from "../components/ReactError";
import { usePreventDeleteNavigation } from "../hooks/use-prevent-delete-navigation";

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  errorComponent: ReactError,
});

function RootComponent() {
  usePreventDeleteNavigation();

  return (
    <AppLayout>
      <Outlet />
    </AppLayout>
  );
}
