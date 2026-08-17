<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { ExternalLink, Lock, RefreshCw, Search, Trash2, Unlock, X } from "@lucide/vue";
import { api } from "../lib/bridge";
import type { RepositorySnapshot } from "../types";

const props = defineProps<{ repository: RepositorySnapshot }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const usedBranches = new Set(props.repository.worktrees.flatMap((worktree) => worktree.branch ? [worktree.branch] : []));
const localBranches = props.repository.branches.filter((branch) => !branch.remote);
const availableBranches = localBranches.filter((branch) => !usedBranches.has(branch.name));
const selectedBranch = ref(availableBranches[0]?.name ?? props.repository.branch);
const createBranch = ref(!availableBranches.length); const branchName = ref(""); const busy = ref(false); const error = ref("");
const search = ref("");
const filteredWorktrees = computed(() => {
  const needle = search.value.trim().toLowerCase();
  if (!needle) return props.repository.worktrees;
  return props.repository.worktrees.filter((worktree) => `${worktree.branch ?? "detached head"} ${worktree.path} ${worktree.head}`.toLowerCase().includes(needle));
});
const filteredBranches = computed(() => {
  const needle = search.value.trim().toLowerCase();
  return needle ? localBranches.filter((branch) => branch.name.toLowerCase().includes(needle)) : localBranches;
});
const targetBranch = computed(() => createBranch.value ? branchName.value.trim() : selectedBranch.value);
const suggestedPath = (branch: string) => `${props.repository.root}-worktrees/${branch.replaceAll('/', '-')}`;
const path = ref(suggestedPath(targetBranch.value));
const validTarget = computed(() => createBranch.value ? Boolean(branchName.value.trim()) : availableBranches.some((branch) => branch.name === selectedBranch.value));

watch(targetBranch, (value, previous) => {
  if (path.value === suggestedPath(previous)) path.value = suggestedPath(value);
});

async function choosePath() {
  const chosen = await open({ directory: true, multiple: false, title: "Choose Worktree Parent Folder" });
  if (typeof chosen === "string") path.value = `${chosen}/${targetBranch.value.replaceAll('/', '-')}`;
}
async function submit() {
  busy.value = true; error.value = "";
  try { const result = await api.addWorktree(props.repository.root, path.value, targetBranch.value, createBranch.value); emit("complete", result.summary); }
  catch (caught) { error.value = typeof caught === "string" ? caught : (caught as { message?: string }).message ?? "Could not create the worktree."; }
  finally { busy.value = false; }
}
async function action(kind: "prune" | "lock" | "unlock" | "remove", worktreePath?: string) {
  if (kind === "remove" && worktreePath && !await confirm(`Remove the worktree at ${worktreePath}? Uncommitted files prevent removal unless Force is chosen.`, { title: "Remove Worktree", kind: "warning", okLabel: "Remove" })) return;
  busy.value = true; error.value = "";
  try { const result = await api.worktreeAction(props.repository.root, kind, worktreePath); emit("complete", result.summary); }
  catch (caught) { error.value = typeof caught === "string" ? caught : (caught as { message?: string }).message ?? "The worktree operation failed."; }
  finally { busy.value = false; }
}
async function openWindow(worktreePath: string) { await api.openWindow(worktreePath); }
</script>

<template>
  <div class="dialog-backdrop" @mousedown.self="emit('close')">
    <form class="dialog" role="dialog" aria-modal="true" aria-labelledby="worktree-title" @submit.prevent="submit">
      <header><h2 id="worktree-title">Worktrees</h2><button type="button" class="icon-button" aria-label="Close" @click="emit('close')"><X :size="15" /></button></header>
      <div class="dialog-search"><Search :size="13" /><input v-model="search" type="search" aria-label="Search worktrees and branches" placeholder="Search worktrees and branches" /><button v-if="search" type="button" aria-label="Clear search" @click="search = ''"><X :size="12" /></button></div>
      <div class="worktree-manager">
        <div v-for="worktree in filteredWorktrees" :key="worktree.path" class="worktree-manage-row">
          <div><strong>{{ worktree.branch ?? 'Detached HEAD' }}</strong><span>{{ worktree.path }}</span></div>
          <em v-if="worktree.path === repository.root">Current</em><em v-else-if="worktree.locked">Locked</em>
          <button type="button" title="Open in new window" aria-label="Open worktree in new window" @click="openWindow(worktree.path)"><ExternalLink :size="13" /></button>
          <button v-if="worktree.path !== repository.root" type="button" :title="worktree.locked ? 'Unlock' : 'Lock'" :aria-label="worktree.locked ? 'Unlock worktree' : 'Lock worktree'" @click="action(worktree.locked ? 'unlock' : 'lock', worktree.path)"><Unlock v-if="worktree.locked" :size="13" /><Lock v-else :size="13" /></button>
          <button v-if="worktree.path !== repository.root" type="button" title="Remove worktree" aria-label="Remove worktree" class="danger-icon" @click="action('remove', worktree.path)"><Trash2 :size="13" /></button>
        </div>
        <p v-if="!filteredWorktrees.length" class="empty-filter">No matching worktrees.</p>
        <button type="button" class="prune-button" @click="action('prune')"><RefreshCw :size="12" />Prune stale entries</button>
      </div>
      <p class="worktree-add-intro">Add a worktree without disturbing the branch and changes in this window.</p>
      <label class="field-label">Branch</label>
      <select v-model="selectedBranch" :disabled="createBranch" aria-label="Worktree branch">
        <option v-for="branch in filteredBranches" :key="branch.name" :value="branch.name" :disabled="usedBranches.has(branch.name)">{{ branch.name }}{{ usedBranches.has(branch.name) ? ' — already checked out' : '' }}</option>
      </select>
      <label class="checkbox-row"><input v-model="createBranch" type="checkbox" />Create a new branch</label>
      <input v-if="createBranch" v-model="branchName" placeholder="feature/my-work" aria-label="New branch name" />
      <label class="field-label">Worktree location</label>
      <div class="path-field"><input v-model="path" aria-label="Worktree location" /><button type="button" @click="choosePath">Choose…</button></div>
      <div v-if="error" class="inline-error">{{ error }}</div>
      <footer><button type="button" @click="emit('close')">Cancel</button><button class="primary-button" :disabled="busy || !path.trim() || !validTarget" type="submit">{{ busy ? 'Creating…' : 'Create Worktree' }}</button></footer>
    </form>
  </div>
</template>
