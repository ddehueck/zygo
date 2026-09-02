import { DbProvider } from "@tanstack/react-db";
import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";
import "../styles/theme.css";
import "../App.css";
import { Titlebar } from "../components/Titlebar";
import { tdb } from "../db/shared";
import type { RouterContext } from "../router-context";

export const Route = createRootRouteWithContext<RouterContext>()({
  beforeLoad: () => ({
    breadcrumb: {
      label: "Home",
      link: "/",
    },
  }),
  component: RootComponent,
});

function RootComponent() {
  return (
    <DbProvider client={tdb}>
      <div className="min-h-screen">
        <Titlebar />
        <Outlet />
      </div>
    </DbProvider>
  );
}
