import { createRootRoute, Outlet } from "@tanstack/react-router";
import { ThemeProvider } from "@/components/theme/theme-provider";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/layout/app-sidebar";
import "../index.css";

const RootLayout = () => (
  <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
    <SidebarProvider open={false} onOpenChange={() => {}}>
      <AppSidebar />
      <SidebarInset className="h-svh overflow-hidden">
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  </ThemeProvider>
);

export const Route = createRootRoute({ component: RootLayout });
