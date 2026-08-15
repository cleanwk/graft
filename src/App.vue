<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { Archive, ArrowDownToLine, ArrowUpFromLine, ChevronDown, CircleAlert, FolderOpen, GitBranch, GitMerge, GitPullRequestArrow, LoaderCircle, Palette, PanelLeftClose, RefreshCw, RotateCcw, Search, TreePine, X } from "@lucide/vue";
import CommitGraph from "./components/CommitGraph.vue";
import CommitPanel from "./components/CommitPanel.vue";
import CommitToolWindow from "./components/CommitToolWindow.vue";
import RepositorySidebar from "./components/RepositorySidebar.vue";
import WorktreeDialog from "./components/WorktreeDialog.vue";
import HistoryOperationDialog from "./components/HistoryOperationDialog.vue";
import ConflictResolver from "./components/ConflictResolver.vue";
import RebaseDialog from "./components/RebaseDialog.vue";
import HunkSelector from "./components/HunkSelector.vue";
import NewReferenceDialog from "./components/NewReferenceDialog.vue";
import UpdateBanner from "./components/UpdateBanner.vue";
import { useRepositoryStore } from "./stores/repository";
import { api } from "./lib/bridge";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { themes, useTheme } from "./lib/theme";

const store = useRepositoryStore();
const showSidebar = ref(true); const showCommit = ref(true); const worktreeDialog = ref(false); const commitTool = ref<InstanceType<typeof CommitToolWindow>>();
const historyOperation = ref<"merge" | "cherryPick" | "revert" | "reset">(); const conflictFile = ref("");
const rebaseDialog = ref(false);
const hunkFile = ref("");
const newReference = ref<"branch" | "tag" | "remote">();
const searchInput = ref<HTMLInputElement>();
const theme = useTheme();
const sidebarWidth = ref(Number(localStorage.getItem("graft.sidebarWidth")) || 222);
const commitWidth = ref(Number(localStorage.getItem("graft.commitWidth")) || 286);
const shellStyle = computed(() => ({ "--sidebar-width": `${sidebarWidth.value}px`, "--commit-width": `${commitWidth.value}px` }));
let stopInvalidation: UnlistenFn | undefined; let refreshTimer: number | undefined;
const branchState = computed(() => store.repository ? `${store.repository.ahead ? `↑${store.repository.ahead}` : ''}${store.repository.behind ? ` ↓${store.repository.behind}` : ''}` : "");
const activeOperation = computed(() => { const state = store.repository?.state; if (state?.rebasing) return "rebase"; if (state?.merging) return "merge"; if (state?.cherryPicking) return "cherryPick"; if (state?.reverting) return "revert"; return ""; });

