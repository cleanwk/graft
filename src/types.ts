export interface RepositoryState {
  merging: boolean;
  rebasing: boolean;
  cherryPicking: boolean;
  reverting: boolean;
  conflicts: number;
}

export interface Change {
  path: string;
  oldPath?: string;
  indexStatus: string;
  worktreeStatus: string;
  staged: boolean;
  conflicted: boolean;
}

export interface Branch {
  name: string;
  current: boolean;
  remote: boolean;
  oid: string;
  upstream?: string;
}

export interface Worktree {
  path: string;
  branch?: string;
  head: string;
  bare: boolean;
  locked: boolean;
  prunable: boolean;
}

export interface RepositorySnapshot {
  root: string;
  name: string;
  branch: string;
  head: string;
  upstream?: string;
  ahead: number;
  behind: number;
  state: RepositoryState;
  changes: Change[];
  changesTruncated: boolean;
  branches: Branch[];
  tags: string[];
  remotes: string[];
  worktrees: Worktree[];
}

export interface WorkspaceRepository {
  root: string;
  name: string;
  branch: string;
  latestTag?: string;
  defaultBranch?: string;
}

export interface WorkspaceSnapshot {
  root: string;
  name: string;
  kind: "repository" | "monorepo";
  repositories: WorkspaceRepository[];
}

export interface BatchWorktreeEntry {
  repository: string;
  repositoryPath: string;
  worktreePath: string;
  base?: string;
  success: boolean;
  message: string;
}

export interface BatchWorktreeResult {
  targetRoot: string;
  entries: BatchWorktreeEntry[];
  succeeded: number;
  failed: number;
}

export interface TerminalApp {
  id: "warp" | "iterm2" | "terminal";
  name: string;
}

export interface CommitRow {
  oid: string;
  shortOid: string;
  parents: string[];
  author: string;
  authorEmail: string;
  timestamp: number;
  relativeDate: string;
  subject: string;
  decorations: string[];
}

export interface ChangedFile { status: string; path: string; oldPath?: string }

export interface CommitDetail {
  oid: string;
  author: string;
  authorEmail: string;
  authoredAt: string;
  committer: string;
  committedAt: string;
  subject: string;
  body: string;
  parents: string[];
  files: ChangedFile[];
  patch: string;
}

export interface LogPage { skip: number; commits: CommitRow[]; hasMore: boolean }
export interface OperationResult { summary: string; output: string }
export interface ConflictFile { path: string; base: string; ours: string; theirs: string; working: string }
export interface RebaseStep { action: "pick" | "reword" | "edit" | "squash" | "fixup" | "drop"; oid: string; shortOid: string; subject: string }
export interface DiffHunk { index: number; header: string; patch: string; additions: number; deletions: number }
export interface CommandFailure { kind?: string; message?: string; recovery?: string }
