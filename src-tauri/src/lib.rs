mod git;

use git::{GitError, GitService};
use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandError {
    kind: &'static str,
    message: String,
    recovery: Option<String>,
}

impl From<GitError> for CommandError {
    fn from(value: GitError) -> Self {
        Self {
            kind: value.kind(),
            message: value.to_string(),
            recovery: value.recovery().map(str::to_owned),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
async fn open_repository(path: String) -> CommandResult<git::RepositorySnapshot> {
    eprintln!("graft: opening {path}");
    let result = tauri::async_runtime::spawn_blocking(move || GitService::open(path).and_then(|git| git.snapshot()))
        .await
        .map_err(|error| CommandError {
            kind: "internal",
            message: error.to_string(),
            recovery: None,
        })?
        .map_err(Into::into);
    eprintln!("graft: repository snapshot complete");
    result
}

#[tauri::command]
async fn refresh_repository(path: String) -> CommandResult<git::RepositorySnapshot> {
    open_repository(path).await
}

#[tauri::command]
async fn log_page(path: String, skip: usize, limit: usize) -> CommandResult<git::LogPage> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.log_page(skip, limit))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn commit_detail(path: String, oid: String) -> CommandResult<git::CommitDetail> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.commit_detail(&oid))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn working_diff(path: String, file: String, staged: bool) -> CommandResult<String> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.working_diff(&file, staged))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn set_staged(path: String, files: Vec<String>, staged: bool) -> CommandResult<git::RepositorySnapshot> {
    tauri::async_runtime::spawn_blocking(move || {
        let git = GitService::open(path)?;
        git.set_staged(&files, staged)?;
        git.snapshot()
    })
    .await
    .map_err(internal_join_error)?
    .map_err(Into::into)
}

#[tauri::command]
async fn commit_changes(path: String, message: String, amend: bool) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.commit(&message, amend))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn run_remote(path: String, operation: String) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.remote_operation(&operation))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn checkout_branch(path: String, branch: String, create: bool) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.checkout(&branch, create))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn add_worktree(path: String, worktree_path: String, branch: String, create_branch: bool) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || {
        GitService::open(path)?.add_worktree(&worktree_path, &branch, create_branch)
    })
    .await
    .map_err(internal_join_error)?
    .map_err(Into::into)
}

#[tauri::command]
async fn remove_worktree(path: String, worktree_path: String, force: bool) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.remove_worktree(&worktree_path, force))
        .await
        .map_err(internal_join_error)?
        .map_err(Into::into)
}

#[tauri::command]
async fn run_history_operation(path: String, operation: String, target: String, mode: Option<String>) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.history_operation(&operation, &target, mode.as_deref()))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn finish_in_progress(path: String, operation: String, action: String) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.finish_in_progress(&operation, &action))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn conflict_file(path: String, file: String) -> CommandResult<git::ConflictFile> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.conflict_file(&file))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn resolve_conflict(path: String, file: String, content: String) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.resolve_conflict(&file, &content))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn rebase_plan(path: String, onto: String) -> CommandResult<Vec<git::RebaseStep>> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.rebase_plan(&onto))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn start_interactive_rebase(path: String, onto: String, steps: Vec<git::RebaseStep>) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.start_interactive_rebase(&onto, &steps))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn diff_hunks(path: String, file: String, staged: bool) -> CommandResult<Vec<git::DiffHunk>> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.diff_hunks(&file, staged))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn apply_hunk(path: String, file: String, staged: bool, index: usize) -> CommandResult<git::RepositorySnapshot> {
    tauri::async_runtime::spawn_blocking(move || {
        let git = GitService::open(path)?;
        git.apply_hunk(&file, staged, index)?;
        git.snapshot()
    }).await.map_err(internal_join_error)?.map_err(Into::into)
}

#[tauri::command]
async fn manage_reference(path: String, kind: String, action: String, name: String, value: Option<String>, force: bool) -> CommandResult<git::OperationResult> {
    tauri::async_runtime::spawn_blocking(move || GitService::open(path)?.manage_reference(&kind, &action, &name, value.as_deref(), force))
        .await.map_err(internal_join_error)?.map_err(Into::into)
}

fn internal_join_error(error: impl std::fmt::Display) -> CommandError {
    CommandError { kind: "internal", message: error.to_string(), recovery: None }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Graft — Development")?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_repository,
            refresh_repository,
            log_page,
            commit_detail,
            working_diff,
            set_staged,
            commit_changes,
            run_remote,
            checkout_branch,
            add_worktree,
            remove_worktree,
            run_history_operation,
            finish_in_progress,
            conflict_file,
            resolve_conflict,
            rebase_plan,
            start_interactive_rebase,
            diff_hunks,
            apply_hunk,
            manage_reference,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Graft");
}
