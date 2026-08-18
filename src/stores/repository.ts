import { computed, ref, shallowRef } from "vue";
import { defineStore } from "pinia";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../lib/bridge";
import { errorMessage } from "../lib/errors";
import type { CommitDetail, CommitRow, RepositorySnapshot, WorkspaceSnapshot } from "../types";

const PAGE_SIZE = 250;
const MAX_LOADED_COMMITS = 2_000;
const NOTICE_TIMEOUT = 6000;

export const useRepositoryStore = defineStore("repository", () => {
  const workspace = shallowRef<WorkspaceSnapshot>();
  const repository = shallowRef<RepositorySnapshot>();
  const commits = shallowRef<CommitRow[]>([]);
  const selectedCommit = shallowRef<CommitRow>();
  const detail = shallowRef<CommitDetail>();
  const loading = ref(false);
  const loadingMore = ref(false);
  const hasMore = ref(false);
  const historyCapped = ref(false);
  const error = ref("");
  const notice = ref("");
  const query = ref("");

  const visibleCommits = computed(() => {
    const needle = query.value.trim().toLowerCase();
    if (!needle) return commits.value;
    return commits.value.filter((commit) =>
      [commit.subject, commit.author, commit.shortOid, ...commit.decorations].some((value) => value.toLowerCase().includes(needle)),
    );
  });

  async function chooseWorkspace() {
    const path = await open({ directory: true, multiple: false, title: "Open Workspace" });
    if (typeof path === "string") await loadWorkspace(path);
  }

  async function loadWorkspace(path: string, preferredRepository?: string) {
    loading.value = true; error.value = ""; notice.value = "";
    const previousWorkspace = workspace.value;
    try {
      const discovered = await api.workspace(path);
      workspace.value = discovered;
      const selected = discovered.repositories.find((item) => item.root === preferredRepository)
        ?? discovered.repositories.find((item) => item.root === localStorage.getItem(`graft.workspaceRepository:${discovered.root}`))
        ?? discovered.repositories[0];
      if (!selected) throw new Error("The workspace does not contain a Git repository.");
      await loadRepository(selected.root);
      localStorage.setItem("graft.lastWorkspace", discovered.root);
    } catch (caught) { workspace.value = previousWorkspace; error.value = errorMessage(caught); }
    finally { loading.value = false; }
  }

  async function loadRepository(path: string) {
    repository.value = await api.open(path);
    api.watch(repository.value.root).catch(() => undefined);
    localStorage.setItem("graft.lastRepository", repository.value.root);
    if (workspace.value) localStorage.setItem(`graft.workspaceRepository:${workspace.value.root}`, repository.value.root);
    else workspace.value = { root: repository.value.root, name: repository.value.name, kind: "repository", repositories: [{ root: repository.value.root, name: repository.value.name, branch: repository.value.branch }] };
    commits.value = []; detail.value = undefined; selectedCommit.value = undefined; hasMore.value = false; historyCapped.value = false;
    await loadMore();
    if (commits.value[0]) await selectCommit(commits.value[0]);
  }

  async function selectWorkspaceRepository(path: string) {
    if (!workspace.value?.repositories.some((item) => item.root === path) || repository.value?.root === path) return;
    loading.value = true; error.value = "";
    try { await loadRepository(path); }
    catch (caught) { error.value = errorMessage(caught); }
    finally { loading.value = false; }
  }

  async function restore() {
    const repositoryPath = new URLSearchParams(window.location.search).get("repo");
    const path = repositoryPath ?? localStorage.getItem("graft.lastWorkspace") ?? localStorage.getItem("graft.lastRepository");
    if (path) await loadWorkspace(path, repositoryPath ?? undefined);
  }

  let refreshPromise: Promise<void> | undefined;
  let refreshQueued = false;
  async function refresh() {
    if (!repository.value) return;
    if (refreshPromise) { refreshQueued = true; return refreshPromise; }
    refreshPromise = (async () => {
      do {
        refreshQueued = false;
        try { if (repository.value) repository.value = await api.refresh(repository.value.root); }
        catch (caught) { error.value = errorMessage(caught); }
      } while (refreshQueued);
    })();
    try { await refreshPromise; } finally { refreshPromise = undefined; }
  }

  async function loadMore() {
    if (!repository.value || loadingMore.value) return;
    loadingMore.value = true;
    try {
      const remaining = MAX_LOADED_COMMITS - commits.value.length;
      if (remaining <= 0) { hasMore.value = false; historyCapped.value = true; return; }
      const page = await api.log(repository.value.root, commits.value.length, Math.min(PAGE_SIZE, remaining));
      commits.value = [...commits.value, ...page.commits];
      historyCapped.value = page.hasMore && commits.value.length >= MAX_LOADED_COMMITS;
      hasMore.value = page.hasMore && !historyCapped.value;
    } catch (caught) { error.value = errorMessage(caught); }
    finally { loadingMore.value = false; }
  }

  let detailRequest = 0;
  async function selectCommit(commit: CommitRow) {
    if (!repository.value) return;
    selectedCommit.value = commit; detail.value = undefined;
    const request = ++detailRequest;
    try {
      const result = await api.detail(repository.value.root, commit.oid);
      if (request === detailRequest) detail.value = result;
    } catch (caught) { if (request === detailRequest) error.value = errorMessage(caught); }
  }

  async function setStaged(path: string, staged: boolean) {
    if (!repository.value) return;
    try { repository.value = await api.stage(repository.value.root, [path], staged); }
    catch (caught) { error.value = errorMessage(caught); }
  }

  async function commit(message: string, amend: boolean, pushAfter = false) {
    if (!repository.value) return false;
    try {
      const result = await api.commit(repository.value.root, message, amend, pushAfter);
      notify(result.summary); await refresh(); commits.value = []; await loadMore(); return true;
    } catch (caught) { error.value = errorMessage(caught); return false; }
  }

  async function remote(operation: "fetch" | "pull" | "push") {
    if (!repository.value) return;
    notice.value = `${operation[0].toUpperCase()}${operation.slice(1)}ing…`;
    try { const result = await api.remote(repository.value.root, operation); notify(result.summary); await refresh(); }
    catch (caught) { notice.value = ""; error.value = errorMessage(caught); }
  }

  let noticeTimer: number | undefined;
  function notify(message: string) {
    notice.value = message;
    window.clearTimeout(noticeTimer);
    noticeTimer = window.setTimeout(() => { notice.value = ""; }, NOTICE_TIMEOUT);
  }

  function clearMessage() { error.value = ""; notice.value = ""; window.clearTimeout(noticeTimer); }

  return { workspace, repository, commits, selectedCommit, detail, loading, loadingMore, hasMore, historyCapped, error, notice, query, visibleCommits, chooseWorkspace, loadWorkspace, loadRepository, selectWorkspaceRepository, restore, refresh, loadMore, selectCommit, setStaged, commit, remote, notify, clearMessage };
});