async function doCommit(message: string, amend: boolean, pushAfter: boolean) { if (await store.commit(message, amend, pushAfter)) commitTool.value?.clear(); }
function worktreeComplete(message: string) { worktreeDialog.value = false; store.notice = message; store.refresh(); }
function operationComplete(message: string) { historyOperation.value = undefined; store.notice = message; store.refresh(); }
function conflictComplete(message: string) { conflictFile.value = ""; store.notice = message; store.refresh(); }
function rebaseComplete(message: string) { rebaseDialog.value = false; store.notice = message; store.refresh(); }
function referenceComplete(message: string) { newReference.value = undefined; store.notice = message; store.refresh(); }
async function checkout(branch: string) { if (!store.repository) return; try { const result = await api.checkout(store.repository.root, branch, false); store.notice = result.summary; await store.refresh(); } catch (caught) { store.error = String(caught); } }
async function openWorktree(path: string) { try { await api.openWindow(path); } catch (caught) { store.error = String(caught); } }
async function finishOperation(action: "continue" | "abort") { if (!store.repository || !activeOperation.value) return; try { const result = await api.finishOperation(store.repository.root, activeOperation.value, action); store.notice = result.summary; await store.refresh(); } catch (caught) { store.error = String(caught); await store.refresh(); } }
function shortcuts(event: KeyboardEvent) { if (!event.metaKey) return; if (event.key.toLowerCase() === "o") { event.preventDefault(); store.chooseRepository(); } else if (event.key.toLowerCase() === "f") { event.preventDefault(); searchInput.value?.focus(); } else if (event.key === "Enter") { event.preventDefault(); commitTool.value?.submit(); } }
function fitPaneWidths() {
  const available = Math.max(416, window.innerWidth - 31 - 420);
  sidebarWidth.value = Math.min(Math.max(sidebarWidth.value, 176), Math.min(360, available - 240));
  commitWidth.value = Math.min(Math.max(commitWidth.value, 240), Math.min(420, available - sidebarWidth.value));
}
function persistPaneWidths() {
  localStorage.setItem("graft.sidebarWidth", String(Math.round(sidebarWidth.value)));
  localStorage.setItem("graft.commitWidth", String(Math.round(commitWidth.value)));
}
function startPaneResize(pane: "sidebar" | "commit", event: PointerEvent) {
  event.preventDefault();
  const shell = (event.currentTarget as HTMLElement).closest(".app-shell") as HTMLElement;
  const bounds = shell.getBoundingClientRect();
  document.body.classList.add("is-resizing-pane");
  const move = (moveEvent: PointerEvent) => {
    if (pane === "sidebar") sidebarWidth.value = moveEvent.clientX - bounds.left;
    else commitWidth.value = bounds.right - moveEvent.clientX - 31;
    fitPaneWidths();
  };
  const stop = () => {
    document.body.classList.remove("is-resizing-pane");
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", stop);
    persistPaneWidths();
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", stop, { once: true });
}
function resizePaneWithKeyboard(pane: "sidebar" | "commit", event: KeyboardEvent) {
  if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
  event.preventDefault();
  const direction = event.key === "ArrowRight" ? 1 : -1;
  if (pane === "sidebar") sidebarWidth.value += direction * 16;
  else commitWidth.value -= direction * 16;
  fitPaneWidths(); persistPaneWidths();
}
onMounted(async () => { fitPaneWidths(); window.addEventListener("resize", fitPaneWidths); window.addEventListener("keydown", shortcuts); stopInvalidation = await listen("repository-invalidated", () => { window.clearTimeout(refreshTimer); refreshTimer = window.setTimeout(() => store.refresh(), 300); }); store.restore(); });
onBeforeUnmount(() => { window.removeEventListener("resize", fitPaneWidths); window.removeEventListener("keydown", shortcuts); stopInvalidation?.(); window.clearTimeout(refreshTimer); });
</script>

<template>
  <main class="app-shell" :class="{ 'sidebar-hidden': !showSidebar, 'commit-hidden': !showCommit }" :style="shellStyle">
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-leading">
        <div class="traffic-space" data-tauri-drag-region />
        <button class="icon-button" title="Show or hide repository tree" aria-label="Toggle repository tree" @click="showSidebar = !showSidebar"><PanelLeftClose :size="15" /></button>
        <button class="icon-button" title="Open repository" aria-label="Open repository" @click="store.chooseRepository"><FolderOpen :size="15" /></button>
        <div v-if="store.repository" class="branch-button"><GitBranch :size="13" /><strong>{{ store.repository.branch }}</strong><span>{{ branchState }}</span></div>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>{{ store.repository?.name ?? 'Graft' }}</div>
      <div class="toolbar-actions">
        <label class="theme-picker" title="Appearance"><Palette :size="14" /><select v-model="theme" aria-label="Appearance theme"><option v-for="item in themes" :key="item.id" :value="item.id">{{ item.label }}</option></select><ChevronDown :size="11" /></label>
        <button :disabled="!store.repository" @click="store.remote('fetch')"><ArrowDownToLine :size="14" /><span>Fetch</span></button>
        <button :disabled="!store.repository" @click="store.remote('pull')"><GitPullRequestArrow :size="14" /><span>Pull</span></button>
        <button :disabled="!store.repository" @click="store.remote('push')"><ArrowUpFromLine :size="14" /><span>Push</span></button>
        <button class="icon-button" :disabled="!store.repository" title="Refresh" aria-label="Refresh repository" @click="store.refresh"><RefreshCw :size="14" /></button>
      </div>
    </header>

    <template v-if="store.repository">
      <RepositorySidebar v-if="showSidebar" :repository="store.repository" @add-worktree="worktreeDialog = true" @add-reference="newReference = $event" @checkout="checkout" @open-worktree="openWorktree" />
      <div v-if="showSidebar" class="pane-resizer sidebar-resizer" role="separator" aria-label="Resize repository sidebar" aria-orientation="vertical" :aria-valuenow="Math.round(sidebarWidth)" aria-valuemin="176" aria-valuemax="360" tabindex="0" title="Drag to resize repository sidebar" @pointerdown="startPaneResize('sidebar', $event)" @keydown="resizePaneWithKeyboard('sidebar', $event)" />
      <section class="workspace">
        <div class="log-toolbar">
          <div class="search-field"><Search :size="13" /><input ref="searchInput" v-model="store.query" aria-label="Search commits" placeholder="Search commits" /><button v-if="store.query" aria-label="Clear search" @click="store.query = ''"><X :size="12" /></button><kbd>⌘F</kbd></div>
          <button title="Merge a branch into the current branch" @click="historyOperation = 'merge'"><GitMerge :size="13" />Merge</button>
          <button title="Interactively rebase the current branch" @click="rebaseDialog = true">Rebase…</button>
          <button :disabled="!store.selectedCommit" title="Cherry-pick selected commit" @click="historyOperation = 'cherryPick'">Cherry-pick</button>
          <button :disabled="!store.selectedCommit" title="Revert selected commit" @click="historyOperation = 'revert'"><RotateCcw :size="12" />Revert</button>
          <button :disabled="!store.selectedCommit" title="Reset current branch to selected commit" @click="historyOperation = 'reset'">Reset…</button>
          <span class="history-count">{{ store.commits.length.toLocaleString() }} loaded</span>
        </div>
        <div class="log-columns" aria-hidden="true"><span>Graph &amp; Commit</span><span>Author</span><span>Hash</span><span>Date</span></div>
        <CommitGraph :commits="store.visibleCommits" :selected="store.selectedCommit?.oid" :has-more="store.hasMore" :loading-more="store.loadingMore" @select="store.selectCommit" @more="store.loadMore" />
        <CommitPanel :commit="store.selectedCommit" :detail="store.detail" />
      </section>
      <CommitToolWindow v-if="showCommit" ref="commitTool" :changes="store.repository.changes" :truncated="store.repository.changesTruncated" :busy="store.loading" @staged="store.setStaged" @commit="doCommit" @resolve="conflictFile = $event" @inspect="hunkFile = $event" />
      <div v-if="showCommit" class="pane-resizer commit-resizer" role="separator" aria-label="Resize commit panel" aria-orientation="vertical" :aria-valuenow="Math.round(commitWidth)" aria-valuemin="240" aria-valuemax="420" tabindex="0" title="Drag to resize commit panel" @pointerdown="startPaneResize('commit', $event)" @keydown="resizePaneWithKeyboard('commit', $event)" />
      <nav class="tool-stripe" aria-label="Tool windows">
        <button :class="{ active: showCommit }" title="Commit" @click="showCommit = !showCommit"><Archive :size="15" /><span>Commit</span><b v-if="store.repository.changes.length">{{ store.repository.changes.length }}</b></button>
        <button title="Worktrees" @click="worktreeDialog = true"><TreePine :size="15" /><span>Worktrees</span></button>
      </nav>
      <footer class="statusbar">
        <span><GitBranch :size="11" />{{ store.repository.branch }}</span>
        <span v-if="store.repository.state.rebasing" class="operation"><LoaderCircle :size="11" />Rebase in progress</span>
        <span v-if="store.repository.state.merging" class="operation"><LoaderCircle :size="11" />Merge in progress</span>
        <span v-if="store.repository.state.conflicts" class="danger"><CircleAlert :size="11" />{{ store.repository.state.conflicts }} conflicts</span>
        <span v-if="activeOperation" class="operation-actions"><button :disabled="store.repository.state.conflicts > 0" @click="finishOperation('continue')">Continue</button><button @click="finishOperation('abort')">Abort</button></span>
        <span class="status-path">{{ store.repository.root }}</span>
      </footer>
    </template>

    <section v-else class="welcome">
      <div class="welcome-mark"><img src="/graft-icon.png" alt="" /></div>
      <h1>Open a Git repository</h1>
      <p>Inspect history, prepare precise commits, and move between worktrees without the weight of an IDE.</p>
      <button class="primary-button welcome-open" :disabled="store.loading" @click="store.chooseRepository"><FolderOpen :size="15" />{{ store.loading ? 'Opening…' : 'Open Repository…' }}</button>
      <div class="welcome-notes"><span><kbd>⌘O</kbd> Open</span><span>macOS 26 · Apple silicon</span></div>
    </section>

    <button v-if="store.error || store.notice" class="toast" :class="{ error: store.error }" @click="store.clearMessage">
      <CircleAlert v-if="store.error" :size="15" /><RefreshCw v-else :size="15" />
      <span>{{ store.error || store.notice }}</span><X :size="13" />
    </button>
    <WorktreeDialog v-if="worktreeDialog && store.repository" :repository="store.repository" @close="worktreeDialog = false" @complete="worktreeComplete" />
    <HistoryOperationDialog v-if="historyOperation && store.repository" :repository-path="store.repository.root" :operation="historyOperation" :initial-target="historyOperation === 'merge' ? '' : store.selectedCommit?.oid" @close="historyOperation = undefined" @complete="operationComplete" />
    <ConflictResolver v-if="conflictFile && store.repository" :repository-path="store.repository.root" :file="conflictFile" @close="conflictFile = ''" @complete="conflictComplete" />
    <RebaseDialog v-if="rebaseDialog && store.repository" :repository-path="store.repository.root" @close="rebaseDialog = false" @complete="rebaseComplete" />
    <HunkSelector v-if="hunkFile && store.repository" :repository-path="store.repository.root" :file="hunkFile" @close="hunkFile = ''" @changed="store.repository = $event" />
    <NewReferenceDialog v-if="newReference && store.repository" :repository-path="store.repository.root" :kind="newReference" @close="newReference = undefined" @complete="referenceComplete" />
    <UpdateBanner />
  </main>
</template>
