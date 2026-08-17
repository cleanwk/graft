import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../lib/bridge";
import type { CommandFailure, CommitDetail, CommitRow, RepositorySnapshot } from "../types";

const PAGE_SIZE = 500;

export function failureMessage(error: unknown): string {
  if (typeof error === "string") return error;
  const value = error as CommandFailure;
  return [value?.message ?? (error as Error)?.message ?? "The operation failed.", value?.recovery].filter(Boolean).join(" ");
}

export const useRepositoryStore = defineStore("repository", () => {
  const repository = ref<RepositorySnapshot>();
  const commits = ref<CommitRow[]>([]);
  const selectedCommit = ref<CommitRow>();
  const detail = ref<CommitDetail>();
  const loading = ref(false);
  const loadingMore = ref(false);
  const hasMore = ref(false);
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

  async function chooseRepository() {
    const path = await open({ directory: true, multiple: false, title: "Open Git Repository" });
    if (typeof path === "string") await loadRepository(path);
  }

  async function loadRepository(path: string) {
    loading.value = true; error.value = ""; notice.value = "";
    try {
      repository.value = await api.open(path);
      api.watch(path).catch(() => undefined);
      localStorage.setItem("graft.lastRepository", path);
      commits.value = []; detail.value = undefined; selectedCommit.value = undefined;
      await loadMore();
      if (commits.value[0]) await selectCommit(commits.value[0]);
    } catch (caught) { error.value = failureMessage(caught); }
    finally { loading.value = false; }
  }

  async function restore() {
    const path = new URLSearchParams(window.location.search).get("repo") ?? localStorage.getItem("graft.lastRepository");
    if (path) await loadRepository(path);
  }

  async function refresh() {
    if (!repository.value) return;
    try { repository.value = await api.refresh(repository.value.root); }
    catch (caught) { error.value = failureMessage(caught); }
  }

  async function loadMore() {
    if (!repository.value || loadingMore.value) return;
    loadingMore.value = true;
    try {
      const page = await api.log(repository.value.root, commits.value.length, PAGE_SIZE);
      commits.value.push(...page.commits); hasMore.value = page.hasMore;
    } catch (caught) { error.value = failureMessage(caught); }
    finally { loadingMore.value = false; }
  }

  async function selectCommit(commit: CommitRow) {
    if (!repository.value) return;
    selectedCommit.value = commit; detail.value = undefined;
    try { detail.value = await api.detail(repository.value.root, commit.oid); }
    catch (caught) { error.value = failureMessage(caught); }
  }

  async function setStaged(path: string, staged: boolean) {
    if (!repository.value) return;
    try { repository.value = await api.stage(repository.value.root, [path], staged); }
    catch (caught) { error.value = failureMessage(caught); }
  }

  async function commit(message: string, amend: boolean, pushAfter = false) {
    if (!repository.value) return false;
    try {
      const result = await api.commit(repository.value.root, message, amend, pushAfter);
      notice.value = result.summary; await refresh(); commits.value = []; await loadMore(); return true;
    } catch (caught) { error.value = failureMessage(caught); return false; }
  }

  async function remote(operation: "fetch" | "pull" | "push") {
    if (!repository.value) return;
    notice.value = `${operation[0].toUpperCase()}${operation.slice(1)}ing…`;
    try { const result = await api.remote(repository.value.root, operation); notice.value = result.summary; await refresh(); }
    catch (caught) { notice.value = ""; error.value = failureMessage(caught); }
  }

  function clearMessage() { error.value = ""; notice.value = ""; }

  return { repository, commits, selectedCommit, detail, loading, loadingMore, hasMore, error, notice, query, visibleCommits, chooseRepository, loadRepository, restore, refresh, loadMore, selectCommit, setStaged, commit, remote, clearMessage };
});
