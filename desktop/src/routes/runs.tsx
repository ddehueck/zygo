import { createFileRoute, Outlet } from '@tanstack/react-router'
import { MainContentLayout } from '../components/layout/MainContentLayout'

export const Route = createFileRoute('/runs')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
  <MainContentLayout
    titleContent={<p></p>}
    >
      <Outlet />
      </MainContentLayout>
  )
}
