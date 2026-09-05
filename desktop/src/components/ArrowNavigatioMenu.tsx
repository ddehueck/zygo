import { useHotkey } from "@tanstack/react-hotkeys";
import { useRouter, useRouterState, useCanGoBack } from "@tanstack/react-router";
import { Icon, iconDefinitions } from "./icons";
import { IconButton } from "./IconButton";

export function ArrowNavigatioMenu() {
  const router = useRouter();
  const size = 16;
  const canGoBack = useCanGoBack();

  // Wish there was a useCanGoForward hook
  const historyIndex = useRouterState({
    select: ({ location }) => location.state.__TSR_index,
  });
  const canGoForward = historyIndex < router.history.length - 1;

  // Back hotkey definition
  useHotkey("Mod+[", () => router.history.back());
  // Forward hotkey definition
  useHotkey("Mod+]", () => router.history.forward());

  return (
    <div className="flex gap-0">
      <IconButton
        size={size}
        type="button"
        onClick={() => router.history.back()}
        isDisabled={!canGoBack}
        aria-label="Go back"
      >
        <Icon definition={iconDefinitions.previous} optical="circle" size={size} aria-hidden />
      </IconButton>

      <IconButton
        size={size}
        type="button"
        onClick={() => router.history.forward()}
        isDisabled={!canGoForward}
        aria-label="Go forward"
      >
        <Icon definition={iconDefinitions.next} optical="circle" size={size} aria-hidden />
      </IconButton>
    </div>
  );
}
