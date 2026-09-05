import { createLink, useRouter, useRouterState } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import {
  Breadcrumb as AriaBreadcrumb,
  Breadcrumbs as AriaBreadcrumbs,
} from "react-aria-components/Breadcrumbs";
import { Link as AriaLink } from "react-aria-components/Link";
import type { Breadcrumb, BreadcrumbHistoryState } from "../router-context";

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
  const router = useRouter();
  const historyIndex = useRouterState({
    select: ({ location }) => location.state.__TSR_index,
  });
  const backTarget = useRouterState({
    select: ({ location }) => (location.state as BreadcrumbHistoryState).breadcrumbBack,
  });

  // We want to preserve the back behavior when a breadcrumb aligns with the back history.
  // Matching both the path and adjacent history index proves this crumb represents
  // the actual previous entry, rather than merely linking to an ancestor route.
  const isBackPathMatch = backTarget?.pathname === breadcrumb.link;
  const isHistoryIndexMatch = backTarget?.historyIndex === historyIndex - 1;
  const isBackTarget = isBackPathMatch && isHistoryIndexMatch;

  return (
    <AriaBreadcrumb className="flex min-w-0 items-center gap-1">
      {({ isCurrent }) => (
        <>
          <RouterBreadcrumbLink
            to={isBackTarget ? backTarget.href : breadcrumb.link}
            onClick={(event) => {
              // Match TanStack Router's link guard: only intercept an unmodified
              // primary click, leaving every other click to normal link behavior.
              if (
                !isBackTarget ||
                event.button !== 0 ||
                event.metaKey ||
                event.altKey ||
                event.ctrlKey ||
                event.shiftKey
              )
                return;

              // Modified clicks keep the real href; normal activation restores the
              // existing list entry, including its search params and scroll state.
              event.preventDefault();
              router.history.back();
            }}
            className={`truncate rounded-sm outline-none hover:text-app-foreground focus-visible:outline-1 focus-visible:outline-offset-2 focus-visible:outline-app-accent ${
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
