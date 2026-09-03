import { Breadcrumbs } from "../Breadcrumbs";
import { MainContentLayout } from "./MainContentLayout";

export function BreadcrumbHeaderLayout({ children }: { children: React.ReactNode }) {
  return <MainContentLayout header={<BreadcrumbHeader />}>{children}</MainContentLayout>;
}

function BreadcrumbHeader() {
  return (
    <div className="flex h-10 w-full shrink-0 items-center border-b border-app-border px-3.5 py-2">
      <Breadcrumbs />
    </div>
  );
}
