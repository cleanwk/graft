import type { CommandFailure } from "../types";

/** Renders any command rejection as one readable sentence, keeping the backend's recovery hint when present. */
export function errorMessage(caught: unknown, fallback = "The operation failed."): string {
  if (typeof caught === "string") return caught;
  const value = caught as (CommandFailure & Error) | undefined;
  return [value?.message ?? fallback, value?.recovery].filter(Boolean).join(" ");
}
