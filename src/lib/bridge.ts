import { invoke } from "@tauri-apps/api/core";
import type { CommitDetail, ConflictFile, DiffHunk, LogPage, OperationResult, RebaseStep, RepositorySnapshot } from "../types";

const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function command<T>(name: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri()) throw new Error("Graft repository commands are available in the macOS app.");
  return invoke<T>(name, args);
}

export const api = {
  open: (path: string) => command<RepositorySnapshot>("open_repository", { path }),
  refresh: (path: string) => command<RepositorySnapshot>("refresh_repository", { path }),
  log: (path: string, skip: number, limit: number) => command<LogPage>("log_page", { path, skip, limit }),
  detail: (path: string, oid: string) => command<CommitDetail>("commit_detail", { path, oid }),
  diff: (path: string, file: string, staged: boolean) => command<string>("working_diff", { path, file, staged }),
  stage: (path: string, files: string[], staged: boolean) => command<RepositorySnapshot>("set_staged", { path, files, staged }),
  commit: (path: string, message: string, amend: boolean, pushAfter = false) => command<OperationResult>("commit_changes", { path, message, amend, pushAfter }),
  remote: (path: string, operation: "fetch" | "pull" | "push") => command<OperationResult>("run_remote", { path, operation }),
  checkout: (path: string, branch: string, create: boolean) => command<OperationResult>("checkout_branch", { path, branch, create }),
  addWorktree: (path: string, worktreePath: string, branch: string, createBranch: boolean) => command<OperationResult>("add_worktree", { path, worktreePath, branch, createBranch }),
  historyOperation: (path: string, operation: string, target: string, mode?: string) => command<OperationResult>("run_history_operation", { path, operation, target, mode }),
  finishOperation: (path: string, operation: string, action: "continue" | "abort") => command<OperationResult>("finish_in_progress", { path, operation, action }),
  conflict: (path: string, file: string) => command<ConflictFile>("conflict_file", { path, file }),
  resolveConflict: (path: string, file: string, content: string) => command<OperationResult>("resolve_conflict", { path, file, content }),
  rebasePlan: (path: string, onto: string) => command<RebaseStep[]>("rebase_plan", { path, onto }),
  startRebase: (path: string, onto: string, steps: RebaseStep[]) => command<OperationResult>("start_interactive_rebase", { path, onto, steps }),
  hunks: (path: string, file: string, staged: boolean) => command<DiffHunk[]>("diff_hunks", { path, file, staged }),
  applyHunk: (path: string, file: string, staged: boolean, index: number) => command<RepositorySnapshot>("apply_hunk", { path, file, staged, index }),
  manageReference: (path: string, kind: "branch" | "tag" | "remote", action: "create" | "delete" | "rename" | "add" | "remove", name: string, value?: string, force = false) => command<OperationResult>("manage_reference", { path, kind, action, name, value, force }),
  watch: (path: string) => command<void>("watch_repository", { path }),
  openWindow: (path: string) => command<void>("open_repository_window", { path }),
  worktreeAction: (path: string, action: "prune" | "lock" | "unlock" | "remove", worktreePath?: string, force = false) => command<OperationResult>("worktree_action", { path, action, worktreePath, force }),
};
