<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Archive, ArrowDownToLine, ArrowUpFromLine, Braces, ChevronDown, CircleAlert, FolderOpen, GitBranch, GitMerge, GitPullRequestArrow, ListFilter, LoaderCircle, PanelLeftClose, RefreshCw, RotateCcw, Search, Settings2, TreePine, X } from "@lucide/vue";
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
import { useRepositoryStore } from "./stores/repository";
import { api } from "./lib/bridge";

const store = useRepositoryStore();
const showSidebar = ref(true); const showCommit = ref(true); const worktreeDialog = ref(false); const commitTool = ref<InstanceType<typeof CommitToolWindow>>();
const historyOperation = ref<"merge" | "cherryPick" | "revert" | "reset">(); const conflictFile = ref("");
const rebaseDialog = ref(false);
const hunkFile = ref("");
const newReference = ref<"branch" | "tag" | "remote">();
const branchState = computed(() => store.repository ? `${store.repository.ahead ? `↑${store.repository.ahead}` : ''}${store.repository.behind ? ` ↓${store.repository.behind}` : ''}` : "");
const activeOperation = computed(() => { const state = store.repository?.state; if (state?.rebasing) return "rebase"; if (state?.merging) return "merge"; if (state?.cherryPicking) return "cherryPick"; if (state?.reverting) return "revert"; return ""; });

async function doCommit(message: string, amend: boolean) { if (await store.commit(message, amend)) commitTool.value?.clear(); }
function worktreeComplete(message: string) { worktreeDialog.value = false; store.notice = message; store.refresh(); }
function operationComplete(message: string) { historyOperation.value = undefined; store.notice = message; store.refresh(); }
function conflictComplete(message: string) { conflictFile.value = ""; store.notice = message; store.refresh(); }
function rebaseComplete(message: string) { rebaseDialog.value = false; store.notice = message; store.refresh(); }
function referenceComplete(message: string) { newReference.value = undefined; store.notice = message; store.refresh(); }
async function checkout(branch: string) { if (!store.repository) return; try { const result = await api.checkout(store.repository.root, branch, false); store.notice = result.summary; await store.refresh(); } catch (caught) { store.error = String(caught); } }
async function finishOperation(action: "continue" | "abort") { if (!store.repository || !activeOperation.value) return; try { const result = await api.finishOperation(store.repository.root, activeOperation.value, action); store.notice = result.summary; await store.refresh(); } catch (caught) { store.error = String(caught); await store.refresh(); } }
onMounted(() => store.restore());
</script>

<template>
  <main class="app-shell" :class="{ 'sidebar-hidden': !showSidebar, 'commit-hidden': !showCommit }">
    <header class="titlebar" data-tauri-drag-region>
      <div class="traffic-space" data-tauri-drag-region />
      <button class="icon-button" title="Show or hide repository tree" aria-label="Toggle repository tree" @click="showSidebar = !showSidebar"><PanelLeftClose :size="15" /></button>
      <button v-if="store.repository" class="branch-button"><GitBranch :size="13" /><strong>{{ store.repository.branch }}</strong><span>{{ branchState }}</span><ChevronDown :size="12" /></button>
      <div class="titlebar-center" data-tauri-drag-region>{{ store.repository?.name ?? 'Graft' }}</div>
      <div class="toolbar-actions">
        <button :disabled="!store.repository" @click="store.remote('fetch')"><ArrowDownToLine :size="14" /><span>Fetch</span></button>
        <button :disabled="!store.repository" @click="store.remote('pull')"><GitPullRequestArrow :size="14" /><span>Pull</span></button>
        <button :disabled="!store.repository" @click="store.remote('push')"><ArrowUpFromLine :size="14" /><span>Push</span></button>
        <button class="icon-button" :disabled="!store.repository" title="Refresh" aria-label="Refresh repository" @click="store.refresh"><RefreshCw :size="14" /></button>
        <button class="icon-button" title="Settings" aria-label="Settings"><Settings2 :size="14" /></button>
      </div>
    </header>

    <template v-if="store.repository">
      <RepositorySidebar v-if="showSidebar" :repository="store.repository" @add-worktree="worktreeDialog = true" @add-reference="newReference = $event" @checkout="checkout" />
      <section class="workspace">
        <div class="log-toolbar">
          <div class="search-field"><Search :size="13" /><input v-model="store.query" aria-label="Search commits" placeholder="Search commits" /><button v-if="store.query" aria-label="Clear search" @click="store.query = ''"><X :size="12" /></button><kbd>⌘F</kbd></div>
          <button><ListFilter :size="13" />All branches<ChevronDown :size="11" /></button>
          <button><Braces :size="13" />Paths</button>
          <span class="toolbar-separator" />
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
  </main>
</template>
