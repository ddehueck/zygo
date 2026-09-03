import { createFileRoute, Outlet } from '@tanstack/react-router'
import { BreadcrumbHeaderLayout } from '../components/layout/BreadcrumbHeaderLayout'

export const Route = createFileRoute('/runs')({
  component: RouteComponent,
  beforeLoad: () => ({
    breadcrumb: {
      label: "Workflow Runs",
      link: "/",
    },
  }),
})

function RouteComponent() {
  return (
  <BreadcrumbHeaderLayout>
      <Outlet />
  </BreadcrumbHeaderLayout>
  )
}
