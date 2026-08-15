import { onBeforeUnmount, onMounted } from "vue";

/** Closes a modal surface with Escape, matching macOS dialog conventions. */
export function useEscapeClose(close: () => void) {
  const onKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  };
  onMounted(() => window.addEventListener("keydown", onKeydown));
  onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
}
