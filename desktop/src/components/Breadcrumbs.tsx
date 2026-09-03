import { createLink, useRouterState } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import {
  Breadcrumb as AriaBreadcrumb,
  Breadcrumbs as AriaBreadcrumbs,
} from "react-aria-components/Breadcrumbs";
import { Link as AriaLink } from "react-aria-components/Link";
import type { Breadcrumb } from "../router-context";

const RouterBreadcrumbLink = createLink(AriaLink);

export function Breadcrumbs() {
  const breadcrumbs = useRouterState({
    select: (state) =>
      state.matches.flatMap(({ context }) => (context.breadcrumb ? [context.breadcrumb] : [])),
  });

  if (breadcrumbs.length === 0) {
    return null;
  }

  return (
    <AriaBreadcrumbs
      aria-label="Breadcrumb"
      className="flex min-w-0 items-center gap-1 text-xs text-app-foreground-muted"
    >
      {breadcrumbs.map((breadcrumb, index) => (
        <BreadcrumbItem key={`${breadcrumb.link}-${index}`} breadcrumb={breadcrumb} />
      ))}
    </AriaBreadcrumbs>
  );
}

function BreadcrumbItem({ breadcrumb }: { breadcrumb: Breadcrumb }) {
  return (
    <AriaBreadcrumb className="flex min-w-0 items-center gap-1">
      {({ isCurrent }) => (
        <>
          <RouterBreadcrumbLink
            to={breadcrumb.link}
            className={`truncate rounded-sm outline-none hover:text-app-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-app-accent ${
              isCurrent ? "font-medium text-app-foreground" : ""
            }`}
          >
            {breadcrumb.label}
          </RouterBreadcrumbLink>
          {!isCurrent && <ChevronRight aria-hidden className="size-3 shrink-0" />}
        </>
      )}
    </AriaBreadcrumb>
  );
}
