import { DbProvider } from "@tanstack/react-db";
import { Outlet, createRootRoute } from "@tanstack/react-router";
import "../App.css";
import { tdb } from "../db/shared";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  return (
    <DbProvider client={tdb}>
      <Outlet />
    </DbProvider>
  );
}
