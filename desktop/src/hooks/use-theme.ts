import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Theme = "light" | "dark";

const THEME_TRANSITION_CLASS = "is-theme-transitioning";

export function useTheme() {
  const transitionTimeoutRef = useRef<number | null>(null);
  const [theme, setTheme] = useState<Theme>(() => {
    if (typeof window === "undefined") {
      return "light";
    }

    const storedTheme = window.localStorage.getItem("zygo-theme");
    if (storedTheme === "light" || storedTheme === "dark") {
      return storedTheme;
    }

    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  });

  useLayoutEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("zygo-theme", theme);

    if (!("__TAURI_INTERNALS__" in window)) {
      return;
    }

    const backgroundColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--app-bg-base")
      .trim();
    const nativeWindow = getCurrentWindow();

    void Promise.all([
      nativeWindow.setBackgroundColor(backgroundColor),
      nativeWindow.setTheme(theme),
    ]).catch((error) => {
      console.error("Failed to update the native window appearance", error);
    });
  }, [theme]);

  useEffect(() => {
    return () => {
      if (transitionTimeoutRef.current !== null) {
        window.clearTimeout(transitionTimeoutRef.current);
      }
      document.documentElement.classList.remove(THEME_TRANSITION_CLASS);
    };
  }, []);

  const toggleTheme = () => {
    const root = document.documentElement;

    if (!window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      if (transitionTimeoutRef.current !== null) {
        window.clearTimeout(transitionTimeoutRef.current);
      }

      root.classList.remove(THEME_TRANSITION_CLASS);
      void root.offsetWidth;
      root.classList.add(THEME_TRANSITION_CLASS);

      const duration = Number.parseFloat(
        getComputedStyle(root).getPropertyValue("--theme-transition-dur"),
      );
      transitionTimeoutRef.current = window.setTimeout(
        () => {
          root.classList.remove(THEME_TRANSITION_CLASS);
          transitionTimeoutRef.current = null;
        },
        Number.isFinite(duration) ? duration : 250,
      );
    }

    setTheme((current) => (current === "light" ? "dark" : "light"));
  };

  return [theme, toggleTheme] as const;
}
