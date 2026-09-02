import { Moon, Sun } from "lucide-react";
import { IconButton } from "./IconButton";
import { useTheme } from "../hooks/use-theme";

export function ThemeToggle({ size = 16 }: { size?: number }) {
  const [theme, toggleTheme] = useTheme();
  const nextTheme = theme === "light" ? "dark" : "light";

  return (
    <IconButton
      size={size}
      type="button"
      onClick={toggleTheme}
      aria-label={`Switch to ${nextTheme} theme`}
    >
      <span className="t-icon-swap" data-state={theme === "light" ? "a" : "b"}>
        <span className="t-icon" data-icon="a">
          <Moon size={size} aria-hidden />
        </span>
        <span className="t-icon" data-icon="b">
          <Sun size={size} aria-hidden />
        </span>
      </span>
    </IconButton>
  );
}
