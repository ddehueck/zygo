import { useEffect } from "react";

function isTextEditingElement(element: Element | null) {
  return (
    element instanceof HTMLElement &&
    (element.isContentEditable ||
      ["INPUT", "TEXTAREA", "SELECT"].includes(element.tagName) ||
      element.closest('[contenteditable="true"]') !== null)
  );
}

function isTextEditingTarget(target: EventTarget | null) {
  const targetElement = target instanceof Element ? target : null;
  return isTextEditingElement(targetElement) || isTextEditingElement(document.activeElement);
}

export function usePreventDeleteNavigation() {
  useEffect(() => {
    const preventDeleteNavigation = (event: KeyboardEvent) => {
      // WebViews can treat Backspace as browser history navigation. On macOS,
      // the physical Delete key is reported as Backspace by the DOM.
      if (
        (event.key === "Backspace" || event.key === "Delete") &&
        !isTextEditingTarget(event.target)
      ) {
        event.preventDefault();
      }
    };

    window.addEventListener("keydown", preventDeleteNavigation, true);
    return () => window.removeEventListener("keydown", preventDeleteNavigation, true);
  }, []);
}
