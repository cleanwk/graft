use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    io::BufRead,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use thiserror::Error;

const RECORD: char = '\u{1e}';
const FIELD: char = '\u{1f}';

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git is not available: {0}")]
    MissingGit(String),
    #[error("{0} is not a Git repository")]
    NotRepository(String),
    #[error("Git rejected the operation: {0}")]
    Command(String),
    #[error("The repository returned data Graft could not understand: {0}")]
    InvalidOutput(String),
    #[error("That operation is not supported: {0}")]
    Unsupported(String),
}

impl GitError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingGit(_) => "missingGit",
            Self::NotRepository(_) => "notRepository",
            Self::Command(_) => "git",
            Self::InvalidOutput(_) => "invalidOutput",
            Self::Unsupported(_) => "unsupported",
        }
    }

    pub fn recovery(&self) -> Option<&'static str> {
        match self {
            Self::MissingGit(_) => Some("Install the Xcode Command Line Tools, then reopen Graft."),
            Self::NotRepository(_) => Some("Choose a folder that contains a Git working tree."),
            Self::Command(_) => Some("Review the repository state and try the operation again."),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitService {
    root: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySnapshot {
    pub root: String,
    pub name: String,
    pub branch: String,
    pub head: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub state: RepositoryState,
    pub changes: Vec<Change>,
    pub changes_truncated: bool,
    pub branches: Vec<Branch>,
    pub tags: Vec<String>,
    pub remotes: Vec<String>,
    pub worktrees: Vec<Worktree>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRepository {
    pub root: String,
    pub name: String,
    pub branch: String,
    pub latest_tag: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub root: String,
    pub name: String,
    pub kind: String,
    pub repositories: Vec<WorkspaceRepository>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchWorktreeEntry {
    pub repository: String,
    pub repository_path: String,
    pub worktree_path: String,
    pub base: Option<String>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchWorktreeResult {
    pub target_root: String,
    pub entries: Vec<BatchWorktreeEntry>,
    pub succeeded: usize,
    pub failed: usize,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryState {
    pub merging: bool,
    pub rebasing: bool,
    pub cherry_picking: bool,
    pub reverting: bool,
    pub conflicts: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub path: String,
    pub old_path: Option<String>,
    pub index_status: String,
    pub worktree_status: String,
    pub staged: bool,
    pub conflicted: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
    pub current: bool,
    pub remote: bool,
    pub oid: String,
    pub upstream: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Worktree {
    pub path: String,
    pub branch: Option<String>,
    pub head: String,
    pub bare: bool,
    pub locked: bool,
    pub prunable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRow {
    pub oid: String,
    pub short_oid: String,
    pub parents: Vec<String>,
    pub author: String,
    pub author_email: String,
    pub timestamp: i64,
    pub relative_date: String,
    pub subject: String,
    pub decorations: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPage {
    pub skip: usize,
    pub commits: Vec<CommitRow>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub oid: String,
    pub author: String,
    pub author_email: String,
    pub authored_at: String,
    pub committer: String,
    pub committed_at: String,
    pub subject: String,
    pub body: String,
    pub parents: Vec<String>,
    pub files: Vec<ChangedFile>,
    pub patch: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub status: String,
    pub path: String,
    pub old_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub summary: String,
    pub output: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFile {
    pub path: String,
    pub base: String,
    pub ours: String,
    pub theirs: String,
    pub working: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebaseStep {
    pub action: String,
    pub oid: String,
    pub short_oid: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub index: usize,
    pub header: String,
    pub patch: String,
    pub additions: usize,
    pub deletions: usize,
}

impl GitService {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, GitError> {
        let candidate = path.into();
        let output = Self::git_at(&candidate, ["rev-parse", "--show-toplevel"])?;
        if !output.status.success() {
            return Err(GitError::NotRepository(candidate.display().to_string()));
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Ok(Self { root: PathBuf::from(root) })
    }

    pub fn discover_workspace(path: impl Into<PathBuf>) -> Result<WorkspaceSnapshot, GitError> {
        let candidate = path.into();
        let selected = std::fs::canonicalize(&candidate)
            .map_err(|error| GitError::NotRepository(format!("{} ({error})", candidate.display())))?;
        if !selected.is_dir() {
            return Err(GitError::NotRepository(selected.display().to_string()));
        }

        if let Ok(repository) = Self::open(&selected) {
            let root = std::fs::canonicalize(&repository.root).unwrap_or_else(|_| repository.root.clone());
            if root == selected || selected.starts_with(&root) {
                let entry = repository.workspace_repository()?;
                return Ok(WorkspaceSnapshot {
                    root: root.display().to_string(),
                    name: root.file_name().and_then(OsStr::to_str).unwrap_or("Workspace").to_owned(),
                    kind: "repository".into(),
                    repositories: vec![entry],
                });
            }
        }

        let mut repositories = Vec::new();
        let children = std::fs::read_dir(&selected)
            .map_err(|error| GitError::Command(format!("Could not inspect {}: {error}", selected.display())))?;
        for child in children.flatten() {
            let child_path = child.path();
            if !child_path.is_dir() || child.file_name().to_string_lossy().starts_with('.') { continue; }
            let Ok(repository) = Self::open(&child_path) else { continue; };
            let repository_root = std::fs::canonicalize(&repository.root).unwrap_or_else(|_| repository.root.clone());
            let child_root = std::fs::canonicalize(&child_path).unwrap_or(child_path);
            if repository_root == child_root && let Ok(entry) = repository.workspace_repository() {
                repositories.push(entry);
            }
        }
        repositories.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
        if repositories.is_empty() { return Err(GitError::NotRepository(selected.display().to_string())); }
        Ok(WorkspaceSnapshot {
            root: selected.display().to_string(),
            name: selected.file_name().and_then(OsStr::to_str).unwrap_or("Workspace").to_owned(),
            kind: "monorepo".into(),
            repositories,
        })
    }

    pub fn create_workspace_worktrees(
        workspace_path: &str,
        selected_repositories: &[String],
        target_root: &str,
        basis: &str,
    ) -> Result<BatchWorktreeResult, GitError> {
        if !matches!(basis, "latestTag" | "defaultBranch") {
            return Err(GitError::Unsupported(format!("worktree basis {basis}")));
        }
        let workspace = Self::discover_workspace(workspace_path)?;
        if workspace.kind != "monorepo" { return Err(GitError::Unsupported("batch worktrees require a Mono Repo workspace".into())); }
        let target = PathBuf::from(target_root);
        if target.as_os_str().is_empty() { return Err(GitError::Command("Worktree directory cannot be empty.".into())); }
        std::fs::create_dir_all(&target)
            .map_err(|error| GitError::Command(format!("Could not create {}: {error}", target.display())))?;
        let allowed: std::collections::HashMap<_, _> = workspace.repositories.into_iter()
            .map(|repository| (repository.root.clone(), repository)).collect();
        let mut entries = Vec::new();

        for repository_path in selected_repositories {
            let canonical_repository_path = std::fs::canonicalize(repository_path)
                .unwrap_or_else(|_| PathBuf::from(repository_path)).display().to_string();
            let Some(repository_info) = allowed.get(&canonical_repository_path) else {
                entries.push(BatchWorktreeEntry {
                    repository: Path::new(repository_path).file_name().and_then(OsStr::to_str).unwrap_or("Repository").into(),
                    repository_path: repository_path.clone(), worktree_path: String::new(), base: None, success: false,
                    message: "Repository is not part of this workspace.".into(),
                });
                continue;
            };
            let worktree_path = target.join(&repository_info.name);
            let mut entry = BatchWorktreeEntry {
                repository: repository_info.name.clone(), repository_path: canonical_repository_path.clone(),
                worktree_path: worktree_path.display().to_string(), base: None, success: false, message: String::new(),
            };
            let result = (|| {
                if worktree_path.exists() { return Err(GitError::Command(format!("{} already exists", worktree_path.display()))); }
                let git = Self::open(&canonical_repository_path)?;
                if !git.run(["remote"])?.trim().is_empty() { git.run(["fetch", "--prune", "--tags"])?; }
                let latest_tag = git.latest_tag();
                let default_branch = git.default_branch();
                let base = if basis == "latestTag" { latest_tag.or(default_branch) } else { default_branch.or(latest_tag) }
                    .ok_or_else(|| GitError::Command("No tag or default branch is available.".into()))?;
                git.run_owned(vec!["worktree".into(), "add".into(), "--detach".into(), worktree_path.display().to_string(), base.clone()])?;
                Ok::<String, GitError>(base)
            })();
            match result {
                Ok(base) => { entry.base = Some(base.clone()); entry.success = true; entry.message = format!("Created from {base}"); }
                Err(error) => { entry.message = error.to_string(); }
            }
            entries.push(entry);
        }
        let succeeded = entries.iter().filter(|entry| entry.success).count();
        let failed = entries.len() - succeeded;
        Ok(BatchWorktreeResult { target_root: target.display().to_string(), entries, succeeded, failed })
    }

    pub fn snapshot(&self) -> Result<RepositorySnapshot, GitError> {
        let (status, changes_truncated) = self.status_limited(500)?;
        let (branch, head, upstream, ahead, behind, changes) = parse_status(&status);
        let conflicts = changes.iter().filter(|change| change.conflicted).count();
        let git_dir = self.run(["rev-parse", "--git-dir"])?;
        let git_dir = if Path::new(git_dir.trim()).is_absolute() {
            PathBuf::from(git_dir.trim())
        } else {
            self.root.join(git_dir.trim())
        };
        let state = RepositoryState {
            merging: git_dir.join("MERGE_HEAD").exists(),
            rebasing: git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists(),
            cherry_picking: git_dir.join("CHERRY_PICK_HEAD").exists(),
            reverting: git_dir.join("REVERT_HEAD").exists(),
            conflicts,
        };

        Ok(RepositorySnapshot {
            root: self.root.display().to_string(),
            name: self.root.file_name().and_then(OsStr::to_str).unwrap_or("Repository").to_owned(),
            branch,
            head,
            upstream,
            ahead,
            behind,
            state,
            changes,
            changes_truncated,
            branches: self.branches()?,
            tags: lines(self.run(["tag", "--sort=-creatordate"])?)
                .into_iter().take(200).collect(),
            remotes: lines(self.run(["remote"])?) ,
            worktrees: self.worktrees()?,
        })
    }

    fn workspace_repository(&self) -> Result<WorkspaceRepository, GitError> {
        let branch = self.run(["branch", "--show-current"] )?.trim().to_owned();
        Ok(WorkspaceRepository {
            root: self.root.display().to_string(),
            name: self.root.file_name().and_then(OsStr::to_str).unwrap_or("Repository").to_owned(),
            branch: if branch.is_empty() { "Detached HEAD".into() } else { branch },
            latest_tag: self.latest_tag(),
            default_branch: self.default_branch(),
        })
    }

    fn latest_tag(&self) -> Option<String> {
        self.run(["tag", "--sort=-creatordate"]).ok().and_then(|value| lines(value).into_iter().next())
    }

    fn default_branch(&self) -> Option<String> {
        if let Ok(value) = self.run(["symbolic-ref", "--quiet", "--short", "refs/remotes/origin/HEAD"]) {
            let value = value.trim(); if !value.is_empty() { return Some(value.to_owned()); }
        }
        for candidate in ["origin/master", "origin/main", "master", "main"] {
            if self.run(["rev-parse", "--verify", "--quiet", candidate]).is_ok() { return Some(candidate.into()); }
        }
        let current = self.run(["branch", "--show-current"]).ok()?.trim().to_owned();
        (!current.is_empty()).then_some(current)
    }

    pub fn log_page(&self, skip: usize, limit: usize) -> Result<LogPage, GitError> {
        let safe_limit = limit.clamp(1, 5_000);
        let format = "%x1e%H%x1f%h%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%ar%x1f%s%x1f%D".to_owned();
        let text = self.run([
            "log", "--all", "--decorate=short",
            &format!("--skip={skip}"), &format!("--max-count={}", safe_limit + 1),
            &format!("--format={format}"),
        ])?;
        let mut commits: Vec<_> = text
            .split(RECORD)
            .filter(|record| !record.trim().is_empty())
            .filter_map(parse_commit)
            .collect();
        let has_more = commits.len() > safe_limit;
        commits.truncate(safe_limit);
        Ok(LogPage { skip, commits, has_more })
    }

    pub fn commit_detail(&self, oid: &str) -> Result<CommitDetail, GitError> {
        validate_revision(oid)?;
        let meta = self.run(["show", "--no-patch", "--format=%H%x1f%an%x1f%ae%x1f%aI%x1f%cn%x1f%cI%x1f%s%x1f%b%x1f%P", oid])?;
        let fields: Vec<_> = meta.trim_end().split(FIELD).collect();
        if fields.len() < 9 {
            return Err(GitError::InvalidOutput("commit metadata is incomplete".into()));
        }
        let parents: Vec<String> = fields[8].split_whitespace().map(str::to_owned).collect();
        let (names, patch) = if let Some(parent) = parents.first() {
            (
                self.run_owned(vec!["diff".into(), "--name-status".into(), "-M".into(), parent.clone(), oid.into()])?,
                self.run_owned(vec!["diff".into(), "--find-renames".into(), "--no-ext-diff".into(), parent.clone(), oid.into()])?,
            )
        } else {
            (
                self.run(["diff-tree", "--root", "--no-commit-id", "--name-status", "-r", "-M", oid])?,
                self.run(["show", "--format=", "--find-renames", "--no-ext-diff", oid])?,
            )
        };
        Ok(CommitDetail {
            oid: fields[0].to_owned(), author: fields[1].to_owned(), author_email: fields[2].to_owned(),
            authored_at: fields[3].to_owned(), committer: fields[4].to_owned(), committed_at: fields[5].to_owned(),
            subject: fields[6].to_owned(), body: fields[7].trim().to_owned(),
            parents,
            files: names.lines().filter_map(parse_changed_file).collect(), patch: bounded_patch(patch),
        })
    }

    pub fn working_diff(&self, file: &str, staged: bool) -> Result<String, GitError> {
        validate_pathspec(file)?;
        if staged { self.run(["diff", "--cached", "--no-ext-diff", "--", file]) }
        else { self.run(["diff", "--no-ext-diff", "--", file]) }
    }

    pub fn set_staged(&self, files: &[String], staged: bool) -> Result<(), GitError> {
        if files.is_empty() { return Ok(()); }
        files.iter().try_for_each(|path| validate_pathspec(path))?;
        let mut args = if staged { vec!["add".to_owned(), "--".to_owned()] }
        else { vec!["restore".to_owned(), "--staged".to_owned(), "--".to_owned()] };
        args.extend(files.iter().cloned());
        self.run_owned(args)?;
        Ok(())
    }

    pub fn commit(&self, message: &str, amend: bool) -> Result<OperationResult, GitError> {
        let message = message.trim();
        if message.is_empty() { return Err(GitError::Command("Commit message cannot be empty.".into())); }
        let mut args = vec!["commit".to_owned()];
        if amend { args.push("--amend".into()); }
        args.extend(["--file".into(), "-".into()]);
        let output = self.run_with_input(args, message.as_bytes())?;
        Ok(OperationResult { summary: if amend { "Commit amended" } else { "Changes committed" }.into(), output })
    }

    pub fn remote_operation(&self, operation: &str) -> Result<OperationResult, GitError> {
        let args: &[&str] = match operation {
            "fetch" => &["fetch", "--prune"],
            "pull" => &["pull", "--ff-only"],
            "push" => &["push"],
            _ => return Err(GitError::Unsupported(operation.into())),
        };
        let output = self.run_owned(args.iter().map(|value| (*value).to_owned()).collect())?;
        Ok(OperationResult { summary: format!("{} complete", title_case(operation)), output })
    }

    pub fn checkout(&self, branch: &str, create: bool) -> Result<OperationResult, GitError> {
        validate_ref_name(branch)?;
        let output = if create { self.run(["switch", "-c", branch])? } else { self.run(["switch", branch])? };
        Ok(OperationResult { summary: format!("Switched to {branch}"), output })
    }

    pub fn add_worktree(&self, path: &str, branch: &str, create: bool) -> Result<OperationResult, GitError> {
        validate_ref_name(branch)?;
        if path.trim().is_empty() { return Err(GitError::Command("Worktree path cannot be empty.".into())); }
        let output = if create {
            self.run(["worktree", "add", "-b", branch, path])?
        } else {
            self.run(["worktree", "add", path, branch])?
        };
        Ok(OperationResult { summary: format!("Worktree added at {path}"), output })
    }

    pub fn remove_worktree(&self, path: &str, force: bool) -> Result<OperationResult, GitError> {
        let output = if force { self.run(["worktree", "remove", "--force", path])? }
        else { self.run(["worktree", "remove", path])? };
        Ok(OperationResult { summary: "Worktree removed".into(), output })
    }

    pub fn worktree_action(&self, action: &str, path: Option<&str>, force: bool) -> Result<OperationResult, GitError> {
        let output = match action {
            "prune" => self.run(["worktree", "prune"] )?,
            "lock" => self.run(["worktree", "lock", path.ok_or_else(|| GitError::Command("Worktree path is required.".into()))?])?,
            "unlock" => self.run(["worktree", "unlock", path.ok_or_else(|| GitError::Command("Worktree path is required.".into()))?])?,
            "remove" => return self.remove_worktree(path.ok_or_else(|| GitError::Command("Worktree path is required.".into()))?, force),
            _ => return Err(GitError::Unsupported(format!("worktree {action}"))),
        };
        Ok(OperationResult { summary: format!("Worktree {} complete", action), output })
    }

    pub fn history_operation(&self, operation: &str, target: &str, mode: Option<&str>) -> Result<OperationResult, GitError> {
        validate_revision_or_ref(target)?;
        let output = match operation {
            "merge" => self.run(["merge", "--no-edit", target])?,
            "cherryPick" => self.run(["cherry-pick", target])?,
            "revert" => self.run(["revert", "--no-edit", target])?,
            "reset" => {
                let flag = match mode.unwrap_or("mixed") {
                    "soft" => "--soft", "mixed" => "--mixed", "hard" => "--hard",
                    value => return Err(GitError::Unsupported(format!("reset mode {value}"))),
                };
                self.run(["reset", flag, target])?
            }
            _ => return Err(GitError::Unsupported(operation.into())),
        };
        Ok(OperationResult { summary: format!("{} complete", operation_label(operation)), output })
    }

    pub fn finish_in_progress(&self, operation: &str, action: &str) -> Result<OperationResult, GitError> {
        let command = match (operation, action) {
            ("merge", "continue") => ["merge", "--continue"], ("merge", "abort") => ["merge", "--abort"],
            ("rebase", "continue") => ["rebase", "--continue"], ("rebase", "abort") => ["rebase", "--abort"],
            ("cherryPick", "continue") => ["cherry-pick", "--continue"], ("cherryPick", "abort") => ["cherry-pick", "--abort"],
            ("revert", "continue") => ["revert", "--continue"], ("revert", "abort") => ["revert", "--abort"],
            _ => return Err(GitError::Unsupported(format!("{operation} {action}"))),
        };
        let output = self.run(command)?;
        Ok(OperationResult { summary: format!("{} {}d", operation_label(operation), action), output })
    }

    pub fn conflict_file(&self, file: &str) -> Result<ConflictFile, GitError> {
        validate_repo_relative_path(file)?;
        let read_stage = |stage: &str| self.run_owned(vec!["show".into(), format!(":{stage}:{file}")]).unwrap_or_default();
        let working_path = self.root.join(file);
        let working = std::fs::read_to_string(&working_path).unwrap_or_default();
        Ok(ConflictFile { path: file.into(), base: read_stage("1"), ours: read_stage("2"), theirs: read_stage("3"), working })
    }

    pub fn resolve_conflict(&self, file: &str, content: &str) -> Result<OperationResult, GitError> {
        validate_repo_relative_path(file)?;
        let destination = self.root.join(file);
        std::fs::write(&destination, content).map_err(|error| GitError::Command(format!("Could not write {}: {error}", destination.display())))?;
        self.run_owned(vec!["add".into(), "--".into(), file.into()])?;
        Ok(OperationResult { summary: format!("Marked {file} resolved"), output: String::new() })
    }

    pub fn rebase_plan(&self, onto: &str) -> Result<Vec<RebaseStep>, GitError> {
        validate_revision_or_ref(onto)?;
        let range = format!("{onto}..HEAD");
        let text = self.run_owned(vec!["log".into(), "--reverse".into(), "--format=%H%x1f%h%x1f%s".into(), range])?;
        Ok(text.lines().filter_map(|line| {
            let field: Vec<_> = line.split(FIELD).collect();
            (field.len() >= 3).then(|| RebaseStep { action: "pick".into(), oid: field[0].into(), short_oid: field[1].into(), subject: field[2..].join(&FIELD.to_string()) })
        }).collect())
    }

    pub fn start_interactive_rebase(&self, onto: &str, steps: &[RebaseStep]) -> Result<OperationResult, GitError> {
        use std::os::unix::fs::PermissionsExt;
        validate_revision_or_ref(onto)?;
        if steps.is_empty() { return Err(GitError::Command("The rebase plan is empty.".into())); }
        let mut todo = String::new();
        for (index, step) in steps.iter().enumerate() {
            validate_revision(&step.oid)?;
            let action = match step.action.as_str() {
                "pick" => "pick", "edit" | "reword" => "edit", "squash" => "squash", "fixup" => "fixup", "drop" => "drop",
                value => return Err(GitError::Unsupported(format!("rebase action {value}"))),
            };
            if index == 0 && matches!(action, "squash" | "fixup") { return Err(GitError::Command("The first commit cannot be squashed or fixed up.".into())); }
            let subject = step.subject.replace(['\n', '\r'], " ");
            todo.push_str(&format!("{action} {} {subject}\n", step.oid));
        }
        let temp = tempfile::tempdir().map_err(|error| GitError::Command(error.to_string()))?;
        let plan_path = temp.path().join("todo"); let editor_path = temp.path().join("sequence-editor.sh");
        std::fs::write(&plan_path, todo).map_err(|error| GitError::Command(error.to_string()))?;
        std::fs::write(&editor_path, "#!/bin/sh\ncp \"$GRAFT_REBASE_PLAN\" \"$1\"\n").map_err(|error| GitError::Command(error.to_string()))?;
        let mut permissions = std::fs::metadata(&editor_path).map_err(|error| GitError::Command(error.to_string()))?.permissions();
        permissions.set_mode(0o700); std::fs::set_permissions(&editor_path, permissions).map_err(|error| GitError::Command(error.to_string()))?;
        let output = Self::command(&self.root).args(["rebase", "-i", onto])
            .env("GIT_SEQUENCE_EDITOR", &editor_path).env("GRAFT_REBASE_PLAN", &plan_path).env("GIT_EDITOR", "true")
            .output().map_err(|error| GitError::MissingGit(error.to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout); let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}").trim().to_owned();
        if output.status.success() || self.rebase_in_progress()? {
            Ok(OperationResult { summary: if self.rebase_in_progress()? { "Rebase paused" } else { "Rebase complete" }.into(), output: combined })
        } else { Err(GitError::Command(combined)) }
    }

    pub fn diff_hunks(&self, file: &str, staged: bool) -> Result<Vec<DiffHunk>, GitError> {
        let patch = self.working_diff(file, staged)?;
        Ok(split_diff_hunks(&patch))
    }

    pub fn apply_hunk(&self, file: &str, staged: bool, index: usize) -> Result<(), GitError> {
        let hunks = self.diff_hunks(file, staged)?;
        let hunk = hunks.get(index).ok_or_else(|| GitError::Command("That change block no longer exists. Refresh and try again.".into()))?;
        let mut args = vec!["apply".to_owned(), "--cached".to_owned(), "--recount".to_owned(), "--whitespace=nowarn".to_owned()];
        if staged { args.push("--reverse".into()); }
        args.push("-".into());
        self.run_with_input(args, hunk.patch.as_bytes())?;
        Ok(())
    }

    pub fn manage_reference(&self, kind: &str, action: &str, name: &str, value: Option<&str>, force: bool) -> Result<OperationResult, GitError> {
        let output = match (kind, action) {
            ("branch", "create") => {
                validate_ref_name(name)?;
                if let Some(start) = value { validate_revision_or_ref(start)?; self.run(["switch", "-c", name, start])? }
                else { self.run(["switch", "-c", name])? }
            }
            ("branch", "delete") => { validate_ref_name(name)?; self.run(["branch", if force { "-D" } else { "-d" }, name])? }
            ("branch", "rename") => { validate_ref_name(name)?; let next = value.ok_or_else(|| GitError::Command("New branch name is required.".into()))?; validate_ref_name(next)?; self.run(["branch", "-m", name, next])? }
            ("tag", "create") => { validate_ref_name(name)?; let target = value.unwrap_or("HEAD"); validate_revision_or_ref(target)?; self.run(["tag", name, target])? }
            ("tag", "delete") => { validate_ref_name(name)?; self.run(["tag", "-d", name])? }
            ("remote", "add") => { validate_ref_name(name)?; let url = value.ok_or_else(|| GitError::Command("Remote URL is required.".into()))?; validate_remote_url(url)?; self.run(["remote", "add", name, url])? }
            ("remote", "remove") => { validate_ref_name(name)?; self.run(["remote", "remove", name])? }
            _ => return Err(GitError::Unsupported(format!("{kind} {action}"))),
        };
        Ok(OperationResult { summary: format!("{} {} complete", title_case(kind), action), output })
    }

    fn rebase_in_progress(&self) -> Result<bool, GitError> {
        let git_dir = self.run(["rev-parse", "--git-dir"])?;
        let git_dir = if Path::new(git_dir.trim()).is_absolute() { PathBuf::from(git_dir.trim()) } else { self.root.join(git_dir.trim()) };
        Ok(git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists())
    }

    fn branches(&self) -> Result<Vec<Branch>, GitError> {
        let format = "%(HEAD)%1f%(refname:short)%1f%(objectname:short)%1f%(upstream:short)%1f%(refname)".to_owned();
        let text = self.run(["for-each-ref", "refs/heads", "refs/remotes", &format!("--format={format}"), "--sort=refname"])?;
        Ok(text.lines().filter_map(|line| {
            let field: Vec<_> = line.split(FIELD).collect();
            (field.len() >= 5).then(|| Branch {
                current: field[0] == "*", name: field[1].to_owned(), oid: field[2].to_owned(),
                upstream: (!field[3].is_empty()).then(|| field[3].to_owned()), remote: field[4].starts_with("refs/remotes/"),
            })
        }).collect())
    }

    fn worktrees(&self) -> Result<Vec<Worktree>, GitError> {
        let text = self.run(["worktree", "list", "--porcelain"])?;
        Ok(text.split("\n\n").filter_map(|block| {
            let mut path = None; let mut branch = None; let mut head = String::new();
            let mut bare = false; let mut locked = false; let mut prunable = false;
            for line in block.lines() {
                if let Some(value) = line.strip_prefix("worktree ") { path = Some(value.to_owned()); }
                else if let Some(value) = line.strip_prefix("HEAD ") { head = value.chars().take(8).collect(); }
                else if let Some(value) = line.strip_prefix("branch refs/heads/") { branch = Some(value.to_owned()); }
                else if line == "bare" { bare = true; } else if line.starts_with("locked") { locked = true; }
                else if line.starts_with("prunable") { prunable = true; }
            }
            path.map(|path| Worktree { path, branch, head, bare, locked, prunable })
        }).collect())
    }

    fn run<const N: usize>(&self, args: [&str; N]) -> Result<String, GitError> { self.run_owned(args.map(str::to_owned).to_vec()) }

    fn run_owned(&self, args: Vec<String>) -> Result<String, GitError> {
        let output = Self::command(&self.root).args(&args).output().map_err(|e| GitError::MissingGit(e.to_string()))?;
        output_text(output)
    }

    fn run_with_input(&self, args: Vec<String>, input: &[u8]) -> Result<String, GitError> {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = Self::command(&self.root).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| GitError::MissingGit(e.to_string()))?;
        child.stdin.take().ok_or_else(|| GitError::Command("Could not open Git input.".into()))?
            .write_all(input).map_err(|e| GitError::Command(e.to_string()))?;
        output_text(child.wait_with_output().map_err(|e| GitError::Command(e.to_string()))?)
    }

    fn status_limited(&self, max_changes: usize) -> Result<(String, bool), GitError> {
        use std::process::Stdio;
        let mut child = Self::command(&self.root)
            .args(["status", "--porcelain=v2", "--branch", "-z"])
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
            .map_err(|error| GitError::MissingGit(error.to_string()))?;
        let stdout = child.stdout.take().ok_or_else(|| GitError::Command("Could not read Git status.".into()))?;
        let mut reader = std::io::BufReader::new(stdout);
        let mut bytes = Vec::new(); let mut record = Vec::new(); let mut changes = 0; let mut truncated = false;
        loop {
            record.clear();
            let read = reader.read_until(0, &mut record).map_err(|error| GitError::Command(error.to_string()))?;
            if read == 0 { break; }
            let is_change = record.first().is_some_and(|byte| matches!(byte, b'1' | b'2' | b'u' | b'?' | b'!'));
            if is_change && changes >= max_changes { truncated = true; break; }
            if is_change { changes += 1; }
            bytes.extend_from_slice(&record);
        }
        if truncated {
            let _ = child.kill();
            let _ = child.wait();
        } else {
            let status = child.wait().map_err(|error| GitError::Command(error.to_string()))?;
            if !status.success() { return Err(GitError::Command(format!("Git status exited with {status}."))); }
        }
        Ok((String::from_utf8_lossy(&bytes).into_owned(), truncated))
    }

    fn git_at<const N: usize>(path: &Path, args: [&str; N]) -> Result<Output, GitError> {
        Self::command(path).args(args).output().map_err(|e| GitError::MissingGit(e.to_string()))
    }

    fn command(path: &Path) -> Command {
        let mut command = Command::new("git");
        command.current_dir(path).env("LC_ALL", "C").env("GIT_PAGER", "cat").env("GIT_TERMINAL_PROMPT", "0");
        command
    }
}

fn output_text(output: Output) -> Result<String, GitError> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(GitError::Command(if message.is_empty() { format!("Git exited with {}", output.status) } else { message }))
    }
}

fn parse_status(text: &str) -> (String, String, Option<String>, usize, usize, Vec<Change>) {
    let mut branch = "HEAD".to_owned(); let mut head = String::new(); let mut upstream = None;
    let mut ahead = 0; let mut behind = 0; let mut changes = Vec::new();
    let records: Vec<_> = text.split('\0').collect(); let mut index = 0;
    while index < records.len() {
        let record = records[index];
        if let Some(value) = record.strip_prefix("# branch.head ") { branch = value.to_owned(); }
        else if let Some(value) = record.strip_prefix("# branch.oid ") { head = value.chars().take(8).collect(); }
        else if let Some(value) = record.strip_prefix("# branch.upstream ") { upstream = Some(value.to_owned()); }
        else if let Some(value) = record.strip_prefix("# branch.ab ") {
            for part in value.split_whitespace() {
                if let Some(v) = part.strip_prefix('+') { ahead = v.parse().unwrap_or(0); }
                if let Some(v) = part.strip_prefix('-') { behind = v.parse().unwrap_or(0); }
            }
        } else if record.starts_with("1 ") || record.starts_with("2 ") || record.starts_with("u ") {
            let kind = record.as_bytes()[0] as char;
            // Porcelain v2 keeps the path in the ninth ordinary field and the
            // tenth rename/copy field. Split only that far so paths containing
            // spaces remain intact.
            let split_at = if kind == '1' { 9 } else if kind == '2' { 10 } else { 11 };
            let fields: Vec<_> = record.splitn(split_at, ' ').collect();
            if fields.len() >= split_at {
                let xy = fields[1]; let path = fields[split_at - 1].to_owned();
                let old_path = if kind == '2' { index += 1; records.get(index).map(|s| (*s).to_owned()) } else { None };
                changes.push(Change { path, old_path, index_status: xy[..1].to_owned(), worktree_status: xy[1..2].to_owned(), staged: &xy[..1] != ".", conflicted: kind == 'u' });
            }
        } else if let Some(path) = record.strip_prefix("? ") {
            changes.push(Change { path: path.to_owned(), old_path: None, index_status: "?".into(), worktree_status: "?".into(), staged: false, conflicted: false });
        }
        index += 1;
    }
    (branch, head, upstream, ahead, behind, changes)
}

fn parse_commit(record: &str) -> Option<CommitRow> {
    let field: Vec<_> = record.trim_start_matches('\n').split(FIELD).collect();
    if field.len() < 9 { return None; }
    Some(CommitRow { oid: field[0].into(), short_oid: field[1].into(), parents: field[2].split_whitespace().map(str::to_owned).collect(), author: field[3].into(), author_email: field[4].into(), timestamp: field[5].parse().unwrap_or(0), relative_date: field[6].into(), subject: field[7].into(), decorations: field[8].split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect() })
}

fn parse_changed_file(line: &str) -> Option<ChangedFile> {
    let mut part = line.split('\t'); let status = part.next()?.to_owned(); let first = part.next()?.to_owned(); let second = part.next();
    Some(ChangedFile { status, path: second.unwrap_or(&first).to_owned(), old_path: second.map(|_| first) })
}

fn split_diff_hunks(patch: &str) -> Vec<DiffHunk> {
    let mut file_header = String::new(); let mut bodies: Vec<(String, String)> = Vec::new();
    let mut current_header = String::new(); let mut current_body = String::new();
    for line in patch.split_inclusive('\n') {
        if line.starts_with("@@") {
            if !current_header.is_empty() { bodies.push((current_header, current_body)); current_body = String::new(); }
            current_header = line.trim_end().to_owned(); current_body.push_str(line);
        } else if current_header.is_empty() { file_header.push_str(line); } else { current_body.push_str(line); }
    }
    if !current_header.is_empty() { bodies.push((current_header, current_body)); }
    bodies.into_iter().enumerate().map(|(index, (header, body))| {
        let additions = body.lines().filter(|line| line.starts_with('+') && !line.starts_with("+++")).count();
        let deletions = body.lines().filter(|line| line.starts_with('-') && !line.starts_with("---")).count();
        DiffHunk { index, header, patch: format!("{file_header}{body}"), additions, deletions }
    }).collect()
}

fn validate_revision(value: &str) -> Result<(), GitError> {
    if !value.is_empty() && value.len() <= 64 && value.chars().all(|c| c.is_ascii_hexdigit()) { Ok(()) }
    else { Err(GitError::Command("Invalid commit identifier.".into())) }
}

fn validate_ref_name(value: &str) -> Result<(), GitError> {
    if !value.trim().is_empty() && !value.starts_with('-') && !value.contains(char::is_whitespace) { Ok(()) }
    else { Err(GitError::Command("Invalid branch name.".into())) }
}

fn validate_revision_or_ref(value: &str) -> Result<(), GitError> {
    if !value.trim().is_empty() && value.len() <= 512 && !value.starts_with('-') && !value.contains(char::is_whitespace) && !value.contains('\0') {
        Ok(())
    } else { Err(GitError::Command("Invalid revision or branch name.".into())) }
}

fn validate_repo_relative_path(value: &str) -> Result<(), GitError> {
    let path = Path::new(value);
    if !value.is_empty() && !path.is_absolute() && path.components().all(|part| !matches!(part, std::path::Component::ParentDir)) {
        Ok(())
    } else { Err(GitError::Command("Invalid repository path.".into())) }
}

fn validate_remote_url(value: &str) -> Result<(), GitError> {
    if !value.trim().is_empty() && value.len() <= 4_096 && !value.contains(['\n', '\r', '\0']) { Ok(()) }
    else { Err(GitError::Command("Invalid remote URL.".into())) }
}

fn validate_pathspec(value: &str) -> Result<(), GitError> {
    if !value.is_empty() && !value.contains('\0') { Ok(()) } else { Err(GitError::Command("Invalid file path.".into())) }
}

fn lines(value: String) -> Vec<String> { value.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_owned).collect() }
fn title_case(value: &str) -> String { let mut chars = value.chars(); chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default() }
fn operation_label(value: &str) -> &str { match value { "cherryPick" => "Cherry-pick", value => value } }

fn bounded_patch(mut patch: String) -> String {
    const MAX_PATCH_BYTES: usize = 512 * 1024;
    if patch.len() <= MAX_PATCH_BYTES { return patch; }
    let mut end = MAX_PATCH_BYTES;
    while !patch.is_char_boundary(end) { end -= 1; }
    patch.truncate(end);
    patch.push_str("\n\n[Diff truncated at 512 KiB to protect application memory. Open the file or use system Git for the complete patch.]\n");
    patch
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git").args(["init", "-b", "main"]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["config", "user.name", "Graft Test"]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["config", "user.email", "graft@example.test"]).current_dir(dir.path()).output().unwrap();
        fs::write(dir.path().join("README.md"), "one\n").unwrap();
        Command::new("git").args(["add", "."]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["commit", "-m", "Initial commit"]).current_dir(dir.path()).output().unwrap();
        dir
    }

    fn initialize_repository(path: &Path, branch: &str) {
        Command::new("git").args(["init", "-b", branch]).current_dir(path).output().unwrap();
        Command::new("git").args(["config", "user.name", "Graft Test"]).current_dir(path).output().unwrap();
        Command::new("git").args(["config", "user.email", "graft@example.test"]).current_dir(path).output().unwrap();
        fs::write(path.join("README.md"), "workspace\n").unwrap();
        Command::new("git").args(["add", "."]).current_dir(path).output().unwrap();
        Command::new("git").args(["commit", "-m", "Initial"]).current_dir(path).output().unwrap();
    }

    #[test]
    fn workspace_discovers_immediate_git_repositories() {
        let workspace = tempfile::tempdir().unwrap();
        let alpha = workspace.path().join("alpha"); let beta = workspace.path().join("beta");
        fs::create_dir(&alpha).unwrap(); fs::create_dir(&beta).unwrap();
        initialize_repository(&alpha, "main"); initialize_repository(&beta, "main");

        let discovered = GitService::discover_workspace(workspace.path()).unwrap();
        assert_eq!(discovered.kind, "monorepo");
        assert_eq!(discovered.repositories.iter().map(|repository| repository.name.as_str()).collect::<Vec<_>>(), ["alpha", "beta"]);
    }

    #[test]
    fn workspace_creates_one_detached_worktree_per_selected_repository() {
        let workspace = tempfile::tempdir().unwrap();
        let alpha = workspace.path().join("alpha"); let beta = workspace.path().join("beta");
        fs::create_dir(&alpha).unwrap(); fs::create_dir(&beta).unwrap();
        initialize_repository(&alpha, "master"); initialize_repository(&beta, "main");
        let target = workspace.path().join("Worktree");

        let result = GitService::create_workspace_worktrees(
            workspace.path().to_str().unwrap(), &[alpha.display().to_string()], target.to_str().unwrap(), "defaultBranch",
        ).unwrap();
        assert_eq!((result.succeeded, result.failed), (1, 0), "{:?}", result.entries);
        assert!(target.join("alpha/README.md").exists());
    }

    #[test]
    fn snapshot_tracks_working_and_staged_changes() {
        let dir = fixture(); let git = GitService::open(dir.path()).unwrap();
        fs::write(dir.path().join("README.md"), "one\ntwo\n").unwrap();
        let snapshot = git.snapshot().unwrap();
        assert_eq!(snapshot.branch, "main"); assert_eq!(snapshot.changes.len(), 1); assert!(!snapshot.changes[0].staged);
        assert!(snapshot.branches.iter().any(|branch| branch.name == "main" && branch.current));
        git.set_staged(&["README.md".into()], true).unwrap();
        assert!(git.snapshot().unwrap().changes[0].staged);
    }

    #[test]
    fn log_and_detail_are_structured() {
        let dir = fixture(); let git = GitService::open(dir.path()).unwrap();
        let log = git.log_page(0, 50).unwrap();
        assert_eq!(log.commits[0].subject, "Initial commit");
        let detail = git.commit_detail(&log.commits[0].oid).unwrap();
        assert_eq!(detail.author, "Graft Test"); assert!(detail.files.iter().any(|f| f.path == "README.md"));
    }

    #[test]
    fn merge_detail_compares_against_first_parent() {
        let dir = fixture();
        Command::new("git").args(["checkout", "-b", "topic"]).current_dir(dir.path()).output().unwrap();
        fs::write(dir.path().join("topic.txt"), "topic\n").unwrap();
        Command::new("git").args(["add", "."]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["commit", "-m", "Topic change"]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["checkout", "main"]).current_dir(dir.path()).output().unwrap();
        fs::write(dir.path().join("main.txt"), "main\n").unwrap();
        Command::new("git").args(["add", "."]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["commit", "-m", "Main change"]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["merge", "--no-ff", "topic", "-m", "Merge topic"]).current_dir(dir.path()).output().unwrap();
        let git = GitService::open(dir.path()).unwrap();
        let merge_oid = git.run(["rev-parse", "HEAD"]).unwrap();
        let detail = git.commit_detail(merge_oid.trim()).unwrap();
        assert_eq!(detail.parents.len(), 2);
        assert!(detail.files.iter().any(|file| file.path == "topic.txt"));
        assert!(detail.patch.contains("topic.txt"));
    }

    #[test]
    fn unmerged_status_keeps_the_exact_file_path() {
        let record = "# branch.oid 916158ce\0# branch.head main\0u UU N... 100644 100644 100644 100644 1e15e0100000000000000000000000000000000 1b5411a000000000000000000000000000000000 388ab35000000000000000000000000000000000 plan.txt\0";
        let (_, _, _, _, _, changes) = parse_status(record);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "plan.txt");
        assert!(changes[0].conflicted);
    }

    #[test]
    fn ordinary_status_does_not_prefix_the_path_with_the_index_oid() {
        let record = "# branch.oid 916158ce\0# branch.head main\x001 .M N... 100644 100644 100644 82fc3886d45c663b85731dc7904a804b401db795 82fc3886d45c663b85731dc7904a804b401db795 .gitignore\0";
        let (_, _, _, _, _, changes) = parse_status(record);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, ".gitignore");
        assert_eq!(changes[0].worktree_status, "M");
    }

    #[test]
    fn renamed_status_keeps_both_paths_with_spaces() {
        let record = "# branch.oid 916158ce\0# branch.head main\x002 R. N... 100644 100644 100644 82fc3886d45c663b85731dc7904a804b401db795 82fc3886d45c663b85731dc7904a804b401db795 R100 docs/new name.md\0docs/old name.md\0";
        let (_, _, _, _, _, changes) = parse_status(record);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "docs/new name.md");
        assert_eq!(changes[0].old_path.as_deref(), Some("docs/old name.md"));
    }

    #[test]
    fn worktree_and_interactive_rebase_use_system_git() {
        let dir = fixture(); let git = GitService::open(dir.path()).unwrap();
        for number in 2..=3 {
            fs::write(dir.path().join("README.md"), format!("commit {number}\n")).unwrap();
            Command::new("git").args(["commit", "-am", &format!("Commit {number}")]).current_dir(dir.path()).output().unwrap();
        }
        let plan = git.rebase_plan("HEAD~2").unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].subject, "Commit 2");
        let result = git.start_interactive_rebase("HEAD~2", &plan).unwrap();
        assert_eq!(result.summary, "Rebase complete");

        let worktree_parent = tempfile::tempdir().unwrap();
        let path = worktree_parent.path().join("review");
        git.add_worktree(path.to_str().unwrap(), "review-branch", true).unwrap();
        assert!(path.join(".git").exists());
        assert_eq!(git.worktrees().unwrap().len(), 2);
        git.remove_worktree(path.to_str().unwrap(), false).unwrap();
        assert_eq!(git.worktrees().unwrap().len(), 1);
    }

    #[test]
    fn individual_hunks_can_move_in_and_out_of_the_index() {
        let dir = fixture(); let git = GitService::open(dir.path()).unwrap();
        let baseline = (1..=24).map(|line| format!("line {line}\n")).collect::<String>();
        fs::write(dir.path().join("README.md"), &baseline).unwrap();
        Command::new("git").args(["commit", "-am", "Add lines"]).current_dir(dir.path()).output().unwrap();
        let changed = baseline.replace("line 2\n", "line two\n").replace("line 22\n", "line twenty-two\n");
        fs::write(dir.path().join("README.md"), changed).unwrap();
        let hunks = git.diff_hunks("README.md", false).unwrap();
        assert_eq!(hunks.len(), 2);
        git.apply_hunk("README.md", false, 0).unwrap();
        assert_eq!(git.diff_hunks("README.md", true).unwrap().len(), 1);
        assert_eq!(git.diff_hunks("README.md", false).unwrap().len(), 1);
        git.apply_hunk("README.md", true, 0).unwrap();
        assert!(git.diff_hunks("README.md", true).unwrap().is_empty());
    }

    #[test]
    #[ignore = "requires GRAFT_BENCH_REPO"]
    fn benchmark_repository_stays_bounded() {
        let path = std::env::var("GRAFT_BENCH_REPO").expect("set GRAFT_BENCH_REPO");
        let git = GitService::open(path).unwrap();
        let snapshot = git.snapshot().unwrap();
        assert!(snapshot.changes.len() <= 2_000);
        let log = git.log_page(0, 500).unwrap();
        assert_eq!(log.commits.len(), 500);
    }
}
