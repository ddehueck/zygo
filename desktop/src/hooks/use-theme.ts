import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Theme = "light" | "dark";

export function useTheme() {
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

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("zygo-theme", theme);

    if (!("__TAURI_INTERNALS__" in window)) {
      return;
    }

    const backgroundColor = getComputedStyle(document.documentElement)
      .getPropertyValue("--app-background")
      .trim();
    const nativeWindow = getCurrentWindow();

    void Promise.all([
      nativeWindow.setBackgroundColor(backgroundColor),
      nativeWindow.setTheme(theme),
    ]).catch((error) => {
      console.error("Failed to update the native window appearance", error);
    });
  }, [theme]);

  return [theme, () => setTheme((current) => (current === "light" ? "dark" : "light"))] as const;
}
