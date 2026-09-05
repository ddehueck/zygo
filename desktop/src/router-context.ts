export interface Breadcrumb {
  label: string;
  link: string;
}

// Stored on a destination history entry so breadcrumbs can identify its real predecessor.
export interface BreadcrumbBackTarget {
  href: string;
  pathname: string;
  historyIndex: number;
}

export interface BreadcrumbHistoryState {
  breadcrumbBack?: BreadcrumbBackTarget;
}

export interface RouterContext {
  breadcrumb?: Breadcrumb;
}
