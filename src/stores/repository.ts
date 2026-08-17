import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../lib/bridge";
import { errorMessage } from "../lib/errors";
import type { CommitDetail, CommitRow, RepositorySnapshot } from "../types";

const PAGE_SIZE = 500;
const NOTICE_TIMEOUT = 6000;

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
    } catch (caught) { error.value = errorMessage(caught); }
    finally { loading.value = false; }
  }

  async function restore() {
    const path = new URLSearchParams(window.location.search).get("repo") ?? localStorage.getItem("graft.lastRepository");
    if (path) await loadRepository(path);
  }

  async function refresh() {
    if (!repository.value) return;
    try { repository.value = await api.refresh(repository.value.root); }
    catch (caught) { error.value = errorMessage(caught); }
  }

  async function loadMore() {
    if (!repository.value || loadingMore.value) return;
    loadingMore.value = true;
    try {
      const page = await api.log(repository.value.root, commits.value.length, PAGE_SIZE);
      commits.value.push(...page.commits); hasMore.value = page.hasMore;
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

  return { repository, commits, selectedCommit, detail, loading, loadingMore, hasMore, error, notice, query, visibleCommits, chooseRepository, loadRepository, restore, refresh, loadMore, selectCommit, setStaged, commit, remote, notify, clearMessage };
});
