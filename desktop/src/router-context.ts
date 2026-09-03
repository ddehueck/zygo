export interface Breadcrumb {
  label: string;
  link: string;
}

export interface RouterContext {
  breadcrumb?: Breadcrumb;
}
