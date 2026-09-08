import { useEffect } from "react";

/** Matches true/plaintext-only/empty contenteditable, but not contenteditable="false". */
const EDITABLE_CONTENT_SELECTOR = "[contenteditable]:not([contenteditable='false'])";

function asElement(target: EventTarget | null): Element | null {
  return target instanceof Element ? target : null;
}

function getContentEditableHost(element: Element | null): HTMLElement | null {
  if (!(element instanceof HTMLElement)) return null;
  if (element.isContentEditable) return element;
  return element.closest(EDITABLE_CONTENT_SELECTOR);
}

function getNativeTextControl(
  element: Element | null,
): HTMLInputElement | HTMLTextAreaElement | null {
  if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
    return element;
  }
  return null;
}

function deleteInputType(event: KeyboardEvent): string | null {
  if (event.key === "Backspace") {
    if (event.metaKey || event.ctrlKey) return "deleteHardLineBackward";
    if (event.altKey) return "deleteWordBackward";
    return "deleteContentBackward";
  }
  if (event.key === "Delete") {
    if (event.metaKey || event.ctrlKey) return "deleteHardLineForward";
    if (event.altKey) return "deleteWordForward";
    return "deleteContentForward";
  }
  return null;
}

/**
 * WebViews (esp. macOS WKWebView) treat Delete/Backspace as history.back() when
 * the key isn't consumed as text editing. React Aria TokenField always cancels
 * the native beforeinput delete, so WebKit can fall through to navigation even
 * while the field is focused — prevent keydown and re-dispatch beforeinput.
 */
export function usePreventDeleteNavigation() {
  useEffect(() => {
    const preventDeleteNavigation = (event: KeyboardEvent) => {
      if (event.isComposing) return;

      const inputType = deleteInputType(event);
      if (inputType == null) return;

      const target = asElement(event.target);
      const active = asElement(document.activeElement);
      const nativeControl = getNativeTextControl(target) ?? getNativeTextControl(active);
      if (nativeControl && !nativeControl.disabled && !nativeControl.readOnly) {
        // Native controls handle deletion themselves; leaving default enabled
        // avoids breaking input/textarea editing.
        return;
      }

      const editingHost = getContentEditableHost(target) ?? getContentEditableHost(active);

      // Always cancel so WKWebView cannot treat this as history navigation.
      event.preventDefault();

      if (editingHost == null) return;

      // Preventing keydown suppresses the browser's beforeinput; TokenField
      // (and similar editors) rely on beforeinput for delete, so synthesize it.
      editingHost.dispatchEvent(
        new InputEvent("beforeinput", {
          bubbles: true,
          cancelable: true,
          inputType,
        }),
      );
    };

    window.addEventListener("keydown", preventDeleteNavigation, true);
    return () => window.removeEventListener("keydown", preventDeleteNavigation, true);
  }, []);
}
