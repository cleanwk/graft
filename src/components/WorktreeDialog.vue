<script setup lang="ts">
import { computed, ref } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { ExternalLink, Lock, RefreshCw, Trash2, Unlock, X } from "@lucide/vue";
import { api } from "../lib/bridge";
import type { RepositorySnapshot } from "../types";

const props = defineProps<{ repository: RepositorySnapshot }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const selectedBranch = ref(props.repository.branch);
const path = ref(`${props.repository.root}-worktrees/${props.repository.branch}`);
const createBranch = ref(false); const branchName = ref(""); const busy = ref(false); const error = ref("");
const targetBranch = computed(() => createBranch.value ? branchName.value.trim() : selectedBranch.value);

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
      <div class="worktree-manager">
        <div v-for="worktree in repository.worktrees" :key="worktree.path" class="worktree-manage-row">
          <div><strong>{{ worktree.branch ?? 'Detached HEAD' }}</strong><span>{{ worktree.path }}</span></div>
          <em v-if="worktree.path === repository.root">Current</em><em v-else-if="worktree.locked">Locked</em>
          <button type="button" title="Open in new window" aria-label="Open worktree in new window" @click="openWindow(worktree.path)"><ExternalLink :size="13" /></button>
          <button v-if="worktree.path !== repository.root" type="button" :title="worktree.locked ? 'Unlock' : 'Lock'" :aria-label="worktree.locked ? 'Unlock worktree' : 'Lock worktree'" @click="action(worktree.locked ? 'unlock' : 'lock', worktree.path)"><Unlock v-if="worktree.locked" :size="13" /><Lock v-else :size="13" /></button>
          <button v-if="worktree.path !== repository.root" type="button" title="Remove worktree" aria-label="Remove worktree" class="danger-icon" @click="action('remove', worktree.path)"><Trash2 :size="13" /></button>
        </div>
        <button type="button" class="prune-button" @click="action('prune')"><RefreshCw :size="12" />Prune stale entries</button>
      </div>
      <p class="worktree-add-intro">Add a worktree without disturbing the branch and changes in this window.</p>
      <label class="field-label">Branch</label>
      <select v-model="selectedBranch" :disabled="createBranch">
        <option v-for="branch in repository.branches.filter(b => !b.remote)" :key="branch.name" :value="branch.name">{{ branch.name }}</option>
      </select>
      <label class="checkbox-row"><input v-model="createBranch" type="checkbox" />Create a new branch</label>
      <input v-if="createBranch" v-model="branchName" placeholder="feature/my-work" aria-label="New branch name" />
      <label class="field-label">Worktree location</label>
      <div class="path-field"><input v-model="path" aria-label="Worktree location" /><button type="button" @click="choosePath">Choose…</button></div>
      <div v-if="error" class="inline-error">{{ error }}</div>
      <footer><button type="button" @click="emit('close')">Cancel</button><button class="primary-button" :disabled="busy || !path.trim() || !targetBranch" type="submit">{{ busy ? 'Creating…' : 'Create Worktree' }}</button></footer>
    </form>
  </div>
</template>
