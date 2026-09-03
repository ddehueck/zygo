import { Breadcrumbs } from "../Breadcrumbs";
import { MainContentLayout } from "./MainContentLayout";

export function BreadcrumbHeaderLayout({ children }: { children: React.ReactNode }) {
  return (
    <MainContentLayout header={<BreadcrumbHeader />}>
      {children}
    </MainContentLayout>
  )
}

function BreadcrumbHeader() {
  return (
    <div className="shrink-0 w-full h-10 flex items-center px-2 py-2 border-b border-app-border">
      <Breadcrumbs />
    </div>
  )
}
