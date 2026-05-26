/** Available theme accent colors — oklch values for shadcn CSS vars */
export const themeColors = [
  {
    name: "Default",
    light: { primary: "oklch(0.205 0 0)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.985 0 0)", primaryForeground: "oklch(0.205 0 0)" },
    preview: "oklch(0.205 0 0)",
  },
  {
    name: "Blue",
    light: { primary: "oklch(0.488 0.18 260)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.623 0.18 260)", primaryForeground: "oklch(0.985 0 0)" },
    preview: "oklch(0.55 0.18 260)",
  },
  {
    name: "Green",
    light: { primary: "oklch(0.52 0.15 155)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.65 0.15 155)", primaryForeground: "oklch(0.985 0 0)" },
    preview: "oklch(0.58 0.15 155)",
  },
  {
    name: "Purple",
    light: { primary: "oklch(0.48 0.18 295)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.65 0.18 295)", primaryForeground: "oklch(0.985 0 0)" },
    preview: "oklch(0.55 0.18 295)",
  },
  {
    name: "Orange",
    light: { primary: "oklch(0.635 0.2 55)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.7 0.18 55)", primaryForeground: "oklch(0.145 0 0)" },
    preview: "oklch(0.67 0.2 55)",
  },
  {
    name: "Red",
    light: { primary: "oklch(0.535 0.2 25)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.65 0.2 25)", primaryForeground: "oklch(0.985 0 0)" },
    preview: "oklch(0.59 0.2 25)",
  },
  {
    name: "Teal",
    light: { primary: "oklch(0.48 0.12 180)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.65 0.12 180)", primaryForeground: "oklch(0.985 0 0)" },
    preview: "oklch(0.55 0.12 180)",
  },
  {
    name: "Pink",
    light: { primary: "oklch(0.595 0.18 340)", primaryForeground: "oklch(0.985 0 0)" },
    dark: { primary: "oklch(0.7 0.18 340)", primaryForeground: "oklch(0.985 0 0)" },
    preview: "oklch(0.65 0.18 340)",
  },
]

/** Apply theme colors for current light/dark mode */
export function applyTheme(color: (typeof themeColors)[number]) {
  const isDark = document.documentElement.classList.contains("dark")
  const variant = isDark ? color.dark : color.light
  document.documentElement.style.setProperty("--primary", variant.primary)
  document.documentElement.style.setProperty("--primary-foreground", variant.primaryForeground)
  document.documentElement.style.setProperty("--ring", variant.primary)
  localStorage.setItem("theme-accent", color.name)
}

/** Re-apply saved theme accent (call after toggling dark/light mode) */
export function reapplyAccent() {
  const saved = localStorage.getItem("theme-accent")
  if (saved) {
    const color = themeColors.find((c) => c.name === saved)
    if (color) applyTheme(color)
  }
}
