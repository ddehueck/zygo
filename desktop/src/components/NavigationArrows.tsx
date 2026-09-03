import { useRouter, useRouterState, useCanGoBack } from "@tanstack/react-router";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { IconButton } from "./IconButton";

export function NavigationArrows() {
  const router = useRouter();
  const size = 16;
  const canGoBack = useCanGoBack();

  // Wish there was a useCanGoForward hook
  const historyIndex = useRouterState({
    select: ({ location }) => location.state.__TSR_index,
  });
  const canGoForward = historyIndex < router.history.length - 1;

  return (
    <div className="flex gap-0">
      <IconButton
        size={size}
        type="button"
        onClick={() => router.history.back()}
        isDisabled={!canGoBack}
        aria-label="Go back"
      >
        <ChevronLeft size={size} aria-hidden />
      </IconButton>

      <IconButton
        size={size}
        type="button"
        onClick={() => router.history.forward()}
        isDisabled={!canGoForward}
        aria-label="Go forward"
      >
        <ChevronRight size={size} aria-hidden />
      </IconButton>
    </div>
  );
}
