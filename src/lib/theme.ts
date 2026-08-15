import { ref, watch } from "vue";

export const themes = [
  { id: "light", label: "Light" },
  { id: "dark", label: "Dark" },
  { id: "droid", label: "Droid" },
] as const;

export type ThemeId = (typeof themes)[number]["id"];

const STORAGE_KEY = "graft.theme";
const isTheme = (value: string | null): value is ThemeId => themes.some((theme) => theme.id === value);

export function storedTheme(): ThemeId {
  const value = localStorage.getItem(STORAGE_KEY);
  return isTheme(value) ? value : "light";
}

export function applyStoredTheme() {
  document.documentElement.dataset.theme = storedTheme();
}

export function useTheme() {
  const theme = ref<ThemeId>(storedTheme());
  watch(theme, (value) => {
    document.documentElement.dataset.theme = value;
    localStorage.setItem(STORAGE_KEY, value);
  }, { immediate: true });
  return theme;
}
